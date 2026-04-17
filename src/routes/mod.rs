mod admin;
mod app_info;
mod health;
mod token;

use crate::{
    middlewares::authz::{authz_middleware, TokenRouterState},
    state::AppState,
};

use axum::{middleware::from_fn_with_state, Router};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

fn cors_layer_for_non_local() -> CorsLayer {
    let cors_env = std::env::var("CORS").unwrap_or_else(|_| "".to_string());
    if cors_env.is_empty() {
        log::info!(
            "CORS: Empty CORS — no origins allowed"
        );
        return CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|_: &axum::http::HeaderValue, _| false))
            .allow_methods(Any)
            .allow_headers(Any);
    }

    if cors_env.trim() == "*" {
        log::info!("CORS: Allowing any origin (*)");
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
            "CORS: No valid origins after parsing; no origins allowed",
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
    let cors = cors_layer_for_non_local();
    

    let token_state = TokenRouterState {
        app_state: app_state.clone(),
        required_permissions: Arc::from(["token".to_string()]),
    };

    Router::new()
        // .nest(
        //     "/admin",
        //     admin::routes(app_state.clone()).layer(AuthzLayer::new(vec!["admin".to_string()])),
        // )
        .nest(
            "/token",
            token::routes(token_state.clone()).layer(from_fn_with_state(
                token_state.clone(),
                authz_middleware,
            )),
        )
        .nest("/health", health::routes(app_state.clone()))
        .nest("/status", app_info::routes(app_state.clone()))
        .layer(cors)
}
