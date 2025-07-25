use crate::layers::{business::shared::errors::{InternalDependencyError, UseCaseError}, ewm::main_database::qc_collection::camera_qc_collection::{ ICameraQCCollection }};

use super::interface::{CameraListItem, IListCamerasUseCase};

pub struct ListCamerasUseCaseImp<IICameraCommandQueryCollection> 
where IICameraCommandQueryCollection: ICameraQCCollection
{
    camera_qc_collection: IICameraCommandQueryCollection
}

impl<IICameraCommandQueryCollection> ListCamerasUseCaseImp<IICameraCommandQueryCollection>
where
    IICameraCommandQueryCollection: ICameraQCCollection + Sync,
{
    pub fn new(camera_qc_collection: IICameraCommandQueryCollection) -> Self {
        Self { camera_qc_collection }
    }
}

impl<IICameraCommandQueryCollection> IListCamerasUseCase for ListCamerasUseCaseImp<IICameraCommandQueryCollection>
    where IICameraCommandQueryCollection: ICameraQCCollection + Sync
{
    async fn execute(&self) -> Result<Vec<CameraListItem>, UseCaseError> {
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
        let results = query_results
            .into_iter()
            .map(|s| {
                CameraListItem {
                    id: s.id,
                    name: s.name,
                    source_url: s.source_url
                }
            })
            .collect();
        Ok(results)
    }
}
#[cfg(test)]
mod tests {

    use crate::layers::ewm::main_database::qc_collection::{camera_qc_collection::{CameraListQueryResultItem, ListCamerasQueryError}, error::QCError};

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

    #[tokio::test]
    async fn test_list_cameras_success() {
        let mock_collection = MockCameraQCCollection {
            cameras: vec![
                CameraListQueryResultItem { id: 1.to_string(), name: "Camera 1".to_string(), source_url: "something".to_string() },
                CameraListQueryResultItem { id: 2.to_string(), name: "Camera 2".to_string(), source_url: "something".to_string() },
            ],
            should_fail: false,
        };

        let use_case = ListCamerasUseCaseImp {
            camera_qc_collection: mock_collection,
        };

        let result = use_case.execute().await;
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

        let use_case = ListCamerasUseCaseImp {
            camera_qc_collection: mock_collection,
        };

        let result = use_case.execute().await;
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

        let use_case = ListCamerasUseCaseImp {
            camera_qc_collection: mock_collection,
        };

        let result = use_case.execute().await;
        assert!(result.is_ok());
        let cameras = result.unwrap();
        assert!(cameras.is_empty());
    }
}
