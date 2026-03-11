use crate::layers::business::shared::errors::UseCaseError;

pub struct BlockableUserItem {
    pub user_id: String,
    pub email: String,
    pub name: String,
}

pub trait IListBlockableUsersForCameraUseCase {
    fn execute(
        &self,
        camera_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<BlockableUserItem>, UseCaseError>> + Send;
}
