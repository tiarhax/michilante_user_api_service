use crate::layers::{
    business::shared::errors::{InternalDependencyError, UseCaseError},
    ewm::main_database::qc_collection::user_qc_collection::IUserQCCollection,
};

use super::interface::{IListUsersUseCase, UserListItem};

pub struct ListUsersUseCaseImp<IIUserQCCollection>
where
    IIUserQCCollection: IUserQCCollection,
{
    user_qc_collection: IIUserQCCollection,
}

impl<IIUserQCCollection> ListUsersUseCaseImp<IIUserQCCollection>
where
    IIUserQCCollection: IUserQCCollection + Sync,
{
    pub fn new(user_qc_collection: IIUserQCCollection) -> Self {
        Self { user_qc_collection }
    }
}

impl<IIUserQCCollection> IListUsersUseCase for ListUsersUseCaseImp<IIUserQCCollection>
where
    IIUserQCCollection: IUserQCCollection + Sync,
{
    async fn execute(&self) -> Result<Vec<UserListItem>, UseCaseError> {
        let query_results = self
            .user_qc_collection
            .list_users()
            .await
            .map_err(|err| {
                tracing::error!("{:?}", err);
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "failed to load users from database".to_string(),
                    format!("{:?}", err),
                ))
            })?;

        let results = query_results
            .into_iter()
            .map(|s| UserListItem {
                user_id: s.user_id,
                email: s.email,
                name: s.name,
            })
            .collect();

        Ok(results)
    }
}
