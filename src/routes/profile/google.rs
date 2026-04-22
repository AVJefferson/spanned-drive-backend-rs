use crate::{error::AppError, state::AppState};

use axum::{Router, extract::State, response::Json, routing::post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
struct ProfilePayload {
    pub sub: String,

    pub email: String,
    pub email_verified: bool,

    pub name: String,
    pub picture: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AccessTokenPayload {
    pub access_token: String,
}

async fn request_handler_user_info(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<AccessTokenPayload>,
) -> Result<Json<ProfilePayload>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let response = google_client
        .fetch_user_info(payload.access_token)
        .await
        .map_err(|_| AppError::ExternalServiceError(format!("Google")))?;

    print!("Google user info response: {:?}", response);

    Ok(Json(ProfilePayload {
        sub: response["sub"].as_str().unwrap_or_default().to_string(),
        email: response["email"].as_str().unwrap_or_default().to_string(),
        email_verified: response["email_verified"].as_bool().unwrap_or_default(),
        name: response["name"].as_str().unwrap_or_default().to_string(),
        picture: response["picture"].as_str().unwrap_or_default().to_string(),
    }))
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(request_handler_user_info))
        .with_state(app_state)
}
