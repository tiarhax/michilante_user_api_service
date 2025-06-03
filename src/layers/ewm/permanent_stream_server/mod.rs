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
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AddStreamInputInternal {
    pub name: String,
    pub url: String,
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
    fn add_stream(&self, input: AddStreamInput) -> impl std::future::Future<Output = Result<AddCreationOutput, PermanentStreamAPIError>> + Send;
    fn remove_stream(&self, id: String) -> impl std::future::Future<Output = Result<String, PermanentStreamAPIError>> + Send;

}

pub struct PermanentStreamServer {
    pub base_url: String
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

    async fn add_stream(&self, input: AddStreamInput) -> Result<AddCreationOutput, PermanentStreamAPIError> {
        let base_url = self.base_url.clone();
        let url = format!("{}/streams", base_url);
        let client = reqwest::Client::new();
        let request_body = AddStreamInputInternal {
            name: input.name,
            url: input.url,
            down_scale: false,
            expirable: false
        };
        let response = client.post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|err| PermanentStreamAPIError {
                message: "Failed to add stream".to_string(),
                debug_message: err.to_string(),
            })?;
        let stream_output: AddCreationOutput = response.json().await.map_err(|err| PermanentStreamAPIError {
            message: "Failed to parse stream creation output".to_string(),
            debug_message: err.to_string(),
        })?;
        Ok(stream_output)
    }

    async fn remove_stream(&self, id: String) -> Result<String, PermanentStreamAPIError> {
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
        Ok(id)
    }
}
