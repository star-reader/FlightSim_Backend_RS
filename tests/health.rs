use axum::{Router, routing::get};

#[tokio::test]
async fn health_ok() {
    // 后面的在cache里面
    async fn health() -> &'static str { "ok" }
    let app = Router::new().route("/api/health", get(health));
    use tower::ServiceExt;
    let res = app.oneshot(axum::http::Request::builder().uri("/api/health").body(axum::body::Body::empty()).unwrap()).await.unwrap();
    assert!(res.status().is_success());
}