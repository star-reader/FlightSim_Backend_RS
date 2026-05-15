use std::{net::SocketAddr, sync::Arc};

use axum::{
    body::{Body, to_bytes},
    extract::connect_info::ConnectInfo,
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
use pem::parse as pem_parse;
use ring::{
    rand::SystemRandom,
    signature::{RSA_PSS_SHA256, RsaKeyPair},
};
use serde_json::Value;
use sim_flight_backend::{
    app::build_app,
    cache::online_data::update_cache,
    models::{BaseUser, OnlineData, OnlinePilot},
    state::AppState,
};
use tower::ServiceExt;

const TEST_PUBLIC_KEY_PEM: &str = "-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEA2MBtQHMCoVjM8sqScpm7OqWl+rnJlhdcrme5Z6YQzDtiTVwVoe/A
09Qp3nuNPYEhiug/ZpyUtsCieOMk20hl18TDtIbYwVfXfArlC0H0JLB/3r2Xu4qp
OM9Pnul3wqpDrz1t28alUpV60f5NMDfE+CzaKBZVEnC9zrPgwclPpY9TWAfao0WT
As+Ta7CsLsHr7CT/fxGxfxYryQlzrXX8iN0tkR6/MSj4ry41ZvtKR5QwchAXu3CR
jl7DIJEGfS7QbVuq3dgwPDdPCGoiPkh/BvJWt0ad8OAxksYh4J4y0RAVYbrHGFiC
mnXpNi+enR5GO2kn98TYI9mGk759ncB5HQIDAQAB
-----END RSA PUBLIC KEY-----";

const TEST_PRIVATE_KEY_DER_BASE64: &str = "MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDYwG1AcwKhWMzyypJymbs6paX6ucmWF1yuZ7lnphDMO2JNXBWh78DT1Cnee409gSGK6D9mnJS2wKJ44yTbSGXXxMO0htjBV9d8CuULQfQksH/evZe7iqk4z0+e6XfCqkOvPW3bxqVSlXrR/k0wN8T4LNooFlUScL3Os+DByU+lj1NYB9qjRZMCz5NrsKwuwevsJP9/EbF/FivJCXOtdfyI3S2RHr8xKPivLjVm+0pHlDByEBe7cJGOXsMgkQZ9LtBtW6rd2DA8N08IaiI+SH8G8la3Rp3w4DGSxiHgnjLREBVhuscYWIKadek2L56dHkY7aSf3xNgj2YaTvn2dwHkdAgMBAAECggEAD1ZyhKx+w6cSfZ27BjP9rTU6jQbnjmDh1I+PbZexOZp8Jsg82aQrC3JIci9g+7ivBHLRxbOPzGzOMmq0WXYCiuWoCZa/MNND8smckpLcsVnx+nOFEKbLubLlyfNwX4yH1Jcutm4UlcbxFXn+wUo4/GzEFTEbXP4wWDefUGcr5SWhWKjo5UcwN4AzMu88a2d7vx+lhoitpgxcvnLeBZdeCBaReVeZlmifpHkD+HkYf7qRa0byvq+9d1cyK9jGNz2o12vhUYHa42NBHKuFI/E8ZtudpGqxn/fLtA+Jbn3hqtrlsK6Y2pKsX2bL9ef2wH/tri21Ek62rUxNQR8ijh1xawKBgQD4JOb5iXS1M11wZJqgrAWSQ2c0OrptnkYy2AauUwFEl4nVIQJVYxBONrkR3RaEgVCM5Httj2jJubwzzmlNR7+9M03bWGKk2dYHeDmmGQdxF3Q1nattjxfb29g4ZDegWjq6NWVSstWFii/JYUwASA+0zRLxgfEqM1Tjyru1zB++vwKBgQDfnRo5cQfZG5L/qcxNjWUmbPadzErjyUuxIkXJt3M4piJ89YfQHy0500GjakJXfYELCbzZxsNLPCFNp9T41BBtLJmYy2sW5PPskRC6AGnbgM+30+UaNZj9UPUr649+6NmTnO4rYd3o0+25x3qro1LX3PDD1QjsPAksYXw+PcTbIwKBgCMQq+cFSo20hXa1cLhVtq/VgIb76Y5F+GuE5WTK6/nyZBLPCFJinlI/H2Xr1RvCXkyamOqBgnBHI9Y6S90Uz35/+neNhNsAwWLq1VKoZMnRY6WT8z6XIur5pRNK7iVlurjHhNrr1Ip7XYdajW/Yvwdwl0WzdA3flRZfdqkZAD9nAoGBAKlw5fo4x/Wfi1fnVkQ9rDPdxVH1B5prYcuraK5dMBXw7Zk6g9sUhLSYe9hUIEkjdLDkhBGRJ1LPEX1Ce4LBTmErtoetPExw2NSdZ8tJnO0TQlLNYYDmtDLfMahCKs3Dq3rLIz4EPp66WBy6Jx3/+DpR1K7Psp8vLtROWlLfg1d5AoGBAIqnqvRQ0DVKMGMPI3yEf9sbXNM9gXwxxgpKoxWRbDLuRJeqErjBU1f9/JvHrvtmStRKK8VNVnyk7+PqULwvDn+QPQ9aQ5kvuaLV3b8LQb8c1TEnDNOGQbZCtPXXATm0fzgwRHG7Wnq5PhQ/tg10LFU95yoeT+IbFOwyHMUKYKZd";

#[tokio::test]
async fn health_uses_real_route() {
    let app = build_app(test_state());
    let response = app.oneshot(request("/map/v2/health")).await.unwrap();

    assert_response_status(response, StatusCode::OK).await;
}

#[tokio::test]
async fn protected_route_rejects_missing_auth_headers() {
    let app = build_app(test_state());
    let response = app.oneshot(request("/map/v2/online-list")).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn health_route_is_not_rate_limited() {
    let app = build_app(test_state());

    for _ in 0..80 {
        let response = app
            .clone()
            .oneshot(request("/map/v2/health"))
            .await
            .unwrap();
        assert_response_status(response, StatusCode::OK).await;
    }
}

#[tokio::test]
async fn online_list_returns_cached_data_with_valid_auth() {
    let state = test_state();
    update_cache(
        &state.cache,
        OnlineData {
            flights: vec![pilot("1001", "session-1", "CCA123")],
            controllers: Vec::new(),
            atis: Vec::new(),
        },
    );
    let app = build_app(state);
    let timestamp = current_timestamp();

    let response = app
        .oneshot(signed_request(
            "/map/v2/online-list",
            "client-a",
            &timestamp,
        ))
        .await
        .unwrap();

    let response = assert_response_status(response, StatusCode::OK).await;
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["flights"][0]["callsign"], "CCA123");
}

#[tokio::test]
async fn signature_covers_query_string() {
    let app = build_app(test_state());
    let timestamp = current_timestamp();

    let accepted = app
        .clone()
        .oneshot(signed_request(
            "/map/v2/online-list?scope=all",
            "client-a",
            &timestamp,
        ))
        .await
        .unwrap();
    let rejected = app
        .oneshot(signed_request_with_signature(
            "/map/v2/online-list?scope=all",
            "client-b",
            &timestamp,
            &sign("/map/v2/online-list", "client-b", &timestamp),
        ))
        .await
        .unwrap();

    assert_response_status(accepted, StatusCode::OK).await;
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn repeated_signature_is_rejected() {
    let app = build_app(test_state());
    let timestamp = current_timestamp();
    let signature = sign("/map/v2/online-list", "client-a", &timestamp);
    let first = app
        .clone()
        .oneshot(signed_request_with_signature(
            "/map/v2/online-list",
            "client-a",
            &timestamp,
            &signature,
        ))
        .await
        .unwrap();
    let second = app
        .oneshot(signed_request_with_signature(
            "/map/v2/online-list",
            "client-a",
            &timestamp,
            &signature,
        ))
        .await
        .unwrap();

    assert_response_status(first, StatusCode::OK).await;
    assert_eq!(second.status(), StatusCode::UNAUTHORIZED);
}

async fn assert_response_status(
    response: axum::response::Response,
    expected: StatusCode,
) -> axum::response::Response {
    let status = response.status();
    if status != expected {
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        panic!(
            "expected status {expected}, got {status}, body: {}",
            String::from_utf8_lossy(&body)
        );
    }
    response
}

fn test_state() -> AppState {
    let pem = pem_parse(TEST_PUBLIC_KEY_PEM).unwrap();
    AppState::new(
        Arc::new(pem.contents),
        "https://example.com/online.json".parse().unwrap(),
        15,
    )
}

fn request(uri: &str) -> Request<Body> {
    let mut request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 50000))));
    request
}

fn signed_request(path: &str, id: &str, timestamp: &str) -> Request<Body> {
    let signature = sign(path, id, timestamp);
    signed_request_with_signature(path, id, timestamp, &signature)
}

fn signed_request_with_signature(
    path: &str,
    id: &str,
    timestamp: &str,
    signature: &str,
) -> Request<Body> {
    let mut request = request(path);
    let headers = request.headers_mut();
    headers.insert("x-id", id.parse().unwrap());
    headers.insert("x-timestamp", timestamp.parse().unwrap());
    headers.insert("x-signature", signature.parse().unwrap());
    request
}

fn sign(path: &str, id: &str, timestamp: &str) -> String {
    let private_key_der = BASE64_STANDARD.decode(TEST_PRIVATE_KEY_DER_BASE64).unwrap();
    let key_pair = RsaKeyPair::from_pkcs8(&private_key_der).unwrap();
    let rng = SystemRandom::new();
    let message = format!("{path}|{id}|{timestamp}");
    let mut signature = vec![0; key_pair.public().modulus_len()];
    key_pair
        .sign(&RSA_PSS_SHA256, &rng, message.as_bytes(), &mut signature)
        .unwrap();
    BASE64_STANDARD.encode(signature)
}

fn current_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

fn pilot(cid: &str, session_id: &str, callsign: &str) -> OnlinePilot {
    OnlinePilot {
        base: BaseUser {
            cid: cid.to_string(),
            name: "Test Pilot".to_string(),
            callsign: callsign.to_string(),
            server: "TEST".to_string(),
            session_id: session_id.to_string(),
            logon_time: "2026-05-15T00:00:00Z".to_string(),
        },
        latitude: 39.9,
        longitude: 116.4,
        altitude: 33000,
        groundspeed: 450,
        transponder: 2200,
        heading: 90,
        bank: 0,
        pitch: 0,
        flight_plan: None,
    }
}
