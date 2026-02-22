use crate::layers::business::shared::errors::UseCaseError;

pub struct BlockedUserInfo {
    pub user_id: String,
    pub user_name: String,
}

pub struct CameraTempBlockingItem {
    pub id: String,
    pub camera_id: String,
    pub end_date: String,
    pub blocked_user: BlockedUserInfo,
}

pub trait IListCameraTempBlockingsByCameraUseCase {
    fn execute(
        &self,
        camera_id: &str,
    ) -> impl std::future::Future<Output = Result<Vec<CameraTempBlockingItem>, UseCaseError>> + Send;
}
