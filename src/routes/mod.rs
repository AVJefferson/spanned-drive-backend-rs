mod admin;
mod app_info;
mod health;
mod token;

use crate::{middlewares::authz::AuthzLayer, state::AppState};

use axum::{Extension, Router};
use std::sync::Arc;

pub fn routes(app_state: Arc<AppState>) -> Router {
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
}
