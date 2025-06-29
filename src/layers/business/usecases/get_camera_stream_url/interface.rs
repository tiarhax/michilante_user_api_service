use crate::layers::business::shared::errors::UseCaseError;

pub struct GetCameraStreamURLOutput {
    pub camera_id: String,
    pub temp_rtsp_url: String,
    pub expiration_date: chrono::DateTime<chrono::Utc>,
}

pub trait IGetCameraStremaURLUseCase {
    fn execute(&self, id: &str) -> impl std::future::Future<Output = Result<GetCameraStreamURLOutput, UseCaseError>> + Send;
}