use crate::layers::{
    business::shared::errors::{InternalDependencyError, UseCaseError},
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
