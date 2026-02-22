use crate::layers::business::shared::errors::UseCaseError;

pub struct UserListItem {
    pub user_id: String,
    pub email: String,
    pub name: String,
}

pub trait IListUsersUseCase {
    fn execute(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<UserListItem>, UseCaseError>> + Send;
}
