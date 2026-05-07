use crate::{error::AppError, external_systems::google::config::AccessTokenPayload, state::AppState};

use axum::{Router, extract::State, response::Json, routing::post};
use std::sync::Arc;

// ---- appData by-ID handlers ----

async fn request_handler_get_primary_file_id(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<AccessTokenPayload>,
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
    Json(payload): Json<AccessTokenPayload>,
) -> Result<Json<bool>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let primary_file_id = google_client
        .get_primary_file_id(payload.access_token.clone())
        .await
        .map_err(|_| AppError::ExternalServiceError("Google Drive Call Failed".to_string()))?;

    if !primary_file_id.is_empty() {
        return Ok(Json(true));
    }

    let _ = google_client
        .set_is_primary(payload.access_token)
        .await
        .map_err(|_| AppError::ExternalServiceError("Google Drive Call Failed".to_string()))?;

    Ok(Json(true))
}

async fn request_handler_get_secondary_drives(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<AccessTokenPayload>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let secondary_drives = google_client
        .get_secondary_drives(payload.access_token)
        .await
        .map_err(|_| AppError::ExternalServiceError("Google Drive Call Failed".to_string()))?;

    Ok(Json(secondary_drives))
}

#[derive(serde::Deserialize)]
struct GetAppdataFilePayload {
    pub access_token: String,
    pub file_id: String,
}

async fn request_handler_get_appdata_file(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<GetAppdataFilePayload>,
) -> Result<Json<String>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let appdata_file = google_client
        .get_appdata_file(payload.access_token, payload.file_id)
        .await
        .map_err(|_| AppError::ExternalServiceError("Google Drive Call Failed".to_string()))?;

    Ok(Json(appdata_file))
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
    Json(payload): Json<AccessTokenPayload>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let logical_folders = google_client
        .get_logical_folders(payload.access_token)
        .await
        .map_err(|_| AppError::ExternalServiceError("Google Drive Call Failed".to_string()))?;

    Ok(Json(logical_folders))
}

#[derive(serde::Deserialize)]
struct SetLogicalFolderPayload {
    pub access_token: String,
    pub new_logical_folder_name: String,
    /// Each tuple is `[provider, email, root_folder_id?]`; arbitrary length
    /// strings are forwarded verbatim into the stored appData file.
    pub drives: Vec<Vec<String>>,
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
        .set_logical_folder(
            payload.access_token,
            payload.new_logical_folder_name,
            payload.drives,
        )
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Google Drive Call Failed: {}", e)))?;

    Ok(Json(true))
}

// ---- appData by-name handlers ----

#[derive(serde::Deserialize)]
struct GetAppdataFileByNamePayload {
    pub access_token: String,
    pub file_name: String,
}

async fn request_handler_get_appdata_file_by_name(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<GetAppdataFileByNamePayload>,
) -> Result<Json<Option<String>>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let content = google_client
        .get_appdata_file_by_name(payload.access_token, payload.file_name)
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    Ok(Json(content))
}

#[derive(serde::Deserialize)]
struct SetAppdataFileByNamePayload {
    pub access_token: String,
    pub file_name: String,
    pub content: String,
}

async fn request_handler_set_appdata_file_by_name(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<SetAppdataFileByNamePayload>,
) -> Result<Json<bool>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    google_client
        .set_appdata_file_by_name(payload.access_token, payload.file_name, payload.content)
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    Ok(Json(true))
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/get_appdata_file", post(request_handler_get_appdata_file))
        .route(
            "/get_primary_file_id",
            post(request_handler_get_primary_file_id),
        )
        .route("/set_as_primary", post(request_handler_set_as_primary))
        .route(
            "/get_secondary_drives",
            post(request_handler_get_secondary_drives),
        )
        .route(
            "/set_secondary_drive",
            post(request_handler_set_new_secondary_drive),
        )
        .route(
            "/get_logical_folders",
            post(request_handler_get_logical_folders),
        )
        .route(
            "/set_logical_folder",
            post(request_handler_set_new_logical_folder),
        )
        .route(
            "/get_appdata_file_by_name",
            post(request_handler_get_appdata_file_by_name),
        )
        .route(
            "/set_appdata_file_by_name",
            post(request_handler_set_appdata_file_by_name),
        )
        .with_state(app_state)
}
