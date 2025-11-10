use std::time::{SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use ring::signature::{RSA_PSS_2048_8192_SHA256, UnparsedPublicKey};

use crate::api::types::ApiErr;
use crate::state::AppState;

// 鉴权中间件：验证 RSA-PSS 签名（path|id|timestamp）
pub async fn authorize(
    State(state): State<AppState>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, axum::response::Response> {
    let headers = req.headers();
    let path = req.uri().path().to_string();

    match verify_request(&state, &path, headers) {
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

// 验证函数：RSA-PSS (SHA-256)
fn verify_request(
    state: &AppState,
    path: &str,
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
    if (now - timestamp).abs() > 60 {
        return Err((StatusCode::UNAUTHORIZED, "签名已过期或过早".to_string()));
    }

    let message = format!("{}|{}|{}", path, id, timestamp);
    let signature = BASE64_STANDARD
        .decode(sig_b64)
        .map_err(|_| (StatusCode::BAD_REQUEST, "签名 Base64 解码失败".to_string()))?;

    let public_key =
        UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA256, state.public_key_der.as_slice());
    public_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "RSA-PSS 验证失败".to_string()))?;
    Ok(())
}
