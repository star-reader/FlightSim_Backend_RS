use std::sync::Arc;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_governor::key_extractor::PeerIpKeyExtractor;
use governor::middleware::NoOpMiddleware;

pub fn layer() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware> {
    let cfg = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(100)
        .finish()
        .expect("构建 GovernorConfig 失败");
    GovernorLayer { config: Arc::new(cfg) }
}