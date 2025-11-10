use std::sync::Arc;
use crate::models::OnlineData;

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<tokio::sync::RwLock<OnlineData>>, // 内存缓存
    pub public_key_der: Arc<Vec<u8>>,                // RSA 公钥（DER）
    pub external_api_url: Arc<String>,               // 外部 API 地址
    pub poll_interval_seconds: u64,                  // 轮询间隔
}