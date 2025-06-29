use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::layers::{
    business::usecases::{
        create_camera::{CreateCameraInput, CreateCameraOutput, CreateCameraUseCase},
        delete_camera::{implementation::DeleteCameraUseCase, interface::IDeleteCameraUseCase},
        get_camera_stream_url::{
            implementation::GetCameraStreamUrlUseCase, interface::IGetCameraStremaURLUseCase,
        },
        list_cameras::{
            implementation::ListCamerasUseCaseImp,
            interface::{CameraListItem, IListCamerasUseCase},
        },
        put_camera::{
            implementation::PutCameraUseCase,
            interface::{IPutCameraUseCase, PutCameraInput, PutCameraOutput},
        },
    },
    ewi::{appstate::AppState, error::AppError, providers::permanent_stream_server},
    ewm::{
        main_database::qc_collection::camera_qc_collection::CameraQCCollection,
        permanent_stream_server::PermanentStreamServer,
        temporary_stream_server::TemporaryStreamServer,
    },
};

#[derive(Serialize, Deserialize)]
pub struct CameraResultItem {
    id: String,
    name: String,
    source_url: String,
}

impl From<CameraListItem> for CameraResultItem {
    fn from(item: CameraListItem) -> Self {
        CameraResultItem {
            id: item.id,
            name: item.name,
            source_url: item.source_url,
        }
    }
}

pub async fn list_cameras(
    State(camera_qc_collection): State<CameraQCCollection>,
) -> Result<Json<Vec<CameraResultItem>>, AppError> {
    let list_cameras_use_case = ListCamerasUseCaseImp::new(camera_qc_collection);

    let cameras = list_cameras_use_case
        .execute()
        .await
        .map_err(|err| AppError::from_use_case_error(err, None))?;

    let result = cameras
        .into_iter()
        .map(|c| c.into())
        .collect::<Vec<CameraResultItem>>();

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
        Self {
            id: output.id,
            name: output.name,
            source_url: output.source_url,
            updated_at: output.updated_at.to_rfc3339(),
            created_at: output.created_at.to_rfc3339(),
        }
    }
}
pub async fn create_camera(
    State(camera_qc_collection): State<CameraQCCollection>,
    State(permanent_stream_server): State<PermanentStreamServer>,
    Json(input): Json<CreateCameraHttpInput>,
) -> Result<Json<CameraCreationHTTPResponseBody>, AppError> {
    let create_camera_use_case =
        CreateCameraUseCase::new(camera_qc_collection, permanent_stream_server);
    let result = create_camera_use_case
        .execute(input.into())
        .await
        .map_err(|err| AppError::from_use_case_error(err, None))?;

    Ok(Json(result.into()))
}

#[derive(Deserialize)]
pub struct UpdateCameraHttpInput {
    pub name: String,
    pub source_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct CameraUpdateHTTPResponseBody {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<PutCameraOutput> for CameraUpdateHTTPResponseBody {
    fn from(value: PutCameraOutput) -> Self {
        CameraUpdateHTTPResponseBody {
            id: value.id,
            name: value.name,
            source_url: value.source_url,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

pub async fn put_camera(
    Path(id): Path<String>,
    State(camera_qc_collection): State<CameraQCCollection>,
    State(permanent_stream_server): State<PermanentStreamServer>,
    Json(input): Json<UpdateCameraHttpInput>,
) -> Result<Json<CameraUpdateHTTPResponseBody>, AppError> {
    let update_camera_use_case =
        PutCameraUseCase::new(camera_qc_collection, permanent_stream_server);
    tracing::info!("id received from path {}", id);

    let use_case_in = PutCameraInput {
        id,
        name: input.name,
        source_url: input.source_url,
    };
    let use_case_out = update_camera_use_case
        .execute(use_case_in)
        .await
        .map_err(|err| AppError::from_use_case_error(err, None))?;

    Ok(Json(use_case_out.into()))
}

pub async fn delete_camera(
    Path(id): Path<String>,
    State(camera_qc_collection): State<CameraQCCollection>,
    State(permanent_stream_server): State<PermanentStreamServer>,
) -> Result<(), AppError> {
    let delete_camera_use_case =
        DeleteCameraUseCase::new(camera_qc_collection, permanent_stream_server);
    delete_camera_use_case
        .execute(id)
        .await
        .map_err(|err| AppError::from_use_case_error(err, None))?;
    Ok(())
}

#[derive(Serialize)]
pub struct CameraStreamHttpResponseBody {
    camera_id: String,
    temp_rtsp_url: String,
    expiration_date: String,
}
pub async fn get_camera_stream_url(
    Path(id): Path<String>,
    State(camera_qc_collection): State<CameraQCCollection>,
    State(temporary_stream_server): State<TemporaryStreamServer>,
) -> Result<Json<CameraStreamHttpResponseBody>, AppError> {
    let get_stream_url_use_case =
        GetCameraStreamUrlUseCase::new(camera_qc_collection, temporary_stream_server);
    let out = get_stream_url_use_case
        .execute(&id)
        .await
        .map_err(|err| AppError::from_use_case_error(err, None))?;
    Ok(Json(CameraStreamHttpResponseBody {
        camera_id: out.camera_id,
        temp_rtsp_url: out.temp_rtsp_url,
        expiration_date: out.expiration_date.to_rfc3339(),
    }))
}

pub fn setup_endpoints(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/cameras", get(list_cameras))
        .route("/cameras", post(create_camera))
        .route("/cameras/{id}", put(put_camera))
        .route("/cameras/{id}", delete(delete_camera))
        .route("/cameras/{id}/temp-stream", get(get_camera_stream_url))
}
