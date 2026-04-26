use crate::state::AppState;

use axum::{Router, extract::State, routing::get};
use std::sync::Arc;

async fn request_handler(State(app_state): State<Arc<AppState>>) -> &'static str {
    static OPENAPI_JSON: &str = include_str!("../../openapi.json");
    OPENAPI_JSON
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(request_handler))
        .with_state(app_state)
}
