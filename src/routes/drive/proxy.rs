use crate::{error::AppError, external_systems::google::config::AccessTokenPayload, state::AppState};

use axum::{
    Router,
    extract::{Multipart, State},
    http::{HeaderValue, StatusCode, header},
    response::{Json, Response},
    routing::post,
};
use std::sync::Arc;

async fn request_handler_drive_about(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<AccessTokenPayload>,
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

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/drive_about", post(request_handler_drive_about))
        .route("/list_children", post(request_handler_list_children))
        .route("/file_metadata", post(request_handler_file_metadata))
        .route("/create_folder", post(request_handler_create_folder))
        .route("/upload_file", post(request_handler_upload_file))
        .route("/delete_item", post(request_handler_delete_item))
        .route("/copy_item", post(request_handler_copy_item))
        .route("/download_file", post(request_handler_download_file))
        .with_state(app_state)
}
