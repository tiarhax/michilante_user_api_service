use axum::{extract::State, routing::{get, post, put}, Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::layers::{business::usecases::{upsert_camera::{UpsertCameraInput, PutCameraOutput, UpsertCameraUseCase}, list_cameras::{implementation::ListCamerasUseCaseImp, interface::{CameraListItem, IListCamerasUseCase}}}, ewi::{appstate::AppState, error::AppError, providers::permanent_stream_server}, ewm::{main_database::qc_collection::camera_qc_collection::CameraQCCollection, permanent_stream_server::PermanentStreamServer}};

#[derive(Serialize, Deserialize)]
pub struct CameraResultItem {
    id: String,
    name: String,
    source_url: String
}

impl From<CameraListItem> for CameraResultItem {
    fn from(item: CameraListItem) -> Self {
        CameraResultItem {
            id: item.id,
            name: item.name,
            source_url: item.source_url
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
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Deserialize)]
pub struct CreateCameraHttpInput {
    pub name: String,
    pub source_url: String,
}

impl Into<UpsertCameraInput> for CreateCameraHttpInput {
    fn into(self) -> UpsertCameraInput {
        UpsertCameraInput {
            name: self.name,
            source_url: self.source_url,
        }
    }
}

impl From<PutCameraOutput> for CameraCreationHTTPResponseBody {
    fn from(output: PutCameraOutput) -> Self {
        CameraCreationHTTPResponseBody {
            id: output.id,
            name: output.name,
            source_url: output.source_url,
            updated_at: output.updated_at.to_rfc3339(),
            created_at: output.created_at.to_rfc3339()
        }
    }
}
pub async fn put_camera(
    State(camera_qc_collection): State<CameraQCCollection>, 
    State(permanent_stream_server): State<PermanentStreamServer>,
    Json(input): Json<CreateCameraHttpInput>
) -> Result<Json<CameraCreationHTTPResponseBody>, AppError> {
    let create_camera_use_case = UpsertCameraUseCase::new(camera_qc_collection, permanent_stream_server);
    let result = create_camera_use_case.execute(input.into()).await.map_err(|err| {
        AppError::from_use_case_error(err, None)
    })?;

    Ok(Json(result.into()))
}


pub fn setup_endpoints(router: Router<AppState>) -> Router<AppState> {
    router.route("/cameras", get(list_cameras))
        .route("/cameras", put(put_camera))
}