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
    let snapshot = state.cache.load();
    if let Some(p) = snapshot.pilot_by_id(&id) {
        Ok(Json(ApiOk {
            code: 200,
            data: p.clone(),
        }))
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
    let snapshot = state.cache.load();
    if let Some(p) = snapshot.pilot_by_callsign(&callsign) {
        Ok(Json(ApiOk {
            code: 200,
            data: p.clone(),
        }))
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
