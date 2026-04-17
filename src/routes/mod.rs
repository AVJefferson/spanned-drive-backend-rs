mod admin;
mod app_info;
mod health;
mod token;

use crate::{middlewares::authz::AuthzLayer, state::AppState};

use axum::{Extension, Router};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

fn cors_layer_for_non_local(environment: &str) -> CorsLayer {
    let cors_env = std::env::var("CORS").unwrap_or_else(|_| "".to_string());
    if cors_env.is_empty() {
        log::info!(
            "CORS: ENVIRONMENT={} with empty CORS — no origins allowed",
            environment
        );
        return CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|_: &axum::http::HeaderValue, _| false))
            .allow_methods(Any)
            .allow_headers(Any);
    }

    if cors_env.trim() == "*" {
        log::info!("CORS: ENVIRONMENT={} — allowing any origin (*)", environment);
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
    }

    let mut origins = Vec::new();
    for fragment in cors_env.split(',') {
        let fragment = fragment.trim();
        if fragment.is_empty() {
            continue;
        }
        match fragment.parse::<axum::http::HeaderValue>() {
            Ok(h) => origins.push(h),
            Err(e) => {
                log::warn!(
                    "CORS: skipping invalid origin {:?}: {}",
                    fragment,
                    e
                );
            }
        }
    }

    if origins.is_empty() {
        log::info!(
            "CORS: ENVIRONMENT={} — no valid origins after parsing; no origins allowed",
            environment
        );
        CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|_: &axum::http::HeaderValue, _| false))
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        log::info!("CORS: allowing {} explicit origin(s)", origins.len());
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    }
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());

    let cors = if environment == "dev" || environment == "local" {
        CorsLayer::new()
            .allow_origin(vec!["http://localhost:1420".parse::<axum::http::HeaderValue>().unwrap()])
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        cors_layer_for_non_local(&environment)
    };

    log::info!("CORS layer configured for ENVIRONMENT={}", environment);

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
