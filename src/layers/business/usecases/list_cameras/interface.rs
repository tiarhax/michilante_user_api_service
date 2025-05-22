use crate::layers::business::shared::errors::UseCaseError;


pub struct CameraListItem {
    pub id: String,
    pub name: String
}




pub trait IListCamerasUseCase {
    async fn execute(&self) -> Result<Vec<CameraListItem>, UseCaseError>;
}