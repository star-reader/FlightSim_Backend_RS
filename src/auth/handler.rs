use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::Json;
use axum::extract::{OriginalUri, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use pem::parse as pem_parse;
use ring::signature::{RSA_PSS_2048_8192_SHA256, UnparsedPublicKey};

use crate::api::types::ApiErr;
use crate::state::AppState;

const SIGNATURE_WINDOW_SECONDS: i64 = 60;
const RSA_PUBLIC_KEY_PEM_TAG: &str = "RSA PUBLIC KEY";

#[derive(Debug, Default)]
pub struct ReplayCache {
    seen: HashMap<String, i64>,
}

impl ReplayCache {
    fn check_and_record(&mut self, key: String, now: i64) -> bool {
        self.purge_expired(now);
        if self.seen.contains_key(&key) {
            return false;
        }

        self.seen.insert(key, now + SIGNATURE_WINDOW_SECONDS);
        true
    }

    fn purge_expired(&mut self, now: i64) {
        self.seen.retain(|_, expires_at| *expires_at >= now);
    }
}

pub async fn authorize(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, axum::response::Response> {
    let headers = req.headers();
    let signing_target = req
        .extensions()
        .get::<OriginalUri>()
        .and_then(|uri| uri.0.path_and_query())
        .or_else(|| req.uri().path_and_query())
        .map(|path_and_query| path_and_query.as_str())
        .unwrap_or_else(|| req.uri().path())
        .to_string();
    // path_and_query|id|timestamp
    match verify_request(&state, &signing_target, headers) {
        Ok(_) => Ok(next.run(req).await),
        Err((status, msg)) => {
            let err = Json(ApiErr {
                code: status.as_u16(),
                error: msg,
            });
            Err((status, err).into_response())
        }
    }
}

pub fn parse_rsa_public_key_pem(public_key_pem: &str) -> Result<Vec<u8>, String> {
    let pem = pem_parse(public_key_pem).map_err(|err| format!("解析 RSA 公钥 PEM 失败: {err}"))?;
    if pem.tag != RSA_PUBLIC_KEY_PEM_TAG {
        return Err(format!(
            "RSA_PUBLIC_KEY 必须使用 PKCS#1 公钥格式（-----BEGIN RSA PUBLIC KEY-----），当前为 -----BEGIN {}-----",
            pem.tag
        ));
    }

    Ok(pem.contents)
}

// 验证函数：RSA-PSS (SHA-256)
fn verify_request(
    state: &AppState,
    signing_target: &str,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, String)> {
    let id = headers
        .get("x-id")
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "缺少 X-ID".to_string()))?;
    let ts_str = headers
        .get("x-timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "缺少 X-Timestamp".to_string()))?;
    let sig_b64 = headers
        .get("x-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "缺少 X-Signature".to_string()))?;

    let timestamp: i64 = ts_str
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "X-Timestamp 非法".to_string()))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "时间获取失败".to_string(),
            )
        })?
        .as_secs() as i64;
    if (now - timestamp).abs() > SIGNATURE_WINDOW_SECONDS {
        return Err((StatusCode::UNAUTHORIZED, "签名已过期或过早".to_string()));
    }

    let message = format!("{}|{}|{}", signing_target, id, timestamp);
    let signature = BASE64_STANDARD
        .decode(sig_b64)
        .map_err(|_| (StatusCode::BAD_REQUEST, "签名 Base64 解码失败".to_string()))?;

    let public_key =
        UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, state.public_key_der.as_slice());
    public_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "RSA-PSS 验证失败".to_string()))?;

    let replay_key = format!("{signing_target}|{id}|{timestamp}|{sig_b64}");
    let mut replay_cache = state.replay_cache.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "防重放缓存不可用".to_string(),
        )
    })?;
    if !replay_cache.check_and_record(replay_key, now) {
        return Err((StatusCode::UNAUTHORIZED, "请求已被使用".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PKCS1_PUBLIC_KEY: &str =
        "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEAwQIDAQAB\n-----END RSA PUBLIC KEY-----";
    const SPKI_PUBLIC_KEY: &str =
        "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A\n-----END PUBLIC KEY-----";

    #[test]
    fn accepts_pkcs1_rsa_public_key_pem() {
        assert!(parse_rsa_public_key_pem(PKCS1_PUBLIC_KEY).is_ok());
    }

    #[test]
    fn rejects_spki_public_key_pem_with_clear_message() {
        let err = parse_rsa_public_key_pem(SPKI_PUBLIC_KEY).unwrap_err();
        assert!(err.contains("BEGIN RSA PUBLIC KEY"));
    }
}
