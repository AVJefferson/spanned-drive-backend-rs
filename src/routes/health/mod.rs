mod client;
mod is_running;

use crate::state::AppState;

use axum::Router;
use std::sync::Arc;

async fn request_handler() -> () {
    ()
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/is_running", is_running::routes(app_state.clone()))
        .nest("/client", client::routes(app_state.clone()))
}
