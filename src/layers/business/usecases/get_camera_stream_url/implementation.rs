use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tracing::field::Field;

use crate::layers::{
    business::{
        shared::{
            business_rules::{
                FieldValidationResult, UseCaseInputValidationResult, UseCaseInvalidInputResult,
            },
            errors::{BusinessError, InternalDependencyError, UseCaseError},
            validation_rules::strings::non_empty,
        },
        usecases::get_camera_stream_url::{
            interface::{GetCameraStreamURLOutput, IGetCameraStremaURLUseCase},
            sanitization_rules::GetCameraStreamUrlSanitizedInput,
        },
    },
    ewi::error::UserInputError,
    ewm::{
        main_database::qc_collection::camera_qc_collection::ICameraQCCollection,
        temporary_stream_server::ITemporaryStreamServer,
    },
};


pub struct GetCameraStreamUrlUseCase<IICameraCommandQueryCollection, IITemporaryStreamServer>
where
    IICameraCommandQueryCollection: ICameraQCCollection + Sync,
    IITemporaryStreamServer: ITemporaryStreamServer + Sync,
{
    camera_qc_collection: IICameraCommandQueryCollection,
    temporary_stream_server: IITemporaryStreamServer,
}

impl<IICameraCommandQueryCollection, IITemporaryStreamServer>
    GetCameraStreamUrlUseCase<IICameraCommandQueryCollection, IITemporaryStreamServer>
where
    IICameraCommandQueryCollection: ICameraQCCollection + Sync,
    IITemporaryStreamServer: ITemporaryStreamServer + Sync,
{
    pub fn new(
        camera_qc_collection: IICameraCommandQueryCollection,
        temporary_stream_server: IITemporaryStreamServer,
        
    ) -> Self {
        Self {
            camera_qc_collection,
            temporary_stream_server,
        }
    }

    async fn apply_business_rules(
        &self,
        input: &GetCameraStreamUrlSanitizedInput,
    ) -> Result<UseCaseInputValidationResult, UseCaseError> {
        let fields_validation_result: Vec<FieldValidationResult> = vec![non_empty(
            &input.0,
            "id",
            format!("{} cannot be empty", "id"),
        )];

        let mut feedback: HashMap<String, Vec<String>> = HashMap::new();
        for vr in fields_validation_result {
            if let FieldValidationResult::Invalid(field_name, message) = vr {
                feedback.entry(field_name).or_insert(vec![]).push(message);
            }
        }

        let result = if feedback.is_empty() {
            UseCaseInputValidationResult::Valid
        } else {
            UseCaseInputValidationResult::Invalid(UseCaseInvalidInputResult::new(
                "could not complete operation due to invalid date, please check feedback"
                    .to_string(),
                feedback,
            ))
        };

        Ok(result)
    }

    async fn camera_exists(&self, id: &str) -> Result<FieldValidationResult, UseCaseError> {
        let camera_exists = self
            .camera_qc_collection
            .camera_exists_by_id(id)
            .await
            .map_err(|e| {
                UseCaseError::InternalDependencyError({
                    InternalDependencyError {
                        message: "Failed to check if camera exists in the database".to_owned(),
                        debug_details: e
                            .0
                            .debug_details
                            .unwrap_or("No details provided".to_owned()),
                    }
                })
            })?;

        let result = match camera_exists {
            true => FieldValidationResult::Valid,
            false => {
                let mut field_feedback: HashMap<String, Vec<String>> = HashMap::new();
                field_feedback.insert(
                    "id".to_string(),
                    vec!["the camera should exists in the database".to_owned()],
                );

                FieldValidationResult::Invalid(
                    "id".to_owned(),
                    "camera not found in database".to_owned(),
                )
            }
        };

        Ok(result)
    }
}
impl<IICameraCommandQueryCollection, IITemporaryStreamServer> IGetCameraStremaURLUseCase
    for GetCameraStreamUrlUseCase<IICameraCommandQueryCollection, IITemporaryStreamServer>
where
    IICameraCommandQueryCollection: ICameraQCCollection + Sync,
    IITemporaryStreamServer: ITemporaryStreamServer + Sync,
{
    async fn execute(&self, id: &str) -> Result<GetCameraStreamURLOutput, UseCaseError> {
        let sanitized_input = GetCameraStreamUrlSanitizedInput::try_from(id).map_err(|e| {
            UseCaseError::InternalDependencyError(InternalDependencyError {
                message: "error while sanitizing input".to_string(),
                debug_details: e,
            })
        })?;

        let business_rules_application_result = self
            .apply_business_rules(&sanitized_input)
            .await
            .map_err(|e| e)?;

        if let UseCaseInputValidationResult::Invalid(r) = business_rules_application_result{
            return Err(
                UseCaseError::BusinessError(BusinessError::new(r.message, r.feedback))
            )
        }

        let camera = self
            .camera_qc_collection
            .find_camera_by_id(&sanitized_input.0)
            .await
            .map_err(|e| {
                UseCaseError::InternalDependencyError(InternalDependencyError {
                    message: "Failed to find camera by id in the database".to_owned(),
                    debug_details: e
                        .0
                        .debug_details
                        .unwrap_or("No details provided".to_owned()),
                })
            })?;

        let permanent_url = match camera.permanent_stream_url {
            Some(s) => Ok(s),
            None => Err(UseCaseError::InternalDependencyError(
                InternalDependencyError { message: "Could not get tamporary stream for camera".to_owned(), debug_details: format!("{} no permanent stream found for camera", camera.id) }
            )),
        }?;
        let temporary_stream = self
            .temporary_stream_server
            .get_stream(&camera.id, &permanent_url)
            .await
            .map_err(|e| {
                UseCaseError::InternalDependencyError(InternalDependencyError {
                    message: "Failed to get temporary stream".to_owned(),
                    debug_details: format!("{:?}", e),
                })
            })?;
        let expiration_date = match temporary_stream.expiration_date {
            Some(d) => Ok(d),
            None => {
                let err = UseCaseError::InternalDependencyError(InternalDependencyError {
                    message: "temporary stream server returned a stream with no expiration date"
                        .to_owned(),
                    debug_details: "".to_owned(),
                });
                Err(err)
            }
        }?;
        Ok(
            GetCameraStreamURLOutput { camera_id: camera.id.clone(), temp_rtsp_url: temporary_stream.url, expiration_date }
        )
    }
}
