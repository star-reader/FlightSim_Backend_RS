use std::sync::Arc;

use arc_swap::ArcSwap;
use reqwest::Url;

use crate::auth::handler::ReplayCache;
use crate::cache::online_data::OnlineCache;

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<ArcSwap<OnlineCache>>,
    pub public_key_der: Arc<Vec<u8>>,
    pub external_api_url: Arc<Url>,
    pub poll_interval_seconds: u64,
    pub replay_cache: Arc<std::sync::Mutex<ReplayCache>>,
}

impl AppState {
    pub fn new(
        public_key_der: Arc<Vec<u8>>,
        external_api_url: Url,
        poll_interval_seconds: u64,
    ) -> Self {
        Self {
            cache: Arc::new(ArcSwap::from_pointee(OnlineCache::default())),
            public_key_der,
            external_api_url: Arc::new(external_api_url),
            poll_interval_seconds,
            replay_cache: Arc::new(std::sync::Mutex::new(ReplayCache::default())),
        }
    }
}
