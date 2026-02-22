use std::collections::HashMap;

use crate::layers::{
    business::shared::errors::{BusinessError, InternalDependencyError, UseCaseError},
    ewm::main_database::qc_collection::camera_temp_blocking_qc_collection::{
        CreateCameraTempBlockingInput as QCCreateInput, ICameraTempBlockingQCCollection,
    },
};

use super::interface::{CreateCameraTempBlockingInput, ICreateCameraTempBlockingUseCase};

pub struct CreateCameraTempBlockingUseCaseImp<IICameraTempBlockingQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection,
{
    camera_temp_blocking_qc_collection: IICameraTempBlockingQCCollection,
}

impl<IICameraTempBlockingQCCollection>
    CreateCameraTempBlockingUseCaseImp<IICameraTempBlockingQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync,
{
    pub fn new(camera_temp_blocking_qc_collection: IICameraTempBlockingQCCollection) -> Self {
        Self {
            camera_temp_blocking_qc_collection,
        }
    }
}

impl<IICameraTempBlockingQCCollection> ICreateCameraTempBlockingUseCase
    for CreateCameraTempBlockingUseCaseImp<IICameraTempBlockingQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync,
{
    async fn execute(&self, input: CreateCameraTempBlockingInput) -> Result<(), UseCaseError> {
        let mut already_blocked_users = Vec::new();

        for user_id in &input.user_ids {
            let existing = self
                .camera_temp_blocking_qc_collection
                .get_temp_blocking(&input.camera_id, user_id)
                .await
                .map_err(|err| {
                    tracing::error!("{:?}", err);
                    UseCaseError::InternalDependencyError(InternalDependencyError::new(
                        "failed to check existing temp blocking".to_string(),
                        format!("{:?}", err),
                    ))
                })?;

            if existing.is_some() {
                already_blocked_users.push(user_id.clone());
            }
        }

        if !already_blocked_users.is_empty() {
            let mut details = HashMap::new();
            details.insert("already_blocked_users".to_string(), already_blocked_users);
            return Err(UseCaseError::BusinessError(BusinessError::new(
                "Some users already have a temp blocking for this camera".to_string(),
                details,
            )));
        }

        let qc_input = QCCreateInput {
            camera_id: input.camera_id,
            start_time: input.start_time,
            end_time: input.end_time,
            user_ids: input.user_ids,
        };

        self.camera_temp_blocking_qc_collection
            .create_temp_blocking(qc_input)
            .await
            .map_err(|err| {
                tracing::error!("{:?}", err);
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "failed to create camera temp blocking".to_string(),
                    format!("{:?}", err),
                ))
            })?;

        Ok(())
    }
}
