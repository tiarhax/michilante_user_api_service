use aws_sdk_dynamodb::Client;
use axum::extract::FromRef;

use crate::layers::{ewi::appstate::AppState, ewm::main_database::qc_collection::camera_qc_collection::CameraQCCollection};

impl FromRef<AppState> for CameraQCCollection {
    fn from_ref(app_state: &AppState) -> Self {
        let client = Client::new(&app_state.aws_config);
        CameraQCCollection::new(client, app_state.app_config.dynamo_db_table.clone())
    }
}