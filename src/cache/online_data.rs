use std::sync::Arc;
use crate::models::OnlineData;

// 简单封装一下
pub async fn update_cache(cache: &Arc<tokio::sync::RwLock<OnlineData>>, data: OnlineData) {
    let mut guard = cache.write().await;
    *guard = data;
}

pub async fn get_cache(cache: &Arc<tokio::sync::RwLock<OnlineData>>) -> OnlineData {
    cache.read().await.clone()
}