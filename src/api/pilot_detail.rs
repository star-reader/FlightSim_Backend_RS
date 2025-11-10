use super::types::{ApiErr, ApiOk};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};


pub async fn pilot_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiErr>)> {
    let data = state.cache.read().await.clone();
    if let Some(p) = data
        .flights
        .into_iter()
        .find(|p| p.base.cid == id || p.base.session_id == id)
    {
        Ok(Json(ApiOk { code: 200, data: p }))
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

pub async fn pilot_by_callsign(
    State(state): State<AppState>,
    Path(callsign): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiErr>)> {
    let data = state.cache.read().await.clone();
    if let Some(p) = data
        .flights
        .into_iter()
        .find(|p| p.base.callsign.eq_ignore_ascii_case(&callsign))
    {
        Ok(Json(ApiOk { code: 200, data: p }))
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
