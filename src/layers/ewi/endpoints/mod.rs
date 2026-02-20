use axum::Router;

use crate::layers::ewi::appstate::AppState;

pub mod cameras;
pub mod camerasv2;

pub fn setup_routes(router: Router<AppState>) -> Router<AppState> {
    let router = camerasv2::setup_endpoints(router);
    cameras::setup_endpoints(router)
}