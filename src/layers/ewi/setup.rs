use std::sync::Arc;

use aws_config::BehaviorVersion;
use axum::{
    routing::{delete, get, post, put},
    Router,
};



#[derive(Debug, Clone)]
struct ReadConfigErr {
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct StartupServerError {
    pub reason: String,
}

struct ServerConfig {
    pub http_port: i32,
    pub http_host: String,
    pub stream_expiration_time_in_minutes: i64,
    pub root_url: String,
    pub load_default_streams: bool,
    pub table_name: String,
    pub partition_key: String
}
fn read_config() -> Result<ServerConfig, ReadConfigErr> {
    let http_port: i32 = std::env::var("HTTP_PORT")
        .map_err(|_| ReadConfigErr {
            reason: "HTTP_PORT not set or invalid".to_string(),
        })?
        .parse()
        .map_err(|_| ReadConfigErr {
            reason: "HTTP_PORT must be a valid integer".to_string(),
        })?;

    let http_host: String = std::env::var("HTTP_HOST").map_err(|_| ReadConfigErr {
        reason: "HTTP_HOST not set".to_string(),
    })?;

    let stream_expiration_time_in_minutes: i64 = std::env::var("STREAM_EXPIRATION_TIME_IN_MINUTES")
        .map_err(|_| ReadConfigErr {
            reason: "STREAM_EXPIRATION_TIME_IN_MINUTES not set or invalid".to_string(),
        })?
        .parse()
        .map_err(|_| ReadConfigErr {
            reason: "STREAM_EXPIRATION_TIME_IN_MINUTES must be a valid integer".to_string(),
        })?;

    let root_url: String = std::env::var("ROOT_URL").map_err(|_| ReadConfigErr {
        reason: "ROOT_URL not set".to_string(),
    })?;

    let load_default_streams: bool = std::env::var("LOAD_DEFAULT_STREAMS")
        .map_err(|_| ReadConfigErr {
            reason: "LOAD_DEFAULT_STREAMS not set or invalid".to_string(),
        })?
        .parse()
        .map_err(|_| ReadConfigErr {
            reason: "LOAD_DEFAULT_STREAMS must be a valid boolean".to_string(),
        })?;
    let table_name = std::env::var("TABLE_NAME")
        .map_err(|_| ReadConfigErr {
            reason: "TABLE_NAME not set or invalid".to_string(),
        })?;

    let partition_key = std::env::var("PARTITION_KEY")
        .map_err(|_| ReadConfigErr {
            reason: "PARTITION_KEY not set or invalid".to_string(),
        })?;

    Ok(ServerConfig {
        http_port,
        http_host,
        stream_expiration_time_in_minutes,
        root_url,
        load_default_streams,
        table_name,
        partition_key,
    })
}

pub async fn setup_and_run() -> Result<(), StartupServerError> {
    

    let app = Router::new();
    let bind_str = format!("{}:{}", server_config.http_host, server_config.http_port);

    tracing::info!("Starting server on {}", bind_str);
    let listener = tokio::net::TcpListener::bind(&bind_str)
        .await
        .map_err(|err| StartupServerError {
            reason: format!("Failed to bind to {}: {:?}", bind_str, err),
        })?;


    axum::serve(listener, app)
        .await
        .map_err(|err| StartupServerError {
            reason: format!("Server error: {:?}", err),
        })?;
    Ok(())
}
