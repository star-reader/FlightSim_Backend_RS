use super::types::{AirportTraffic, ApiOk};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

pub async fn airport_traffic(
    State(state): State<AppState>,
    Path(icao): Path<String>,
) -> impl IntoResponse {
    let data = state.cache.read().await.clone();
    let mut dep = 0u32;
    let mut arr = 0u32;
    // inbound和outbound分开算
    // bugfix:// Jerry Some(fp)分配问题
    for p in data.flights.into_iter() {
        if let Some(fp) = &p.flight_plan {
            if fp.departure.eq_ignore_ascii_case(&icao) {
                dep += 1;
            }
            if fp.arrival.eq_ignore_ascii_case(&icao) {
                arr += 1;
            }
        }
    }
    Json(ApiOk {
        code: 200,
        data: AirportTraffic {
            icao,
            departures: dep,
            arrivals: arr,
        },
    })
}
