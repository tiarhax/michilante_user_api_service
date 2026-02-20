use axum::{
    extract::State,
    routing::get,
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::layers::{
    business::usecases::v2::list_cameras::{
        implementation::ListCamerasUseCaseImp,
        interface::{CameraAvailability, CameraListItem, IListCamerasUseCase, ListCamerasInput},
    },
    ewi::{appstate::AppState, error::AppError, middleware::auth0::Claims},
    ewm::main_database::qc_collection::{
        camera_qc_collection::CameraQCCollection,
        camera_temp_blocking_qc_collection::CameraTempBlockingQCCollection,
    },
};

#[derive(Serialize, Deserialize)]
pub struct CameraResultItemV2 {
    id: String,
    name: String,
    source_url: String,
    is_available: bool,
    available_at: Option<String>,
}

impl From<CameraListItem> for CameraResultItemV2 {
    fn from(item: CameraListItem) -> Self {
        let (is_available, available_at) = match item.is_available {
            CameraAvailability::Available => (true, None),
            CameraAvailability::NotAvailable(end_date) => (false, Some(end_date)),
        };
        CameraResultItemV2 {
            id: item.id,
            name: item.name,
            source_url: item.source_url,
            is_available,
            available_at,
        }
    }
}

pub async fn list_cameras(
    State(camera_qc_collection): State<CameraQCCollection>,
    State(camera_temp_blocking_qc_collection): State<CameraTempBlockingQCCollection>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<CameraResultItemV2>>, AppError> {
    let list_cameras_use_case =
        ListCamerasUseCaseImp::new(camera_qc_collection, camera_temp_blocking_qc_collection);

    let input = ListCamerasInput {
        user_id: claims.sub,
    };

    let cameras = list_cameras_use_case
        .execute(&input)
        .await
        .map_err(|err| AppError::from_use_case_error(err, None))?;

    let result = cameras
        .into_iter()
        .map(|c| c.into())
        .collect::<Vec<CameraResultItemV2>>();

    Ok(Json(result))
}

pub fn setup_endpoints(router: Router<AppState>) -> Router<AppState> {
    router.route("/v2/cameras", get(list_cameras))
}
