use crate::layers::{business::{shared::errors::{InternalDependencyError, UseCaseError}, usecases::delete_camera::{interface::IDeleteCameraUseCase, sanitization_rules::DeleteCameraSanitizedInput}}, ewi::error::AppError, ewm::{
    main_database::qc_collection::camera_qc_collection::ICameraQCCollection,
    permanent_stream_server::IPermanentStreamServer,
}};

pub struct DeleteCameraUseCase<IICamercaQCCollection, IIPermanentStreamServer>
where
    IICamercaQCCollection: ICameraQCCollection,
    IIPermanentStreamServer: IPermanentStreamServer,
{
    camera_qc_collection: IICamercaQCCollection,
    permanent_stream_server: IIPermanentStreamServer,
}

impl<IICamercaQCCollection, IIPermanentStreamServer>
    DeleteCameraUseCase<IICamercaQCCollection, IIPermanentStreamServer>
where
    IICamercaQCCollection: ICameraQCCollection,
    IIPermanentStreamServer: IPermanentStreamServer,
{
    pub fn new(
        camera_qc_collection: IICamercaQCCollection,
        permanent_stream_server: IIPermanentStreamServer,
    ) -> Self {
        Self {
            camera_qc_collection,
            permanent_stream_server,
        }
    }
}


impl<IICamercaQCCollection, IIPermanentStreamServer> IDeleteCameraUseCase for
    DeleteCameraUseCase<IICamercaQCCollection, IIPermanentStreamServer>
where
    IICamercaQCCollection: ICameraQCCollection + Sync,
    IIPermanentStreamServer: IPermanentStreamServer + Sync,
{
    async fn execute(&self, input: String) -> Result<(), UseCaseError> {
        let sanitized_input: DeleteCameraSanitizedInput = input.try_into().map_err(|err| {
            UseCaseError::InternalDependencyError(InternalDependencyError::new(
                "Failed to sanitize input".to_owned(),
                err,
            ))
        })?;

        self.camera_qc_collection.delete_camera_by_id(&sanitized_input.0).await.map_err(
            |err| {
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "Failed to delete camera from database".to_owned(),
                    format!("{:?}", err),
                ))
            }
        )?;

        self.permanent_stream_server.remove_stream(&sanitized_input.0).await.map_err(|err| {
            UseCaseError::InternalDependencyError(InternalDependencyError::new(
                "Failed to remove stream from permanent stream server".to_owned(),
                format!("{:?}", err),
            ))
        })?;

        Ok(())
    }
}



