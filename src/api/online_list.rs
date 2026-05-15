use axum::{
    Json,
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

use super::types::{ApiErr, ApiOk};
use crate::state::AppState;

pub async fn online_list(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.cache.load_full();
    match serde_json::to_vec(&ApiOk {
        code: 200,
        data: &snapshot.data,
    }) {
        Ok(body) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .expect("valid JSON response"),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiErr {
                code: 500,
                error: "JSON serialization failed".to_string(),
            }),
        )
            .into_response(),
    }
}
