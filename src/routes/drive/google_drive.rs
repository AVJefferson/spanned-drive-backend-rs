use crate::{constants, error::AppError, external_systems::google::config, state::AppState};

use axum::{
    Router,
    extract::State,
    response::Json,
    routing::{get, post, put},
};
use std::sync::Arc;

async fn request_handler_get_primary_file_id(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<config::AccessTokenPayload>,
) -> Result<Json<String>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let primary_file_id = google_client
        .get_primary_file_id(payload.access_token)
        .await?;
    Ok(Json(primary_file_id))
}

async fn request_handler_set_as_primary(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<config::AccessTokenPayload>,
) -> Result<Json<bool>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let primary_file_id = google_client
        .get_primary_file_id(payload.access_token.clone())
        .await
        .map_err(|_| AppError::ExternalServiceError(format!("Google Drive Call Failed")))?;

    if primary_file_id != "" {
        return Ok(Json(true));
    }

    let _ = google_client
        .set_is_primary(payload.access_token)
        .await
        .map_err(|_| AppError::ExternalServiceError(format!("Google Drive Call Failed")))?;

    Ok(Json(true))
}

async fn request_handler_get_secondary_drives(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<config::AccessTokenPayload>,
) -> Result<Json<Vec<String>>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let secondary_drives = google_client
        .get_secondary_drives(payload.access_token)
        .await
        .map_err(|_| AppError::ExternalServiceError(format!("Google Drive Call Failed")))?;

    Ok(Json(secondary_drives))
}

#[derive(serde::Deserialize)]
struct SetSecondaryDrivePayload {
    pub access_token: String,
    pub new_secondary_drive_email: String,
    pub drive_provider: String,
}

async fn request_handler_set_new_secondary_drive(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<SetSecondaryDrivePayload>,
) -> Result<Json<bool>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let _ = google_client
        .set_secondary_drive(
            payload.access_token,
            payload.drive_provider,
            payload.new_secondary_drive_email,
        )
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Google Drive Call Failed: {}", e)))?;

    Ok(Json(true))
}

async fn request_handler_get_logical_folders(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<config::AccessTokenPayload>,
) -> Result<Json<Vec<String>>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let logical_folders = google_client
        .get_logical_folders(payload.access_token)
        .await
        .map_err(|_| AppError::ExternalServiceError(format!("Google Drive Call Failed")))?;

    Ok(Json(logical_folders))
}

#[derive(serde::Deserialize)]
struct SetLogicalFolderPayload {
    pub access_token: String,
    pub new_logical_folder_name: String,
    pub drives: Vec<(String, String)>,
}

async fn request_handler_set_new_logical_folder(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<SetLogicalFolderPayload>,
) -> Result<Json<bool>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let _ = google_client
        .set_logical_folder(payload.access_token, payload.new_logical_folder_name, payload.drives)
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Google Drive Call Failed: {}", e)))?;

    Ok(Json(true))
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/logical_folders", get(request_handler_get_logical_folders))
        .route(
            "/logical_folder",
            post(request_handler_set_new_logical_folder),
        )
        .route(
            "/secondary_drives",
            get(request_handler_get_secondary_drives),
        )
        .route(
            "/secondary_drive",
            post(request_handler_set_new_secondary_drive),
        )
        .route("/primary_file_id", get(request_handler_get_primary_file_id))
        .route("/set_as_primary", post(request_handler_set_as_primary))
        .with_state(app_state)
}
