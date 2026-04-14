mod admin;
mod app_info;
mod health;
mod token;

use crate::{middlewares::authz::AuthzLayer, state::AppState};

use axum::{Extension, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub fn routes(app_state: Arc<AppState>) -> Router {
    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());

    let cors = if environment == "dev" || environment == "local" {
        CorsLayer::new()
            .allow_origin(vec!["http://localhost:1420".parse::<axum::http::HeaderValue>().unwrap()])
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let cors_env = std::env::var("CORS").unwrap_or_else(|_| "".to_string());
        if cors_env.is_empty() {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        } else {
            let origins: Vec<axum::http::HeaderValue> = cors_env
                .split(',')
                .map(|s| s.trim().parse::<axum::http::HeaderValue>().expect("Invalid CORS header value"))
                .collect();
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(Any)
                .allow_headers(Any)
        }
    };

    Router::new()
        // .nest(
        //     "/admin",
        //     admin::routes(app_state.clone()).layer(AuthzLayer::new(vec!["admin".to_string()])),
        // )
        .nest(
            "/token",
            token::routes(app_state.clone()).layer(AuthzLayer::new(vec!["token".to_string()])),
        )
        .nest("/health", health::routes(app_state.clone()))
        .nest("/status", app_info::routes(app_state.clone()))
        .layer(Extension(app_state))
        .layer(cors)
}
