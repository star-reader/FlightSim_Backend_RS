use super::types::{ApiErr, ApiOk, TrackPosition, TrackResponse};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

// 根据 ID 获取航班轨迹
pub async fn pilot_request_track(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiErr>)> {
    let snapshot = state.cache.load();
    // 当前位置 + flight_plan
    if let Some(p) = snapshot.pilot_by_id(&id) {
        let resp = TrackResponse {
            position: TrackPosition {
                latitude: p.latitude,
                longitude: p.longitude,
                altitude: p.altitude as f64,
                heading: p.heading as f64,
            },
            flight_plan: p.flight_plan.clone(),
        };
        Ok(Json(ApiOk {
            code: 200,
            data: resp,
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
