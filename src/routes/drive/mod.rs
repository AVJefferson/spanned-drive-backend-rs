mod appdata;
mod proxy;

use crate::state::AppState;

use axum::Router;
use std::sync::Arc;

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new().nest(
        "/google-drive",
        appdata::routes(app_state.clone()).merge(proxy::routes(app_state)),
    )
}
