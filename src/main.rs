use std::{error::Error, net::SocketAddr, sync::Arc};

use dotenvy::dotenv;
use sim_flight_backend::{app, auth, config::AppConfig, polling, state::AppState};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    // 初始化日志
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // 读取环境变量
    let config = AppConfig::from_env().map_err(|err| format!("配置错误: {err}"))?;

    // 解析并校验 PEM 为 DER
    let public_key_der = Arc::new(auth::handler::parse_rsa_public_key_pem(
        &config.public_key_pem,
    )?);

    let state = AppState::new(
        public_key_der,
        config.external_api_url.clone(),
        config.poll_interval_seconds,
    );

    // 启动轮询任务
    let poll_state = state.clone();
    let _poll_handle = polling::task::spawn_polling(poll_state);

    let app = app::build_app(state);

    info!("Server listening on {}", config.bind_addr);
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        error!("server error: {}", e);
    }
    Ok(())
}
