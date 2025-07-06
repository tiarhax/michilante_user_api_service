
pub mod auth0;


use serde::Serialize;

use crate::layers::ewi::appstate::auth0::Auth0State;

//TODO: Find out how to shape AppState
#[derive(Clone, Serialize)]
pub struct StreamInfo {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub aws_config: aws_config::SdkConfig,
    pub app_config: AppConfig,
    pub auth0: Auth0State
}

#[derive(Clone)]
pub struct AppConfig {
    pub dynamo_db_table: String,
    pub permanent_relay_server_base_url: String,
    pub temporary_stream_server_base_url: String,
}


impl AppState {
    pub fn new(aws_config: aws_config::SdkConfig, app_config: AppConfig, auth0: Auth0State) -> Self {
        AppState {
            aws_config,
            app_config,
            auth0
        }
    }
}
