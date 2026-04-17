use crate::{error::AppError, middlewares::authz::TokenRouterState, state::AppState};

use axum::{Router, extract::State, response::Json, routing::post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
struct RefreshTokenPayload {
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AccessTokenPayload {
    pub refresh_token: String,
}

async fn request_handler_refresh_token(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let token_response = google_client
        .fetch_refresh_token(payload.code, payload.code_verifier, payload.redirect_uri)
        .await?;

    Ok(Json(token_response))
}

async fn request_handler_access_token(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<AccessTokenPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let token_response = google_client
        .fetch_access_token(payload.refresh_token)
        .await?;

    Ok(Json(token_response))
}

pub fn routes(state: TokenRouterState) -> Router {
    Router::new()
        .route("/refresh_token", post(request_handler_refresh_token))
        .route("/access_token", post(request_handler_access_token))
        .with_state(state)
}
