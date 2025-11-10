use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};

pub fn governor_config() -> GovernorConfig {
    GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(100)
        .finish()
        .expect("构建 GovernorConfig 失败")
}