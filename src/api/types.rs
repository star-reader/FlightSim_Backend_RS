use serde::{Deserialize, Serialize};
use crate::models::FlightPlan;

#[derive(Serialize, Deserialize)]
pub struct ApiOk<T> {
    pub code: u16,
    pub data: T,
}

#[derive(Serialize, Deserialize)]
pub struct ApiErr {
    pub code: u16,
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct TrackPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub heading: f64,
}

#[derive(Serialize, Deserialize)]
pub struct TrackResponse {
    pub position: TrackPosition,
    pub flight_plan: Option<FlightPlan>,
}

#[derive(Serialize, Deserialize)]
pub struct AirportTraffic {
    pub icao: String,
    pub departures: u32,
    pub arrivals: u32,
}