use crate::state::AppState;

use axum::{Router, routing::post};
use std::sync::Arc;

async fn request_handler() -> () {
    ()
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(request_handler))
        .with_state(app_state)
}
