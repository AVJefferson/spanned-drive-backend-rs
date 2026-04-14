use crate::state::AppState;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use axum_extra::headers::{Authorization, HeaderMapExt, authorization::Bearer};
use futures_util::future::BoxFuture;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct AuthzLayer {
    required_permissions: Arc<[String]>,
}

impl AuthzLayer {
    pub fn new(required_permissions: Vec<String>) -> Self {
        Self {
            required_permissions: Arc::from(required_permissions),
        }
    }
}

impl<S> Layer<S> for AuthzLayer {
    type Service = AuthzMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthzMiddleware {
            inner,
            required_permissions: self.required_permissions.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthzMiddleware<S> {
    inner: S,
    required_permissions: Arc<[String]>,
}

impl<S> Service<Request<Body>> for AuthzMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let required_permissions = self.required_permissions.clone();
        let state = req.extensions().get::<Arc<AppState>>().cloned();
        let auth_header = req.headers().typed_get::<Authorization<Bearer>>();

        let mut inner = self.inner.clone();

        Box::pin(async move {
            let state = match state {
                Some(state) => state,
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("AppState not found"))
                        .unwrap());
                }
            };

            let token = match auth_header {
                Some(Authorization(bearer)) => bearer.token().to_string(),
                None => {
                    log::trace!("Missing authorization header");
                    return Ok(Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::from("Missing authorization header"))
                        .unwrap());
                }
            };

            let is_authorized = match state.auth_tokens.get(&token) {
                Some(user_permissions) => required_permissions
                    .iter()
                    .any(|p| user_permissions.contains(p)),
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::from("Invalid token"))
                        .unwrap());
                }
            };

            if is_authorized {
                inner.call(req).await
            } else {
                log::trace!("Insufficient permissions");
                Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("Insufficient permissions"))
                    .unwrap())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use axum::{Extension, Router, routing::get};
    use axum_test::TestServer;
    use dashmap::DashMap;

    use crate::{clients::Clients, middlewares::authz::AuthzLayer};
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

            let app = Router::new()
                .route("/", get(|| async { (StatusCode::OK, "OK") }))
                .layer(AuthzLayer::new(vec!["admin".to_string()]))
                .layer(Extension(app_state.clone()));
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
