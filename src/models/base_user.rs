use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BaseUser {
    pub cid: String,
    pub name: String,
    pub callsign: String,
    pub server: String,
    pub session_id: String,
    pub logon_time: String,
}