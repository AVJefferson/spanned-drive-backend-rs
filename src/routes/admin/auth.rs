use crate::state::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct TokenPayload {
    #[validate(length(min = 1, message = "Token cannot be empty"))]
    pub token: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct PermissionsPayload {
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct TokenPath {
    #[validate(length(min = 1, message = "Token cannot be empty"))]
    token: String,
}

async fn add_token(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<TokenPayload>,
) -> Result<(StatusCode, &'static str), (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        return Err((StatusCode::BAD_REQUEST, e.to_string()));
    }
    app_state.auth_tokens.insert(payload.token, payload.permissions);
    Ok((StatusCode::CREATED, "Token created"))
}

async fn delete_token(
    State(app_state): State<Arc<AppState>>,
    Path(token): Path<TokenPath>,
) -> Result<(StatusCode, &'static str), (StatusCode, String)> {
    if let Err(e) = token.validate() {
        return Err((StatusCode::BAD_REQUEST, e.to_string()));
    }
    if app_state.auth_tokens.remove(&token.token).is_some() {
        Ok((StatusCode::OK, "Token deleted"))
    } else {
        Err((StatusCode::NOT_FOUND, "Token not found".to_string()))
    }
}

async fn update_token(
    State(app_state): State<Arc<AppState>>,
    Path(token): Path<TokenPath>,
    Json(payload): Json<PermissionsPayload>,
) -> Result<(StatusCode, &'static str), (StatusCode, String)> {
    if let Err(e) = token.validate() {
        return Err((StatusCode::BAD_REQUEST, e.to_string()));
    }
    if let Err(e) = payload.validate() {
        return Err((StatusCode::BAD_REQUEST, e.to_string()));
    }
    if let Some(mut permissions) = app_state.auth_tokens.get_mut(&token.token) {
        *permissions = payload.permissions;
        Ok((StatusCode::OK, "Token updated"))
    } else {
        Err((StatusCode::NOT_FOUND, "Token not found".to_string()))
    }
}

async fn list_tokens(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<TokenPayload>>, (StatusCode, String)> {
    let tokens = app_state
        .auth_tokens
        .iter()
        .map(|entry| TokenPayload {
            token: entry.key().clone(),
            permissions: entry.value().clone(),
        })
        .collect();
    Ok(Json(tokens))
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/tokens", post(add_token))
        .route("/tokens/{token}", delete(delete_token))
        .route("/tokens/{token}", put(update_token))
        .route("/tokens", get(list_tokens))
        .with_state(app_state)
}

#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    use dashmap::DashMap;

    use crate::clients::Clients;
    use crate::test_utils::timeout;

    use super::*;

    // Setup Server before each test
    async fn get_test_server() -> (TestServer, Arc<AppState>) {
        let clients = Clients::empty();
        let auth_tokens = DashMap::new();
        auth_tokens.insert("test".to_string(), vec!["admin".to_string()]);
        let app_state = Arc::new(AppState {
            clients: clients,
            auth_tokens: auth_tokens,
        });

        let app = routes(app_state.clone());
        let server = TestServer::new(app);

        (server, app_state)
    }

    #[tokio::test]
    async fn add_token() {
        timeout::with_default(async {
            let (server, app_state) = get_test_server().await;

            let response = server
                .post("/tokens")
                .json(&TokenPayload {
                    token: "test_new".to_string(),
                    permissions: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                })
                .await;

            assert_eq!(response.status_code(), StatusCode::CREATED);
            assert!(app_state.auth_tokens.contains_key("test_new"));
            assert!(
                app_state
                    .auth_tokens
                    .get("test_new")
                    .unwrap()
                    .contains(&"a".to_string())
            );

            assert!(
                app_state
                    .auth_tokens
                    .get("test_new")
                    .unwrap()
                    .contains(&"b".to_string())
            );

            assert!(
                app_state
                    .auth_tokens
                    .get("test_new")
                    .unwrap()
                    .contains(&"c".to_string())
            );
        })
        .await;
    }

    #[tokio::test]
    async fn delete_token() {
        timeout::with_default(async {
            let (server, app_state) = get_test_server().await;

            let response = server.delete("/tokens/test").await;

            response.assert_status_ok();
            assert!(!app_state.auth_tokens.contains_key("test"));
        })
        .await;
    }

    #[tokio::test]
    async fn update_token() {
        timeout::with_default(async {
            let (server, app_state) = get_test_server().await;

            let response = server
                .put("/tokens/test")
                .json(&PermissionsPayload {
                    permissions: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                })
                .await;

            response.assert_status_ok();
            assert!(
                app_state
                    .auth_tokens
                    .get("test")
                    .unwrap()
                    .contains(&"a".to_string())
            );

            assert!(
                app_state
                    .auth_tokens
                    .get("test")
                    .unwrap()
                    .contains(&"b".to_string())
            );

            assert!(
                app_state
                    .auth_tokens
                    .get("test")
                    .unwrap()
                    .contains(&"c".to_string())
            );

            assert!(
                !app_state
                    .auth_tokens
                    .get("test")
                    .unwrap()
                    .contains(&"admin".to_string())
            );
        })
        .await;
    }

    #[tokio::test]
    async fn get_tokens() {
        timeout::with_default(async {
            let (server, _app_state) = get_test_server().await;

            let response = server.get("/tokens").await;

            response.assert_status_ok();
            let tokens: Vec<TokenPayload> = response.json();

            assert!(!tokens.is_empty());
            assert!(tokens.iter().any(|t| t.token == "test"));
        })
        .await;
    }

    #[tokio::test]
    async fn add_token_empty_token() {
        timeout::with_default(async {
            let (server, _app_state) = get_test_server().await;

            let response = server
                .post("/tokens")
                .json(&TokenPayload {
                    token: "".to_string(),
                    permissions: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                })
                .await;

            assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
        })
        .await;
    }
}
