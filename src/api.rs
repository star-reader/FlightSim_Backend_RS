use axum::{routing::get, Router};

use crate::state::AppState;

pub mod types;
pub mod online_list;
pub mod pilot_detail;
pub mod controller_detail;
pub mod flight_track;
pub mod airport_traffic;

// 挂载所有受保护的接口路由
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/online-list", get(online_list::online_list))
        .route("/pilot/detail/:id", get(pilot_detail::pilot_detail))
        .route("/pilot/request-track/:id", get(flight_track::pilot_request_track))
        .route("/controller/detail/:id", get(controller_detail::controller_detail))
        .route("/pilot/by-callsign/:callsign", get(pilot_detail::pilot_by_callsign))
        .route("/airport/traffic/:icao", get(airport_traffic::airport_traffic))
}