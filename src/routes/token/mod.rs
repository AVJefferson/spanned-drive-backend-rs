mod google_drive;

use crate::state::AppState;

use axum::Router;
use std::sync::Arc;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new().nest("/google-drive", google_drive::routes(app_state))
}
