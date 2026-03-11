use std::collections::HashSet;

use crate::layers::{
    business::shared::errors::{InternalDependencyError, UseCaseError},
    ewm::main_database::qc_collection::{
        camera_temp_blocking_qc_collection::ICameraTempBlockingQCCollection,
        user_qc_collection::IUserQCCollection,
    },
};

use super::interface::{BlockableUserItem, IListBlockableUsersForCameraUseCase};

pub struct ListBlockableUsersForCameraUseCaseImp<IICameraTempBlockingQCCollection, IIUserQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection,
    IIUserQCCollection: IUserQCCollection,
{
    camera_temp_blocking_qc_collection: IICameraTempBlockingQCCollection,
    user_qc_collection: IIUserQCCollection,
}

impl<IICameraTempBlockingQCCollection, IIUserQCCollection>
    ListBlockableUsersForCameraUseCaseImp<IICameraTempBlockingQCCollection, IIUserQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync,
    IIUserQCCollection: IUserQCCollection + Sync,
{
    pub fn new(
        camera_temp_blocking_qc_collection: IICameraTempBlockingQCCollection,
        user_qc_collection: IIUserQCCollection,
    ) -> Self {
        Self {
            camera_temp_blocking_qc_collection,
            user_qc_collection,
        }
    }
}

impl<IICameraTempBlockingQCCollection, IIUserQCCollection> IListBlockableUsersForCameraUseCase
    for ListBlockableUsersForCameraUseCaseImp<IICameraTempBlockingQCCollection, IIUserQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync,
    IIUserQCCollection: IUserQCCollection + Sync,
{
    async fn execute(&self, camera_id: &str) -> Result<Vec<BlockableUserItem>, UseCaseError> {
        let blockings = self
            .camera_temp_blocking_qc_collection
            .list_temp_blocking_for_camera(camera_id)
            .await
            .map_err(|err| {
                tracing::error!("{:?}", err);
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "failed to list camera temp blockings".to_string(),
                    format!("{:?}", err),
                ))
            })?;

        let blocked_user_ids: HashSet<String> = blockings
            .into_iter()
            .map(|b| b.user_id)
            .collect();

        let all_users = self
            .user_qc_collection
            .list_users()
            .await
            .map_err(|err| {
                tracing::error!("{:?}", err);
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "failed to list users".to_string(),
                    format!("{:?}", err),
                ))
            })?;

        let blockable_users = all_users
            .into_iter()
            .filter(|user| !blocked_user_ids.contains(&user.user_id))
            .map(|user| BlockableUserItem {
                user_id: user.user_id,
                email: user.email,
                name: user.name,
            })
            .collect();

        Ok(blockable_users)
    }
}
