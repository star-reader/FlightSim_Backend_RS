use serde::{Deserialize, Serialize};
use super::{OnlineController, OnlinePilot};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OnlineData {
    pub controllers: Vec<OnlineController>,
    pub flights: Vec<OnlinePilot>,
    pub atis: Vec<serde_json::Value>,
}