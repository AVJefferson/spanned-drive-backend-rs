use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

// Define our custom error type
#[derive(Debug)]
pub enum AppError {
    InvalidPayload(serde_json::Error),
    Anyhow(anyhow::Error),
    ClientNotAvailable,
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Anyhow(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::InvalidPayload(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::ClientNotAvailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "External client is not configured or available".to_string(),
            ),
        };

        (status, error_message).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>`
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Anyhow(err.into())
    }
}
