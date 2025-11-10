use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use crate::state::AppState;
use super::types::{ApiOk, ApiErr, TrackPosition, TrackResponse};

// 根据 ID 获取航班轨迹
pub async fn pilot_request_track(State(state): State<AppState>, Path(id): Path<String>) -> Result<impl IntoResponse, (StatusCode, Json<ApiErr>)> {
    let data = state.cache.read().await.clone();
    // 当前位置 + flight_plan
    if let Some(p) = data.flights.into_iter().find(|p| p.base.cid == id || p.base.session_id == id) {
        let resp = TrackResponse {
            position: TrackPosition {
                latitude: p.latitude,
                longitude: p.longitude,
                altitude: p.altitude as f64,
                heading: p.heading as f64,
            },
            flight_plan: p.flight_plan,
        };
        Ok(Json(ApiOk { code: 200, data: resp }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiErr { code: 404, error: "Not found".to_string() })))
    }
}