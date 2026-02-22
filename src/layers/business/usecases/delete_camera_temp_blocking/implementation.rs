use crate::layers::{
    business::shared::errors::{InternalDependencyError, UseCaseError},
    ewm::main_database::qc_collection::camera_temp_blocking_qc_collection::ICameraTempBlockingQCCollection,
};

use super::interface::{DeleteCameraTempBlockingInput, IDeleteCameraTempBlockingUseCase};

pub struct DeleteCameraTempBlockingUseCaseImp<IICameraTempBlockingQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection,
{
    camera_temp_blocking_qc_collection: IICameraTempBlockingQCCollection,
}

impl<IICameraTempBlockingQCCollection>
    DeleteCameraTempBlockingUseCaseImp<IICameraTempBlockingQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync,
{
    pub fn new(camera_temp_blocking_qc_collection: IICameraTempBlockingQCCollection) -> Self {
        Self {
            camera_temp_blocking_qc_collection,
        }
    }
}

impl<IICameraTempBlockingQCCollection> IDeleteCameraTempBlockingUseCase
    for DeleteCameraTempBlockingUseCaseImp<IICameraTempBlockingQCCollection>
where
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync,
{
    async fn execute(&self, input: DeleteCameraTempBlockingInput) -> Result<(), UseCaseError> {
        self.camera_temp_blocking_qc_collection
            .delete_temp_blocking(&input.camera_id, &input.user_id)
            .await
            .map_err(|err| {
                tracing::error!("{:?}", err);
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "failed to delete camera temp blocking".to_string(),
                    format!("{:?}", err),
                ))
            })?;

        Ok(())
    }
}
