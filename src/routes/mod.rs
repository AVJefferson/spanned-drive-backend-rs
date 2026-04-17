mod admin;
mod app_info;
mod health;
mod token;

use crate::{middlewares::authz::AuthzLayer, state::AppState};

use axum::{Extension, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub fn routes(app_state: Arc<AppState>) -> Router {
    let cors = if let Ok(cors_env) = std::env::var("CORS") {
        if cors_env == "*" {
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
    } else {
        // If CORS variable is not set, no CORS allowed (disable all)
        CorsLayer::new()
            .allow_origin(Vec::<axum::http::HeaderValue>::new())
            .allow_methods(Vec::<axum::http::Method>::new())
            .allow_headers(Vec::<axum::http::HeaderName>::new())
    };

    println!("CORS: {:?}", cors);

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
