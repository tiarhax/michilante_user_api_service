use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::layers::ewm::permanent_stream_server::AddCreationOutput;

pub struct TemporaryStreamOutput {
    pub id: String,
    pub name: String,
    pub url: String,
    pub expiration_date: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct TemporaryStreamServerError {
    message: String,
    debug_message: String,
}
pub trait ITemporaryStreamServer {
    fn get_stream(
        &self,
        camera_id: &str,
        source_camera_stream_url: &str,
    ) -> impl std::future::Future<Output = Result<TemporaryStreamOutput, TemporaryStreamServerError>>
           + Send;
}

pub struct TemporaryStreamServer {
    base_url: String,
}
impl TemporaryStreamServer {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}
#[derive(Serialize)]
struct CreateStreamRequestBody {
    name: String,
    source_url: String,
    down_scale: bool,
    expirable: bool,
}
#[derive(Deserialize)]
pub struct AddStreamOutput {
    id: String,
    name: String,
    url: String,
    expiration_date: Option<String>,
}
impl ITemporaryStreamServer for TemporaryStreamServer {
    async fn get_stream(
        &self,
        camera_id: &str,
        source_camera_stream_url: &str,
    ) -> Result<TemporaryStreamOutput, TemporaryStreamServerError> {
        let request_body = CreateStreamRequestBody {
            name: camera_id.to_owned(),
            source_url: source_camera_stream_url.to_owned(),
            down_scale: true,
            expirable: true,
        };
        let url = format!("{}/streams", self.base_url);
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|err| TemporaryStreamServerError {
                message: "Failed to add temporary stream stream".to_string(),
                debug_message: err.to_string(),
            })?;
        let status = response.status();
        tracing::info!("add stream status code: {}", status);
        if status != StatusCode::OK {
            return Err(TemporaryStreamServerError {
                message: format!("request failed with: {}", status),
                debug_message: format!("request failed with: {}", status),
            });
        }
        let stream_output: AddStreamOutput =
            response
                .json()
                .await
                .map_err(|err| TemporaryStreamServerError {
                    message: "Failed to parse stream creation output".to_string(),
                    debug_message: format!("{:?}", err),
                })?;
        let expiration_date = match stream_output.expiration_date {
            Some(ed) => Some(DateTime::parse_from_rfc3339(&ed).map_err(|err| TemporaryStreamServerError {
                message: "Failed to parse expiration date".to_string(),
                debug_message: format!("Invalid RFC3339 format: {}", err),
            })?.with_timezone(&Utc)),
            None => {
                tracing::error!("Error temporary stream server returned a stream with no expiration date");
                None
            },
        };
        let stream_output = TemporaryStreamOutput {
            id: stream_output.id,
            name: stream_output.name,
            url: stream_output.url,
            expiration_date,
        };
        Ok(stream_output)
    }
}
