use axum::{extract::State, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};

use crate::layers::{business::usecases::{create_camera::{CreateCameraInput, CreateCameraOutput, CreateCameraUseCase}, list_cameras::{implementation::ListCamerasUseCaseImp, interface::{CameraListItem, IListCamerasUseCase}}}, ewi::{appstate::AppState, error::AppError, providers::permanent_stream_server}, ewm::{main_database::qc_collection::camera_qc_collection::CameraQCCollection, permanent_stream_server::PermanentStreamServer}};

#[derive(Serialize, Deserialize)]
pub struct CameraResultItem {
    id: String,
    name: String
}

impl From<CameraListItem> for CameraResultItem {
    fn from(item: CameraListItem) -> Self {
        CameraResultItem {
            id: item.id,
            name: item.name,
        }
    }
}

pub async fn list_cameras(State(camera_qc_collection): State<CameraQCCollection>) -> Result<Json<Vec<CameraResultItem>>, AppError> {

    let list_cameras_use_case = ListCamerasUseCaseImp::new(camera_qc_collection);

    let cameras = list_cameras_use_case.execute().await.map_err(|err| {
        AppError::from_use_case_error(err, None)
    })?;

    let result = cameras.into_iter().map(|c| c.into()).collect::<Vec<CameraResultItem>>();

    Ok(Json(result))
}

#[derive(Serialize, Deserialize)]
pub struct CameraCreationHTTPResponseBody {
    id: String,
    name: String,
}
#[derive(Deserialize)]
pub struct CreateCameraHttpInput {
    pub name: String,
    pub source_url: String,
}

impl Into<CreateCameraInput> for CreateCameraHttpInput {
    fn into(self) -> CreateCameraInput {
        CreateCameraInput {
            name: self.name,
            source_url: self.source_url,
        }
    }
}

impl From<CreateCameraOutput> for CameraCreationHTTPResponseBody {
    fn from(output: CreateCameraOutput) -> Self {
        CameraCreationHTTPResponseBody {
            id: output.id,
            name: output.name,
        }
    }
}
pub async fn create_camera(
    State(camera_qc_collection): State<CameraQCCollection>, 
    State(permanent_stream_server): State<PermanentStreamServer>,
    Json(input): Json<CreateCameraHttpInput>
) -> Result<Json<CameraCreationHTTPResponseBody>, AppError> {
    let create_camera_use_case = CreateCameraUseCase::new(camera_qc_collection, permanent_stream_server);
    let result = create_camera_use_case.execute(input.into()).await.map_err(|err| {
        AppError::from_use_case_error(err, None)
    })?;

    Ok(Json(result.into()))
}


pub fn setup_endpoints(router: Router<AppState>) -> Router<AppState> {
    router.route("/cameras", get(list_cameras))
        .route("/cameras", post(create_camera))
}