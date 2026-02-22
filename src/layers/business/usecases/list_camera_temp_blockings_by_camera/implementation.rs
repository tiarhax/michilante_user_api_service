use std::collections::HashMap;

use crate::layers::{
    business::shared::errors::{InternalDependencyError, UseCaseError},
    ewm::main_database::qc_collection::{
        camera_temp_blocking_qc_collection::ICameraTempBlockingQCCollection,
        user_qc_collection::IUserQCCollection,
    },
};

use super::interface::{BlockedUserInfo, CameraTempBlockingItem, IListCameraTempBlockingsByCameraUseCase};

pub struct ListCameraTempBlockingsByCameraUseCaseImp<IICameraTempBlockingQCCollection, IIUserQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection,
    IIUserQCCollection: IUserQCCollection,
{
    camera_temp_blocking_qc_collection: IICameraTempBlockingQCCollection,
    user_qc_collection: IIUserQCCollection,
}

impl<IICameraTempBlockingQCCollection, IIUserQCCollection>
    ListCameraTempBlockingsByCameraUseCaseImp<IICameraTempBlockingQCCollection, IIUserQCCollection>
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

impl<IICameraTempBlockingQCCollection, IIUserQCCollection> IListCameraTempBlockingsByCameraUseCase
    for ListCameraTempBlockingsByCameraUseCaseImp<IICameraTempBlockingQCCollection, IIUserQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync,
    IIUserQCCollection: IUserQCCollection + Sync,
{
    async fn execute(&self, camera_id: &str) -> Result<Vec<CameraTempBlockingItem>, UseCaseError> {
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

        let user_ids: Vec<String> = blockings.iter().map(|b| b.user_id.clone()).collect();

        let users = self
            .user_qc_collection
            .find_users_by_ids(user_ids)
            .await
            .map_err(|err| {
                tracing::error!("{:?}", err);
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "failed to find users".to_string(),
                    format!("{:?}", err),
                ))
            })?;

        let user_map: HashMap<String, String> = users
            .into_iter()
            .map(|u| (u.user_id, u.name))
            .collect();

        let result = blockings
            .into_iter()
            .map(|blocking| {
                let user_name = user_map
                    .get(&blocking.user_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());

                CameraTempBlockingItem {
                    id: blocking.id,
                    camera_id: blocking.camera_id,
                    end_date: blocking.end_date,
                    blocked_user: BlockedUserInfo {
                        user_id: blocking.user_id,
                        user_name,
                    },
                }
            })
            .collect();

        Ok(result)
    }
}
