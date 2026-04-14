mod google_web;

use crate::state::AppState;

use axum::Router;
use std::sync::Arc;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new().nest("/google-web", google_web::routes(app_state.clone()))
}
