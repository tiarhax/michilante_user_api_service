
use chrono::Utc;
use serde::Serialize;
//TODO: Find out how to shape AppState
#[derive(Clone, Serialize)]
pub struct StreamInfo {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Clone)]
pub struct StreamInfoInternal {
    pub id: String,
    pub name: String,
    pub url: String,
    pub expirable: bool,
    pub added_at: chrono::DateTime<Utc>,
}

#[derive(Clone)]
pub struct AppState {
    pub aws_config: aws_config::SdkConfig,
    pub app_config: AppConfig
}

#[derive(Clone)]
pub struct AppConfig {
    pub dynamo_db_table: String,
    pub permanent_relay_server_base_url: String
}


impl AppState {
    pub fn new(aws_config: aws_config::SdkConfig, app_config: AppConfig) -> Self {
        AppState {
            aws_config,
            app_config,
        }
    }
}
