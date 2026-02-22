use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::layers::{
    business::usecases::{
        create_camera::{CreateCameraInput, CreateCameraOutput, CreateCameraUseCase},
        create_camera_temp_blocking::{
            implementation::CreateCameraTempBlockingUseCaseImp,
            interface::{CreateCameraTempBlockingInput, ICreateCameraTempBlockingUseCase},
        },
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
    ewi::{appstate::{auth0::User, AppState}, error::AppError},
    ewm::{
        main_database::qc_collection::{
            camera_qc_collection::CameraQCCollection,
            camera_temp_blocking_qc_collection::CameraTempBlockingQCCollection,
        },
        permanent_stream_server::PermanentStreamServer,
        temporary_stream_server::TemporaryStreamServer,
    },
};

#[derive(Serialize, Deserialize, ToSchema)]
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

#[utoipa::path(
    get,
    path = "/cameras",
    tag = "cameras",
    responses(
        (status = 200, description = "List all cameras", body = Vec<CameraResultItem>)
    )
)]
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

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CameraCreationHTTPResponseBody {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Deserialize, ToSchema)]
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
#[utoipa::path(
    post,
    path = "/cameras",
    tag = "cameras",
    request_body = CreateCameraHttpInput,
    responses(
        (status = 200, description = "Camera created successfully", body = CameraCreationHTTPResponseBody)
    )
)]
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

#[derive(Deserialize, ToSchema)]
pub struct UpdateCameraHttpInput {
    pub name: String,
    pub source_url: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
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

#[utoipa::path(
    put,
    path = "/cameras/{id}",
    tag = "cameras",
    params(
        ("id" = String, Path, description = "Camera ID")
    ),
    request_body = UpdateCameraHttpInput,
    responses(
        (status = 200, description = "Camera updated successfully", body = CameraUpdateHTTPResponseBody)
    )
)]
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

#[utoipa::path(
    delete,
    path = "/cameras/{id}",
    tag = "cameras",
    params(
        ("id" = String, Path, description = "Camera ID")
    ),
    responses(
        (status = 200, description = "Camera deleted successfully")
    )
)]
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

#[derive(Serialize, ToSchema)]
pub struct CameraStreamHttpResponseBody {
    camera_id: String,
    temp_rtsp_url: String,
    expiration_date: String,
}
#[utoipa::path(
    get,
    path = "/cameras/{id}/temp-stream",
    tag = "cameras",
    params(
        ("id" = String, Path, description = "Camera ID")
    ),
    responses(
        (status = 200, description = "Temporary stream URL", body = CameraStreamHttpResponseBody)
    )
)]
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

#[derive(Deserialize, ToSchema)]
pub struct CreateCameraTempBlockingHttpInput {
    pub camera_id: String,
    pub start_time: String,
    pub end_time: String,
    pub user_ids: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/cameras/temp-blocking",
    tag = "cameras",
    request_body = CreateCameraTempBlockingHttpInput,
    responses(
        (status = 200, description = "Camera temp blocking created successfully"),
        (status = 403, description = "Forbidden - Admin role required")
    )
)]
pub async fn create_camera_temp_blocking(
    State(camera_temp_blocking_qc_collection): State<CameraTempBlockingQCCollection>,
    user: User,
    Json(input): Json<CreateCameraTempBlockingHttpInput>,
) -> Result<(), AppError> {
    if !user.roles.contains(&"Admin".to_string()) {
        return Err(AppError::Forbidden("Admin role required".to_string()));
    }

    let use_case = CreateCameraTempBlockingUseCaseImp::new(camera_temp_blocking_qc_collection);

    let use_case_input = CreateCameraTempBlockingInput {
        camera_id: input.camera_id,
        start_time: input.start_time,
        end_time: input.end_time,
        user_ids: input.user_ids,
    };

    use_case
        .execute(use_case_input)
        .await
        .map_err(|err| AppError::from_use_case_error(err, None))?;

    Ok(())
}

pub fn setup_endpoints(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/cameras", get(list_cameras))
        .route("/cameras", post(create_camera))
        .route("/cameras/{id}", put(put_camera))
        .route("/cameras/{id}", delete(delete_camera))
        .route("/cameras/{id}/temp-stream", get(get_camera_stream_url))
        .route("/cameras/temp-blocking", post(create_camera_temp_blocking))
}
