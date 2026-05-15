use reqwest::Client;
use std::time::{Duration, Instant};
use tracing::{error, info};

use crate::{cache::online_data::update_cache, models::OnlineData, state::AppState};

const REQUEST_TIMEOUT_SECONDS: u64 = 10;

// 后台轮询任务
pub fn spawn_polling(state: AppState) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let client = Client::builder()
            // 虽然但是，我们其实那个接口不会统计ua的，但也依然写上吧
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0")
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
            .build()
            .expect("build http cient failed");

        let mut interval = tokio::time::interval(Duration::from_secs(state.poll_interval_seconds));
        loop {
            interval.tick().await;
            let started_at = Instant::now();
            match client.get(state.external_api_url.as_str()).send().await {
                Ok(resp) => match resp.error_for_status() {
                    Ok(resp) => match resp.json::<OnlineData>().await {
                        Ok(data) => {
                            let flights = data.flights.len();
                            let controllers = data.controllers.len();
                            update_cache(&state.cache, data);
                            info!(
                                elapsed_ms = started_at.elapsed().as_millis(),
                                flights, controllers, "缓存已更新"
                            );
                        }
                        Err(e) => error!("JSON 解析失败: {}", e),
                    },
                    Err(e) => error!("外部 API 返回非成功状态: {}", e),
                },
                Err(e) => error!("Fetch外部 API 请求失败: {}", e),
            }
        }
    })
}
