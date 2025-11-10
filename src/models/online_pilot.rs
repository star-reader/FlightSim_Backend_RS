use serde::{Deserialize, Serialize};
use super::{BaseUser, FlightPlan};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OnlinePilot {
    // 基础信息
    #[serde(flatten)]
    pub base: BaseUser,
    // 动态的，? -> any
    // TODO 把它和管制员的结合分离
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: i64,
    pub groundspeed: i64,
    pub transponder: i64,
    pub heading: i64,
    pub bank: i64,
    pub pitch: i64,
    pub flight_plan: Option<FlightPlan>,
}