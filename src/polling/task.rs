use reqwest::Client;
use std::time::{Duration, Instant};
use tracing::{error, info};

use crate::{cache::online_data::update_cache, models::OnlineData, state::AppState};

const REQUEST_TIMEOUT_SECONDS: u64 = 10;
const CONNECT_TIMEOUT_SECONDS: u64 = 3;

#[derive(Debug, PartialEq, Eq)]
pub struct PollStats {
    pub flights: usize,
    pub controllers: usize,
    pub elapsed_ms: u128,
}

#[derive(Debug)]
pub enum PollError {
    Request(reqwest::Error),
    Status(reqwest::StatusCode),
    Json(reqwest::Error),
}

// 后台轮询任务
pub fn spawn_polling(state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let client = build_client();

        let mut interval = tokio::time::interval(Duration::from_secs(state.poll_interval_seconds));
        loop {
            interval.tick().await;
            let url = state.external_api_url.as_str();
            match poll_once(&state, &client).await {
                Ok(stats) => {
                    info!(
                        url,
                        elapsed_ms = stats.elapsed_ms,
                        flights = stats.flights,
                        controllers = stats.controllers,
                        "缓存已更新"
                    );
                }
                Err(PollError::Status(status)) => {
                    error!(url, status = status.as_u16(), "外部 API 返回非成功状态");
                }
                Err(PollError::Json(err)) => {
                    error!(url, error = %err, "JSON 解析失败");
                }
                Err(PollError::Request(err)) => {
                    error!(url, error = %err, "Fetch外部 API 请求失败");
                }
            }
        }
    })
}

pub fn build_client() -> Client {
    Client::builder()
        // 虽然但是，我们其实那个接口不会统计ua的，但也依然写上吧
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0")
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECONDS))
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .build()
        .expect("build http client failed")
}

pub async fn poll_once(state: &AppState, client: &Client) -> Result<PollStats, PollError> {
    let started_at = Instant::now();
    let resp = client
        .get(state.external_api_url.as_ref().clone())
        .send()
        .await
        .map_err(PollError::Request)?;

    let status = resp.status();
    if !status.is_success() {
        return Err(PollError::Status(status));
    }

    let data = resp.json::<OnlineData>().await.map_err(PollError::Json)?;
    apply_poll_result(state, Ok(data), started_at.elapsed().as_millis())
}

pub fn apply_poll_result(
    state: &AppState,
    result: Result<OnlineData, PollError>,
    elapsed_ms: u128,
) -> Result<PollStats, PollError> {
    let data = result?;
    let stats = PollStats {
        flights: data.flights.len(),
        controllers: data.controllers.len(),
        elapsed_ms,
    };
    update_cache(&state.cache, data);
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::StatusCode;
    use serde_json::json;

    use super::*;

    #[test]
    fn poll_error_does_not_replace_existing_cache() {
        let state = test_state();
        update_cache(
            &state.cache,
            OnlineData {
                controllers: Vec::new(),
                flights: Vec::new(),
                atis: vec![json!({"previous": true})],
            },
        );

        let err = apply_poll_result(
            &state,
            Err(PollError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
            12,
        )
        .unwrap_err();

        assert!(matches!(
            err,
            PollError::Status(StatusCode::INTERNAL_SERVER_ERROR)
        ));
        assert_eq!(state.cache.load().data.atis.len(), 1);
    }

    #[test]
    fn successful_poll_result_replaces_cache() {
        let state = test_state();
        update_cache(
            &state.cache,
            OnlineData {
                controllers: Vec::new(),
                flights: Vec::new(),
                atis: vec![json!({"previous": true})],
            },
        );

        let stats = apply_poll_result(
            &state,
            Ok(OnlineData {
                controllers: Vec::new(),
                flights: Vec::new(),
                atis: Vec::new(),
            }),
            12,
        )
        .unwrap();

        assert_eq!(stats.flights, 0);
        assert_eq!(stats.elapsed_ms, 12);
        assert_eq!(state.cache.load().data.atis.len(), 0);
    }

    fn test_state() -> AppState {
        AppState::new(
            Arc::new(Vec::new()),
            "https://example.com/online.json".parse().unwrap(),
            15,
        )
    }
}
