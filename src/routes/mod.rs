mod admin;
mod app_info;
mod drive;
mod health;
mod openapi;
mod profile;
mod token;

use crate::{middlewares::authz::check_authz, state::AppState};

use axum::{Router, middleware::from_fn};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::HeaderName;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

fn cors_layer_for_non_local() -> CorsLayer {
    let cors_env = std::env::var("CORS").unwrap_or_else(|_| "".to_string());
    let allowed_headers = [
        AUTHORIZATION,
        CONTENT_TYPE,
        ACCEPT,
        HeaderName::from_static("x-requested-with"),
    ];
    if cors_env.is_empty() {
        log::info!("CORS: Empty CORS — no origins allowed");
        return CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|_: &axum::http::HeaderValue, _| {
                false
            }))
            .allow_methods(Any)
            .allow_headers(allowed_headers);
    }

    if cors_env.trim() == "*" {
        log::info!("CORS: Allowing any origin (*)");
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(allowed_headers);
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
                log::warn!("CORS: skipping invalid origin {:?}: {}", fragment, e);
            }
        }
    }

    if origins.is_empty() {
        log::info!("CORS: No valid origins after parsing; no origins allowed",);
        CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(|_: &axum::http::HeaderValue, _| {
                false
            }))
            .allow_methods(Any)
            .allow_headers(allowed_headers)
    } else {
        log::info!("CORS: allowing {} explicit origin(s)", origins.len());
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(allowed_headers)
    }
}

pub fn routes(app_state: Arc<AppState>) -> Router {
    let cors = cors_layer_for_non_local();

    let authz = |permission: &str| {
        let app_state = app_state.clone();
        let perms: Arc<[String]> = Arc::from([permission.to_string()]);
        from_fn(move |req, next| {
            let app_state = app_state.clone();
            let perms = perms.clone();
            async move { check_authz(app_state, perms, req, next).await }
        })
    };

    Router::new()
        // .nest(
        //     "/admin",
        //     admin::routes(app_state.clone()).layer(authz("admin")),
        // )
        .nest(
            "/token",
            token::routes(app_state.clone()).layer(authz("token")),
        )
        .nest(
            "/drive",
            drive::routes(app_state.clone()).layer(authz("drive")),
        )
        .nest(
            "/profile",
            profile::routes(app_state.clone()).layer(authz("profile")),
        )
        .nest("/health", health::routes(app_state.clone()))
        .nest("/status", app_info::routes(app_state.clone()))
        .nest("/openapi", openapi::routes(app_state.clone()))
        .layer(cors)
}
