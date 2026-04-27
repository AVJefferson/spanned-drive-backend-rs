use crate::state::AppState;

use axum::{Router, routing::get};
use std::sync::Arc;

async fn request_handler() -> () {
    ()
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(request_handler))
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use axum_test::{TestResponse, TestServer};
    use dashmap::DashMap;

    use crate::clients::ExternalClients;

    use super::*;

    #[tokio::test]
    async fn is_running() {
        let clients = ExternalClients::empty();

        let app_state = Arc::new(AppState {
            clients: clients,
            auth_tokens: DashMap::new(),
        });

        let app = routes(app_state.clone());

        let server = TestServer::new(app);
        let response: TestResponse = server.get("/").await;
        response.assert_status_ok();
    }
}
