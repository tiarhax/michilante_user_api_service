use aws_sdk_dynamodb::Client;
use axum::extract::FromRef;

use crate::layers::{ewi::appstate::AppState, ewm::main_database::qc_collection::camera_temp_blocking_qc_collection::CameraTempBlockingQCCollection};

impl FromRef<AppState> for CameraTempBlockingQCCollection {
    fn from_ref(app_state: &AppState) -> Self {
        let client = Client::new(&app_state.aws_config);
        CameraTempBlockingQCCollection::new(client, app_state.app_config.dynamo_db_table.clone())
    }
}
