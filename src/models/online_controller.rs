use serde::{Deserialize, Serialize};
use super::BaseUser;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OnlineController {
    // 基础信息
    #[serde(flatten)]
    pub base: BaseUser,
    // 额外字段
    pub frequency: String,
    pub facility: i32,
    pub rating: i32,
    pub visual_range: i32,
    pub text_atis: Vec<String>,
}

impl OnlineController {
    pub fn cid(&self) -> &str { &self.base.cid }
    pub fn session_id(&self) -> &str { &self.base.session_id }
}