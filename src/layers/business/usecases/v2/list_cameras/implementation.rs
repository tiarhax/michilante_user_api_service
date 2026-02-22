use crate::layers::{business::{shared::errors::{InternalDependencyError, UseCaseError}, usecases::v2::list_cameras::interface::ListCamerasInput}, ewm::main_database::qc_collection::{camera_qc_collection::ICameraQCCollection, camera_temp_blocking_qc_collection::ICameraTempBlockingQCCollection}};

use super::interface::{CameraListItem, IListCamerasUseCase};

pub struct ListCamerasUseCaseImp<IICameraCommandQueryCollection, IICameraTempBlockingQCCollection> 
where 
    IICameraCommandQueryCollection: ICameraQCCollection,
IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync
{
    camera_qc_collection: IICameraCommandQueryCollection,
    camera_temp_blocking_qc_collection: IICameraTempBlockingQCCollection
}

impl<IICameraCommandQueryCollection, IICameraTempBlockingQCCollection> ListCamerasUseCaseImp<IICameraCommandQueryCollection, IICameraTempBlockingQCCollection>
where
    IICameraCommandQueryCollection: ICameraQCCollection,
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync
{
    pub fn new(
        camera_qc_collection: IICameraCommandQueryCollection,
        camera_temp_blocking_qc_collection: IICameraTempBlockingQCCollection
    ) -> Self {
        Self { camera_qc_collection, camera_temp_blocking_qc_collection }
    }
}

impl<IICameraCommandQueryCollection, IICameraTempBlockingQCCollection>
    IListCamerasUseCase for ListCamerasUseCaseImp<IICameraCommandQueryCollection, IICameraTempBlockingQCCollection>
    where IICameraCommandQueryCollection: ICameraQCCollection + Sync,
    IICameraTempBlockingQCCollection: ICameraTempBlockingQCCollection + Sync
{
    async fn execute(&self, input: &ListCamerasInput) -> Result<Vec<CameraListItem>, UseCaseError> {
        let query_results = self
            .camera_qc_collection
            .list_cameras()
            .await
            .map_err(|err| {
                tracing::error!("{:?}", err);
                UseCaseError::InternalDependencyError(
                    InternalDependencyError::new("failed to load cameras from database".to_string(), format!("{:?}", err))
                )
            })?;
        let mut results: Vec<CameraListItem> = query_results
            .into_iter()
            .map(|s| {
                CameraListItem {
                    id: s.id,
                    name: s.name,
                    source_url: s.source_url,
                    is_available: super::interface::CameraAvailability::Available
                }
            })
            .collect();

        let temp_blockings = self.camera_temp_blocking_qc_collection.list_temp_blocking_for_user(&input.user_id)
            .await
            .map_err(|err| {
                tracing::error!("{:?}", err);
                UseCaseError::InternalDependencyError(
                    InternalDependencyError::new("failed to load camera temp blockings from database".to_string(), format!("{:?}", err))
                )
            })?;

        for blocking in temp_blockings {
            if let Some(camera) = results.iter_mut().find(|c| c.id == blocking.camera_id) {
                camera.is_available = super::interface::CameraAvailability::NotAvailable(blocking.end_date);
            }
        }
        Ok(results)
    }
}
#[cfg(test)]
mod tests {

    use crate::layers::ewm::main_database::qc_collection::{camera_qc_collection::{CameraListQueryResultItem, ListCamerasQueryError}, camera_temp_blocking_qc_collection::{CameraTempBlocking, ListCameraTempBlockingsQueryError}, error::QCError};

    use super::*;

    struct MockCameraQCCollection {
        cameras: Vec<CameraListQueryResultItem>,
        should_fail: bool,
    }

    impl ICameraQCCollection for MockCameraQCCollection {
        async fn list_cameras(&self) -> Result<Vec<CameraListQueryResultItem>, ListCamerasQueryError> {
            if self.should_fail {
                Err(ListCamerasQueryError(QCError { message: "mock error".to_string(), debug_details: None}))
            } else {
                Ok(self.cameras.clone())
            }
        }
        
        async fn put_camera(
            &self,
            command_input: crate::layers::ewm::main_database::qc_collection::camera_qc_collection::PutCameraCommandInput,
        ) -> Result<crate::layers::ewm::main_database::qc_collection::camera_qc_collection::CreateCameraCommandOutput, crate::layers::ewm::main_database::qc_collection::camera_qc_collection::CreateCameraCommandError> {
            todo!()
        }
        
        async fn delete_camera_by_id(
            &self,
            id: &str,
        ) -> Result<(), crate::layers::ewm::main_database::qc_collection::camera_qc_collection::DeleteCameraCommandError> {
            todo!()
        }
        
        fn find_camera_by_id(
            &self,
            id: &str,
        ) -> impl std::future::Future<Output = Result<crate::layers::ewm::main_database::qc_collection::camera_qc_collection::FindCameraByIdResult, crate::layers::ewm::main_database::qc_collection::camera_qc_collection::FindCamerabyIdError>> + Send {
            async move {
                Ok(crate::layers::ewm::main_database::qc_collection::camera_qc_collection::FindCameraByIdResult {
                    id: "1".to_string(),
                    name: "Mock Camera".to_string(),
                    source_url: "mock://camera".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    permanent_stream_url: None
                })
            }
        }
        
        fn camera_exists_by_id(
            &self,
            id: &str,
        ) -> impl std::future::Future<Output = Result<bool, crate::layers::ewm::main_database::qc_collection::camera_qc_collection::CheckIfCameraExistsError>> + Send {
            async move {
                Ok(true)
            }
        }
    }

    struct MockCameraTempBlockingQCCollection;

    impl ICameraTempBlockingQCCollection for MockCameraTempBlockingQCCollection {
        async fn list_temp_blocking_for_user(&self, _user_id: &str) -> Result<Vec<CameraTempBlocking>, ListCameraTempBlockingsQueryError> {
            Ok(vec![])
        }

        async fn list_temp_blocking_for_camera(&self, _camera_id: &str) -> Result<Vec<CameraTempBlocking>, ListCameraTempBlockingsQueryError> {
            Ok(vec![])
        }

        async fn create_temp_blocking(&self, _input: crate::layers::ewm::main_database::qc_collection::camera_temp_blocking_qc_collection::CreateCameraTempBlockingInput) -> Result<(), crate::layers::ewm::main_database::qc_collection::camera_temp_blocking_qc_collection::CreateCameraTempBlockingError> {
            Ok(())
        }

        async fn delete_temp_blocking(&self, _camera_id: &str, _user_id: &str) -> Result<(), crate::layers::ewm::main_database::qc_collection::camera_temp_blocking_qc_collection::DeleteCameraTempBlockingError> {
            Ok(())
        }

        async fn get_temp_blocking(&self, _camera_id: &str, _user_id: &str) -> Result<Option<CameraTempBlocking>, crate::layers::ewm::main_database::qc_collection::camera_temp_blocking_qc_collection::GetCameraTempBlockingError> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_list_cameras_success() {
        let mock_collection = MockCameraQCCollection {
            cameras: vec![
                CameraListQueryResultItem { id: 1.to_string(), name: "Camera 1".to_string(), source_url: "something".to_string() },
                CameraListQueryResultItem { id: 2.to_string(), name: "Camera 2".to_string(), source_url: "something".to_string() },
            ],
            should_fail: false,
        };

        let mock_temp_blocking = MockCameraTempBlockingQCCollection;

        let use_case = ListCamerasUseCaseImp {
            camera_qc_collection: mock_collection,
            camera_temp_blocking_qc_collection: mock_temp_blocking,
        };

        let input = ListCamerasInput { user_id: "test_user".to_string() };
        let result = use_case.execute(&input).await;
        assert!(result.is_ok());
        let cameras = result.unwrap();
        assert_eq!(cameras.len(), 2);
        assert_eq!(cameras[0].name, "Camera 1");
        assert_eq!(cameras[1].name, "Camera 2");
    }

    #[tokio::test]
    async fn test_list_cameras_failure() {
        let mock_collection = MockCameraQCCollection {
            cameras: vec![],
            should_fail: true,
        };

        let mock_temp_blocking = MockCameraTempBlockingQCCollection;

        let use_case = ListCamerasUseCaseImp {
            camera_qc_collection: mock_collection,
            camera_temp_blocking_qc_collection: mock_temp_blocking,
        };

        let input = ListCamerasInput { user_id: "test_user".to_string() };
        let result = use_case.execute(&input).await;
        assert!(result.is_err());
        if let UseCaseError::InternalDependencyError(err) = result.unwrap_err() {
            assert_eq!(err.message, "failed to load cameras from database");
        } else {
            panic!("Expected InternalDependencyError");
        }
    }

    #[tokio::test]
    async fn test_list_cameras_empty_result() {
        let mock_collection = MockCameraQCCollection {
            cameras: vec![],
            should_fail: false,
        };

        let mock_temp_blocking = MockCameraTempBlockingQCCollection;

        let use_case = ListCamerasUseCaseImp {
            camera_qc_collection: mock_collection,
            camera_temp_blocking_qc_collection: mock_temp_blocking,
        };

        let input = ListCamerasInput { user_id: "test_user".to_string() };
        let result = use_case.execute(&input).await;
        assert!(result.is_ok());
        let cameras = result.unwrap();
        assert!(cameras.is_empty());
    }
}
