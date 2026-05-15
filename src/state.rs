use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::cache::online_data::OnlineCache;

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<ArcSwap<OnlineCache>>,
    pub public_key_der: Arc<Vec<u8>>,
    pub external_api_url: Arc<String>,
    pub poll_interval_seconds: u64,
}
