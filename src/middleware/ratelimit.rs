use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::PeerIpKeyExtractor;

const PUBLIC_PER_SECOND: u64 = 20;
const PUBLIC_BURST_SIZE: u32 = 60;
const INTERNAL_PER_SECOND: u64 = 100;
const INTERNAL_BURST_SIZE: u32 = 300;

pub fn layer() -> GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware> {
    let (per_second, burst_size) = rate_limit_profile();
    let cfg = GovernorConfigBuilder::default()
        .per_second(per_second)
        .burst_size(burst_size)
        .finish()
        .expect("构建 GovernorConfig 失败");
    GovernorLayer {
        config: Arc::new(cfg),
    }
}

fn rate_limit_profile() -> (u64, u32) {
    match std::env::var("RATE_LIMIT_PROFILE")
        .unwrap_or_else(|_| "public".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "internal" => (INTERNAL_PER_SECOND, INTERNAL_BURST_SIZE),
        _ => (PUBLIC_PER_SECOND, PUBLIC_BURST_SIZE),
    }
}
