use tower_governor::GovernorConfig;
use tower_governor::GovernorConfigBuilder;

pub fn governor_config() -> GovernorConfig {
    GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(200)
        .finish()
        .expect("构建 GovernorConfig 失败")
}