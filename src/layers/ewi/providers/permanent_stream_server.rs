use axum::extract::FromRef;

use crate::layers::{ewi::appstate::AppState, ewm::{permanent_stream_server::PermanentStreamServer}};

impl FromRef<AppState> for PermanentStreamServer {
    fn from_ref(app_state: &AppState) -> Self {
        PermanentStreamServer::new(app_state.app_config.permanent_relay_server_base_url.clone())
    }
}