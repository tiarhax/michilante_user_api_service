use crate::layers::business::shared::errors::UseCaseError;


#[derive(Debug, Clone)]
pub enum CameraAvailability {
    Available,
    NotAvailable(String)
}

#[derive(Debug, Clone)]
pub struct CameraListItem {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub is_available: CameraAvailability
}

pub struct ListCamerasInput {
    pub user_id: String
}

pub trait IListCamerasUseCase {
    fn execute(&self, input: &ListCamerasInput) -> impl std::future::Future<Output = Result<Vec<CameraListItem>, UseCaseError>> + Send;
}