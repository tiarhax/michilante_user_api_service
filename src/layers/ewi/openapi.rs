use utoipa::OpenApi;

use super::endpoints::cameras::{
    BlockedUserHttpResponse, CameraCreationHTTPResponseBody, CameraResultItem,
    CameraStreamHttpResponseBody, CameraTempBlockingHttpResponseItem,
    CameraUpdateHTTPResponseBody, CreateCameraHttpInput, CreateCameraTempBlockingHttpInput,
    UpdateCameraHttpInput,
};
use super::endpoints::camerasv2::CameraResultItemV2;
use super::endpoints::users::UserResultItem;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "User API Server",
        version = "1.0.0",
        description = "API for managing cameras and streams"
    ),
    paths(
        super::endpoints::cameras::list_cameras,
        super::endpoints::cameras::create_camera,
        super::endpoints::cameras::put_camera,
        super::endpoints::cameras::delete_camera,
        super::endpoints::cameras::get_camera_stream_url,
        super::endpoints::cameras::create_camera_temp_blocking,
        super::endpoints::cameras::list_camera_temp_blockings_by_camera,
        super::endpoints::cameras::delete_camera_temp_blocking,
        super::endpoints::camerasv2::list_cameras_v2,
        super::endpoints::users::list_users,
    ),
    components(
        schemas(
            CameraResultItem,
            CameraCreationHTTPResponseBody,
            CreateCameraHttpInput,
            UpdateCameraHttpInput,
            CameraUpdateHTTPResponseBody,
            CameraStreamHttpResponseBody,
            CameraResultItemV2,
            CreateCameraTempBlockingHttpInput,
            CameraTempBlockingHttpResponseItem,
            BlockedUserHttpResponse,
            UserResultItem,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "cameras", description = "Camera management endpoints"),
        (name = "cameras-v2", description = "Camera management endpoints v2"),
        (name = "users", description = "User management endpoints")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(utoipa::openapi::security::HttpAuthScheme::Bearer)
                ),
            );
        }
        openapi.security = Some(vec![
            utoipa::openapi::security::SecurityRequirement::new::<&str, [&str; 0], &str>("bearer_auth", [])
        ]);
    }
}
