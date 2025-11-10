use std::sync::Arc;
use crate::models::OnlineData;

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<tokio::sync::RwLock<OnlineData>>,
    pub public_key_der: Arc<Vec<u8>>,
    pub external_api_url: Arc<String>,
    pub poll_interval_seconds: u64
}