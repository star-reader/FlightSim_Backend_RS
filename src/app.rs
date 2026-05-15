use axum::{Router, routing::get};

use crate::{api, auth, middleware, state::AppState};

pub async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"code":200, "data": {"status":"ok"}}))
}

pub fn build_app(state: AppState) -> Router {
    let protected = api::routes()
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::handler::authorize,
        ))
        .layer(middleware::ratelimit::layer());

    Router::new()
        .route("/map/v2/health", get(health))
        .nest("/map/v2", protected)
        .with_state(state)
        .layer(middleware::cors::cors_layer())
}
