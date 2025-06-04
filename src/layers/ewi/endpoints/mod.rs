use axum::Router;

use crate::layers::ewi::appstate::AppState;

pub mod cameras;

pub fn setup_routes(router: Router<AppState>) -> Router<AppState> {

    cameras::setup_endpoints(router)
}