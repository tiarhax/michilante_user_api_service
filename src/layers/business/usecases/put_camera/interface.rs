use chrono::Utc;

use crate::layers::business::shared::errors::UseCaseError;

#[derive(Debug)]
pub struct PutCameraInput {
    pub id: String,
    pub name: String,
    pub source_url: String,
}

pub struct PutCameraOutput {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub trait IPutCameraUseCase {
    fn execute(&self, input: PutCameraInput) -> impl std::future::Future<Output = Result<PutCameraOutput, UseCaseError>> + Send;
}
