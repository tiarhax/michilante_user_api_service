use crate::layers::{business::shared::errors::{InternalDependencyError, UseCaseError}, ewm::main_database::qc_collection::camera_qc_collection::{ICameraQCCollection}};

use super::interface::{CameraListItem, IListCamerasUseCase};

pub struct ListCamerasUseCaseImp<IICameraCommandQueryCollection> 
where IICameraCommandQueryCollection: ICameraQCCollection
{
    camera_qc_collection: IICameraCommandQueryCollection
}

impl<IICameraCommandQueryCollection> IListCamerasUseCase for ListCamerasUseCaseImp<IICameraCommandQueryCollection>
    where IICameraCommandQueryCollection: ICameraQCCollection
{
    async fn execute(&self) -> Result<Vec<CameraListItem>, UseCaseError> {
        let query_results = self
            .camera_qc_collection
            .list_cameras()
            .await
            .map_err(|err| {
                UseCaseError::InternalDependencyError(
                    InternalDependencyError::new("failed to load cameras from database".to_string(), format!("{:?}", err))
                )
            })?;
        let results = query_results
            .into_iter()
            .map(|s| {
                CameraListItem {
                    id: s.id,
                    name: s.name
                }
            })
            .collect();
        Ok(results)
    }
}