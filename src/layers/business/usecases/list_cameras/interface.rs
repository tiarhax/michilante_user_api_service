use crate::layers::business::shared::errors::UseCaseError;

#[derive(Debug, Clone)]
pub struct CameraListItem {
    pub id: String,
    pub name: String,
    pub source_url: String
}




pub trait IListCamerasUseCase {
    fn execute(&self) -> impl std::future::Future<Output = Result<Vec<CameraListItem>, UseCaseError>> + Send;
}