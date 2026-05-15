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
    let snapshot = state.cache.load();
    let traffic = snapshot.airport_traffic(&icao);

    Json(ApiOk {
        code: 200,
        data: AirportTraffic {
            icao,
            departures: traffic.departures,
            arrivals: traffic.arrivals,
        },
    })
}
