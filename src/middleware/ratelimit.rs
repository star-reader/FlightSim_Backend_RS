use tower_governor::governor::GovernorConfigBuilder;

pub fn governor_config() -> impl Clone {
    GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(100)
        .finish()
        .expect("构建 GovernorConfig 失败")
}