use crate::state::AppState;
use axum::{
    body::Body,
    extract::{FromRef, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::headers::{Authorization, HeaderMapExt, authorization::Bearer};
use std::sync::Arc;

#[derive(Clone)]
pub struct TokenRouterState {
    pub app_state: Arc<AppState>,
    pub required_permissions: Arc<[String]>,
}

impl FromRef<TokenRouterState> for Arc<AppState> {
    fn from_ref(state: &TokenRouterState) -> Self {
        state.app_state.clone()
    }
}

pub async fn authz_middleware(
    State(state): State<TokenRouterState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let auth_header = request.headers().typed_get::<Authorization<Bearer>>();

    let token = match auth_header {
        Some(Authorization(bearer)) => bearer.token().to_string(),
        None => {
            log::trace!("Missing authorization header");
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Missing authorization header"))
                .unwrap();
        }
    };

    let is_authorized = match state.app_state.auth_tokens.get(&token) {
        Some(user_permissions) => state
            .required_permissions
            .iter()
            .any(|p| user_permissions.contains(p)),
        None => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Invalid token"))
                .unwrap();
        }
    };

    if is_authorized {
        next.run(request).await
    } else {
        log::trace!("Insufficient permissions");
        Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Insufficient permissions"))
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use axum::{middleware::from_fn_with_state, routing::get, Router};
    use axum_test::TestServer;
    use dashmap::DashMap;

    use crate::{clients::Clients, middlewares::authz::TokenRouterState};
    use crate::test_utils::timeout;

    use super::*;

    #[tokio::test]
    async fn authz() {
        timeout::with_default(async {
            let clients = Clients::empty();
            let auth_tokens = DashMap::new();
            auth_tokens.insert("test".to_string(), vec!["admin".to_string()]);
            auth_tokens.insert("test_no_permission".to_string(), vec![]);
            auth_tokens.insert("test_min_permission".to_string(), vec!["user".to_string()]);
            let app_state = Arc::new(AppState {
                clients: clients,
                auth_tokens: auth_tokens,
            });

            let token_state = TokenRouterState {
                app_state: app_state.clone(),
                required_permissions: Arc::from(["admin".to_string()]),
            };

            let app = Router::new()
                .route("/", get(|| async { (StatusCode::OK, "OK") }))
                .with_state(token_state.clone())
                .layer(from_fn_with_state(token_state.clone(), authz_middleware));
            let server = TestServer::new(app);

            app_state
                .auth_tokens
                .insert("test_no_permission".to_string(), vec![]);
            app_state
                .auth_tokens
                .insert("test_min_permission".to_string(), vec!["user".to_string()]);

            assert!(app_state.auth_tokens.contains_key("test"));
            assert!(
                app_state
                    .auth_tokens
                    .get("test")
                    .unwrap()
                    .contains(&"admin".to_string())
            );
            assert!(app_state.auth_tokens.contains_key("test_no_permission"));
            assert!(
                app_state
                    .auth_tokens
                    .get("test_no_permission")
                    .unwrap()
                    .is_empty()
            );
            assert!(app_state.auth_tokens.contains_key("test_min_permission"));
            assert!(
                app_state
                    .auth_tokens
                    .get("test_min_permission")
                    .unwrap()
                    .contains(&"user".to_string())
            );

            let response_no_auth = server.get("/").await;
            response_no_auth.assert_status_unauthorized();
            let response_no_auth_message: String = response_no_auth.text();
            assert_eq!(response_no_auth_message, "Missing authorization header");

            let response_wrong_auth = server.get("/").authorization_bearer("wrong").await;
            response_wrong_auth.assert_status_unauthorized();
            let response_message_wrong_auth: String = response_wrong_auth.text();
            assert_eq!(response_message_wrong_auth, "Invalid token");

            let response_no_perm = server
                .get("/")
                .authorization_bearer("test_no_permission")
                .await;
            response_no_perm.assert_status_forbidden();
            let response_message_no_perm: String = response_no_perm.text();
            assert_eq!(response_message_no_perm, "Insufficient permissions");

            let response_min_perm = server
                .get("/")
                .authorization_bearer("test_min_permission")
                .await;
            response_min_perm.assert_status_forbidden();
            let response_message_min_perm: String = response_min_perm.text();
            assert_eq!(response_message_min_perm, "Insufficient permissions");

            let response_auth = server.get("/").authorization_bearer("test").await;
            response_auth.assert_status_ok();
        })
        .await;
    }
}
