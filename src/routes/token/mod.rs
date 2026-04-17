mod google_web;

use crate::middlewares::authz::TokenRouterState;

use axum::Router;

pub fn routes(state: TokenRouterState) -> Router {
    Router::new().nest("/google-drive", google_web::routes(state))
}
