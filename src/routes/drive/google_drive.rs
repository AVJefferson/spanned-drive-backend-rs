use crate::{error::AppError, external_systems::google::config, state::AppState};

use axum::{
    Router,
    extract::{Multipart, State},
    http::{HeaderValue, StatusCode, header},
    response::{Json, Response},
    routing::post,
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
    Json(payload): Json<config::AccessTokenPayload>,
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
    Json(payload): Json<config::AccessTokenPayload>,
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

// ---- Drive operation proxy handlers ----

async fn request_handler_drive_about(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<config::AccessTokenPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let about = google_client
        .drive_about(payload.access_token)
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    Ok(Json(about))
}

#[derive(serde::Deserialize)]
struct ListChildrenPayload {
    pub access_token: String,
    pub parent_id: String,
}

async fn request_handler_list_children(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<ListChildrenPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let children = google_client
        .list_drive_children(payload.access_token, payload.parent_id)
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    Ok(Json(children))
}

#[derive(serde::Deserialize)]
struct FileMetadataPayload {
    pub access_token: String,
    pub file_id: String,
}

async fn request_handler_file_metadata(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<FileMetadataPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let metadata = google_client
        .get_file_metadata_rich(payload.access_token, payload.file_id)
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    Ok(Json(metadata))
}

#[derive(serde::Deserialize)]
struct CreateFolderPayload {
    pub access_token: String,
    pub name: String,
    pub parent_id: String,
}

async fn request_handler_create_folder(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateFolderPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let metadata = google_client
        .create_drive_folder(payload.access_token, payload.name, payload.parent_id)
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    Ok(Json(metadata))
}

/// Accepts `multipart/form-data` with fields:
/// - `access_token` (text)
/// - `parent_id` (text)
/// - `file` (binary, filename and content-type forwarded to Google)
async fn request_handler_upload_file(
    State(app_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let mut access_token = String::new();
    let mut parent_id = String::new();
    let mut file_name = String::new();
    let mut mime_type = String::from("application/octet-stream");
    let mut data: Vec<u8> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Multipart parse error: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        match field_name.as_str() {
            "access_token" => {
                access_token = field
                    .text()
                    .await
                    .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;
            }
            "parent_id" => {
                parent_id = field
                    .text()
                    .await
                    .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;
            }
            "file" => {
                if let Some(fname) = field.file_name() {
                    file_name = fname.to_string();
                }
                if let Some(ct) = field.content_type() {
                    mime_type = ct.to_string();
                }
                data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::ExternalServiceError(e.to_string()))?
                    .to_vec();
            }
            _ => {}
        }
    }

    if access_token.is_empty() || parent_id.is_empty() || data.is_empty() {
        return Err(AppError::ExternalServiceError(
            "Missing required upload fields".to_string(),
        ));
    }

    let metadata = google_client
        .upload_drive_file(access_token, file_name, mime_type, data, parent_id)
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    Ok(Json(metadata))
}

#[derive(serde::Deserialize)]
struct DeleteItemPayload {
    pub access_token: String,
    pub file_id: String,
}

async fn request_handler_delete_item(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<DeleteItemPayload>,
) -> Result<Json<bool>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    google_client
        .delete_drive_item(payload.access_token, payload.file_id)
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    Ok(Json(true))
}

#[derive(serde::Deserialize)]
struct CopyItemPayload {
    pub access_token: String,
    pub file_id: String,
    pub parent_id: String,
    pub name: Option<String>,
}

async fn request_handler_copy_item(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CopyItemPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let metadata = google_client
        .copy_drive_item(
            payload.access_token,
            payload.file_id,
            payload.parent_id,
            payload.name,
        )
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    Ok(Json(metadata))
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/get_appdata_file", post(request_handler_get_appdata_file))
        .route(
            "/get_logical_folders",
            post(request_handler_get_logical_folders),
        )
        .route(
            "/set_logical_folder",
            post(request_handler_set_new_logical_folder),
        )
        .route(
            "/get_secondary_drives",
            post(request_handler_get_secondary_drives),
        )
        .route(
            "/set_secondary_drive",
            post(request_handler_set_new_secondary_drive),
        )
        .route(
            "/get_primary_file_id",
            post(request_handler_get_primary_file_id),
        )
        .route("/set_as_primary", post(request_handler_set_as_primary))
        // Drive operation proxies
        .route("/drive_about", post(request_handler_drive_about))
        .route("/list_children", post(request_handler_list_children))
        .route("/file_metadata", post(request_handler_file_metadata))
        .route("/create_folder", post(request_handler_create_folder))
        .route("/upload_file", post(request_handler_upload_file))
        .route("/delete_item", post(request_handler_delete_item))
        .route("/copy_item", post(request_handler_copy_item))
        // appData by-name (browser browser routing for state files)
        .route(
            "/get_appdata_file_by_name",
            post(request_handler_get_appdata_file_by_name),
        )
        .route(
            "/set_appdata_file_by_name",
            post(request_handler_set_appdata_file_by_name),
        )
        // binary file download
        .route("/download_file", post(request_handler_download_file))
        .with_state(app_state)
}

// ---- appData by-name route handlers ----

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

// ---- Binary file download ----

#[derive(serde::Deserialize)]
struct DownloadFilePayload {
    pub access_token: String,
    pub file_id: String,
}

async fn request_handler_download_file(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<DownloadFilePayload>,
) -> Result<Response, AppError> {
    let google_client = app_state
        .clients
        .google
        .as_ref()
        .ok_or(AppError::ClientNotAvailable)?;

    let bytes = google_client
        .download_drive_file_bytes(payload.access_token, payload.file_id)
        .await
        .map_err(|e| AppError::ExternalServiceError(e.to_string()))?;

    let mut response = Response::new(axum::body::Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );

    Ok(response)
}
