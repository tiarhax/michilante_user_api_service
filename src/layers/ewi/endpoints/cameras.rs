use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::layers::{business::{shared::errors::{BusinessError, InternalDependencyError}, usecases::list_cameras::{implementation::ListCamerasUseCaseImp, interface::{CameraListItem, IListCamerasUseCase}}}, ewi::{appstate::{AppConfig, AppState}, error::AppError}, ewm::main_database::qc_collection::camera_qc_collection::CameraQCCollection};

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


pub fn setup_endpoints(router: Router<AppState>) -> Router<AppState> {
    router.route("/cameras", get(list_cameras))
}