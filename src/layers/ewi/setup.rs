
use aws_config::{BehaviorVersion};

use axum::{middleware, Router};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::layers::ewi::{
    appstate::{auth0::{Auth0Config, Auth0State}, AppConfig, AppState}, endpoints, middleware::auth0::{self, auth0_middleware}
};
use std::env;

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
}

fn read_server_config_from_env() -> Result<ServerConfig, ReadConfigErr> {
    let http_port = env::var("HTTP_PORT")
        .map_err(|err| ReadConfigErr {
            reason: format!("Failed to read HTTP_PORT from env: {:?}", err),
        })?
        .parse::<i32>()
        .map_err(|err| ReadConfigErr {
            reason: format!("Failed to parse HTTP_PORT as i32: {:?}", err),
        })?;

    let http_host = env::var("HTTP_HOST").map_err(|err| ReadConfigErr {
        reason: format!("Failed to read HTTP_HOST from env: {:?}", err),
    })?;

    Ok(ServerConfig {
        http_port,
        http_host,
    })
}
fn read_app_config_from_env() -> Result<AppConfig, ReadConfigErr> {
    let dynamo_db_table = env::var("DYNAMO_DB_TABLE").map_err(|err| ReadConfigErr {
        reason: format!("Failed to read DYNAMO_DB_TABLE from env: {:?}", err),
    })?;

    let permanent_relay_server_base_url = env::var("PERMANENT_STREAM_SERVER_URL").map_err(|err| ReadConfigErr {
        reason: format!("Failed to read PERMANENT_STREAM_SERVER_URL from env: {:?}", err),
    })?;

    let temporary_stream_server_base_url = env::var("TEMPORARY_STREAM_SERVER_URL").map_err(|err| ReadConfigErr {
        reason: format!("Failed to read TEMPORARY_STREAM_SERVER_URL from env: {:?}", err),
    })?;

    Ok(AppConfig { dynamo_db_table, permanent_relay_server_base_url, temporary_stream_server_base_url })
}

fn read_auth0_config_from_env() -> Result<Auth0Config, ReadConfigErr> {
    let auth0_domain = env::var("AUTH0_DOMAIN").map_err(|err| ReadConfigErr {
        reason: format!("Failed to read AUTH0_DOMAIN from env: {:?}", err),
    })?;

    let auth0_audience = env::var("AUTH0_AUDIENCE").map_err(|err| ReadConfigErr {
        reason: format!("Failed to read AUTH0_AUDIENCE from env: {:?}", err),
    })?;

    let auth0_issuer = env::var("AUTH0_ISSUER").map_err(|err| ReadConfigErr {
        reason: format!("Failed to read AUTH0_ISSUER from env: {:?}", err),
    })?;

    Ok(Auth0Config {
        domain: auth0_domain,
        audience: auth0_audience,
        issuer: auth0_issuer
    })
}
pub async fn setup_and_run() -> Result<(), StartupServerError> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().map_err(|e| {
        StartupServerError {
            reason: format!("{:?}", e)
        }
    })?;
    let aws_sdk_config = aws_config::load_defaults(BehaviorVersion::v2025_01_17()).await;
    let app_config = read_app_config_from_env().map_err(|err| StartupServerError {
        reason: format!("Failed to read app config: {:?}", err),
    })?;

    let server_config = read_server_config_from_env().map_err(|err| StartupServerError {
        reason: format!("Failed to read server config: {:?}", err),
    })?;
    let auth0_config = read_auth0_config_from_env().map_err(|err| StartupServerError {
        reason: format!("Failed to read auth0 config: {:?}", err),
    })?;

    let auth0_state = Auth0State::new(auth0_config);

    let app_state = AppState {
        app_config,
        aws_config: aws_sdk_config,
        auth0: auth0_state
    };

    let app = endpoints::setup_routes(Router::new())
        .with_state(app_state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn_with_state(app_state.clone(), auth0_middleware))
        );

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
