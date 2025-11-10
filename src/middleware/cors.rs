use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderName;

// CORS 配置：开发环境允许任意 Origin；生产可按需限制
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([axum::http::Method::GET])
        .allow_origin(Any)
        .allow_headers([
            HeaderName::from_static("x-id"),
            HeaderName::from_static("x-timestamp"),
            HeaderName::from_static("x-signature"),
            HeaderName::from_static("content-type"),
        ])
}