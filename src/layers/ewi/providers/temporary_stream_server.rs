use axum::extract::FromRef;

use crate::layers::{ewi::appstate::AppState, ewm::{permanent_stream_server::PermanentStreamServer, temporary_stream_server::TemporaryStreamServer}};

impl FromRef<AppState> for TemporaryStreamServer {
    fn from_ref(app_state: &AppState) -> Self {
        TemporaryStreamServer::new(app_state.app_config.temporary_stream_server_base_url.clone())
    }
}