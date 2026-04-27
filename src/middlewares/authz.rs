use crate::state::AppState;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::headers::{Authorization, HeaderMapExt, authorization::Bearer};
use std::sync::Arc;

pub async fn check_authz(
    app_state: Arc<AppState>,
    required_permissions: Arc<[String]>,
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

    let is_authorized = match app_state.auth_tokens.get(&token) {
        Some(user_permissions) => required_permissions
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
    use axum::{Router, middleware::from_fn, routing::get};
    use axum_test::TestServer;
    use dashmap::DashMap;

    use crate::external_systems::ExternalClients;
    use crate::test_utils::timeout;

    use super::*;

    #[tokio::test]
    async fn authz() {
        timeout::with_default(async {
            let clients = ExternalClients::empty();
            let auth_tokens = DashMap::new();
            auth_tokens.insert("test".to_string(), vec!["admin".to_string()]);
            auth_tokens.insert("test_no_permission".to_string(), vec![]);
            auth_tokens.insert("test_min_permission".to_string(), vec!["user".to_string()]);
            let app_state = Arc::new(AppState {
                clients,
                auth_tokens,
            });

            let required_permissions: Arc<[String]> = Arc::from(["admin".to_string()]);

            let app = Router::new()
                .route("/", get(|| async { (StatusCode::OK, "OK") }))
                .layer(from_fn({
                    let app_state = app_state.clone();
                    let perms = required_permissions.clone();
                    move |req, next| {
                        let app_state = app_state.clone();
                        let perms = perms.clone();
                        async move { check_authz(app_state, perms, req, next).await }
                    }
                }));
            let server = TestServer::new(app);

            let response_no_auth = server.get("/").await;
            response_no_auth.assert_status_unauthorized();
            assert_eq!(response_no_auth.text(), "Missing authorization header");

            let response_wrong_auth = server.get("/").authorization_bearer("wrong").await;
            response_wrong_auth.assert_status_unauthorized();
            assert_eq!(response_wrong_auth.text(), "Invalid token");

            let response_no_perm = server
                .get("/")
                .authorization_bearer("test_no_permission")
                .await;
            response_no_perm.assert_status_forbidden();
            assert_eq!(response_no_perm.text(), "Insufficient permissions");

            let response_min_perm = server
                .get("/")
                .authorization_bearer("test_min_permission")
                .await;
            response_min_perm.assert_status_forbidden();
            assert_eq!(response_min_perm.text(), "Insufficient permissions");

            let response_auth = server.get("/").authorization_bearer("test").await;
            response_auth.assert_status_ok();
        })
        .await;
    }
}
