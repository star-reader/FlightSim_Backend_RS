use super::types::{ApiErr, ApiOk};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

// 根据 ID 获取管制员详情（cid 或 session_id）
pub async fn controller_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiErr>)> {
    let data = state.cache.read().await.clone();
    if let Some(c) = data
        .controllers
        .into_iter()
        .find(|c| c.base.cid == id || c.base.session_id == id)
    {
        Ok(Json(ApiOk { code: 200, data: c }))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ApiErr {
                code: 404,
                error: "Not found".to_string(),
            }),
        ))
    }
}
