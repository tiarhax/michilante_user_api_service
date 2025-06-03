use axum::{http, response::{IntoResponse, Response}, Json};
use serde::{de, Deserialize, Serialize};

use crate::layers::business::shared::errors::UseCaseError;

#[derive(Debug)]
pub struct UserInputError {
    pub status_code: http::StatusCode,
    pub message: String,
    pub details: serde_json::Value
}
#[derive(Debug)]
pub struct InternalError {
    pub debug_message: String,
}

#[derive(Debug)]
pub enum AppError {
    UserInputError(UserInputError),
    InternalError(InternalError)
}

#[derive(Serialize, Deserialize)]
pub struct AppErrorJson {
    pub message: String,
    pub details: Option<serde_json::Value>
}

impl AppError {
    pub fn from_use_case_error(error: UseCaseError, http_status_code: Option<http::StatusCode>) -> Self {
        match error {
            UseCaseError::BusinessError(be) =>  {
                let details = serde_json::to_value(be.details).unwrap_or(serde_json::Value::String("could not get any details".to_string()));
                AppError::UserInputError(UserInputError {
                    status_code: http_status_code.unwrap_or(http::StatusCode::BAD_REQUEST) ,
                    message: be.message,
                    details
                })
            },
            UseCaseError::InternalDependencyError(internal_error) => {
                AppError::InternalError(InternalError {
                    debug_message: internal_error.message
                })
            },
        }
    }
}


impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, details) = match self {
            AppError::UserInputError(user_input_error) => {
                (user_input_error.status_code, user_input_error.message, Some(user_input_error.details))
            },
            AppError::InternalError(internal_error) => {
                (http::StatusCode::INTERNAL_SERVER_ERROR, internal_error.debug_message, None)
            },
        };

        let app_error_json = AppErrorJson{
            message,
            details
        };
        let payload = match serde_json::to_value(app_error_json) {
            Ok(v) => Json(v),
            Err(_) => Json(serde_json::Value::String("could not process error".to_string())),
        };
        (status, payload).into_response()
    }
}