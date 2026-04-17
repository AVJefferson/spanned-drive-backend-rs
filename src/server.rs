use crate::{
    allowed_clients, clients::Clients, constants, logger::Logger, routes, state::AppState,
};

use axum;
use dashmap::DashMap;
use std::{env, sync::Arc};

pub async fn start_server() {
    Logger::new(constants::LOG_CHANNEL_SIZE).expect("Failed to initialize logger");

    let clients = Clients::new_from_env_variables().await;
    let auth_tokens = DashMap::new();

    if let Err(e) = allowed_clients::load_into_map(&auth_tokens).await {
        log::error!("allowed_clients: failed to load directory: {}", e);
    }

    let app_state = Arc::new(AppState {
        clients,
        auth_tokens,
    });

    let port =
        env::var("SERVER_PORT").unwrap_or_else(|_| constants::DEFAULT_SERVER_PORT.to_string());
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| constants::DEFAULT_SERVER_HOST.to_string());
    let addr = format!("{}:{}", host, port);

    log::info!(
        "Starting server... at http://{}. TraceID: {}",
        addr,
        constants::APP_TRACE_ID.as_str()
    );

    let app = routes::routes(app_state);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server crashed unexpectedly");
}
