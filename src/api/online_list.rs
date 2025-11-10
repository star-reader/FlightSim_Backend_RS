use axum::{extract::State, response::IntoResponse, Json};
use crate::state::AppState;
use super::types::ApiOk;

pub async fn online_list(State(state): State<AppState>) -> impl IntoResponse {
    let data = state.cache.read().await.clone();
    Json(ApiOk { code: 200, data })
}