use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stream {
    pub id: String,
    pub name: String,
    pub url: String,
    pub added_at: String,
    pub expirable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddStreamInput {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AddStreamInputInternal {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub down_scale: bool,
    pub expirable: bool
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AddCreationOutput {
    pub id: String,
    pub name: String,
    pub url: String,
}
#[derive(Clone, Debug)]
pub struct PermanentStreamAPIError{
    pub message: String,
    pub debug_message: String
}
pub trait IPermanentStreamServer {
    fn list_streams(&self) -> impl std::future::Future<Output = Result<Vec<Stream>, PermanentStreamAPIError>> + Send;
    fn put_stream(&self, input: AddStreamInput) -> impl std::future::Future<Output = Result<AddCreationOutput, PermanentStreamAPIError>> + Send;
    fn remove_stream(&self, id: &str) -> impl std::future::Future<Output = Result<String, PermanentStreamAPIError>> + Send;

}

pub struct PermanentStreamServer {
    pub base_url: String
}

impl PermanentStreamServer {
    pub fn new(base_url: String) -> Self {
        PermanentStreamServer { base_url }
    }
}
impl IPermanentStreamServer for PermanentStreamServer {
    async fn list_streams(&self) -> Result<Vec<Stream>, PermanentStreamAPIError> {
        let base_url = self.base_url.clone();
        let url = format!("{}/streams", base_url);
        let response = reqwest::get(&url).await.map_err(|err| PermanentStreamAPIError {
            message: "Failed to fetch streams".to_string(),
            debug_message: err.to_string(),
        })?;
        let streams: Vec<Stream> = response.json().await.map_err(|err| PermanentStreamAPIError {
            message: "Failed to parse streams".to_string(),
            debug_message: err.to_string(),
        })?;
        Ok(streams)
    }

    async fn put_stream(&self, input: AddStreamInput) -> Result<AddCreationOutput, PermanentStreamAPIError> {
        let base_url = self.base_url.clone();
        let url = format!("{}/streams/permanent/{}", base_url, input.id);
        tracing::info!("add stream url: {}", url);
        let client = reqwest::Client::new();
        let request_body = AddStreamInputInternal {
            id: input.id,
            name: input.name,
            source_url: input.url,
            down_scale: false,
            expirable: false
        };
        let response = client.put(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|err| PermanentStreamAPIError {
                message: "Failed to add stream".to_string(),
                debug_message: err.to_string(),
            })?;
        let status = response.status();
        tracing::info!("add stream status code: {}", status);
        if status != StatusCode::OK {
            return Err(
                PermanentStreamAPIError { message: format!("request failed with: {}", status), debug_message: format!("request failed with: {}", status) }
            )
        }
        let stream_output: AddCreationOutput = response.json().await.map_err(|err| PermanentStreamAPIError {
            message: "Failed to parse stream creation output".to_string(),
            debug_message: format!("{:?}", err),
        })?;
        Ok(stream_output)
    }

    async fn remove_stream(&self, id: &str) -> Result<String, PermanentStreamAPIError> {
        let base_url = self.base_url.clone();
        let url = format!("{}/streams/{}", base_url, id);
        let client = reqwest::Client::new();
        client.delete(&url)
            .send()
            .await
            .map_err(|err| PermanentStreamAPIError {
                message: "Failed to remove stream".to_string(),
                debug_message: err.to_string(),
            })?;
        Ok(id.to_owned())
    }
}
