use std::{net::SocketAddr, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    middleware::{from_fn, Next},
    response::{IntoResponse},
    routing::{get},
    Router,
};
use dotenvy::dotenv;
use pem::parse as pem_parse;
use ring::signature::{UnparsedPublicKey, RSA_PSS_2048_8192_SHA256};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tower_governor::{GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

mod models;
mod cache;
mod polling;
mod state;
mod api;
mod auth;
mod middleware;

// AppState 迁移至 state 模块

#[derive(Serialize, Deserialize)]
struct ApiOk<T> {
    code: u16,
    data: T,
}

#[derive(Serialize, Deserialize)]
struct ApiErr {
    code: u16,
    error: String,
}


async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"code":200, "data": {"status":"ok"}}))
}
#[tokio::main]
async fn main() {
    dotenv().ok();

    // 初始化日志
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // 读取环境变量
    let bind_addr: SocketAddr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3490".to_string()).parse().expect("BIND_ADDR 格式错误");
    let external_api_url = std::env::var("EXTERNAL_API_URL").unwrap_or_else(|_| "https://go.api.skylineflyleague.cn/Map/GetOnlineList".to_string());
    let poll_interval_seconds: u64 = std::env::var("POLL_INTERVAL_SECONDS").ok().and_then(|s| s.parse().ok()).unwrap_or(15);
    let public_key_pem = std::env::var("RSA_PUBLIC_KEY").expect("必须设置 RSA_PUBLIC_KEY 环境变量（PEM）");

    // 解析 PEM 为 DER
    let pem = pem_parse(public_key_pem).expect("解析 RSA 公钥 PEM 失败");
    let public_key_der = Arc::new(pem.contents);

    let state = state::AppState {
        cache: Arc::new(tokio::sync::RwLock::new(models::OnlineData::default())),
        public_key_der,
        external_api_url: Arc::new(external_api_url.clone()),
        poll_interval_seconds,
    };

    // 启动轮询任务
    let poll_state = state.clone();
    let _poll_handle: JoinHandle<()> = polling::task::spawn_polling(poll_state);

    // 路由：鉴权作用在 /map/v2
    let protected = api::routes().layer(axum::middleware::from_fn(auth::handler::authorize));

    let cors = middleware::cors::cors_layer();

    let governor_conf = middleware::ratelimit::governor_config();

    let app = Router::new()
        .route("/api/health", get(health))
        .nest("/map/v2", protected)
        .with_state(state)
        .layer(cors)
        .layer(GovernorLayer::new(&governor_conf));

    info!("Server listening on {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        error!("server error: {}", e);
    }
}