use std::collections::HashMap;

use crate::layers::{
    business::{
        shared::{
            business_rules::{
                FieldValidationResult, UseCaseInputValidationResult, UseCaseInvalidInputResult,
            },
            errors::{BusinessError, InternalDependencyError, UseCaseError},
            validation_rules::{rtsp_url::rtsp_url, strings::non_empty},
        },
        usecases::put_camera::{
            interface::{IPutCameraUseCase, PutCameraInput, PutCameraOutput},
            sanitization_rules::PutCameraSanitizedInput,
        },
    },
    ewm::{
        main_database::qc_collection::camera_qc_collection::{
            ICameraQCCollection, PutCameraCommandInput,
        },
        permanent_stream_server::IPermanentStreamServer,
    },
};

pub struct PutCameraUseCase<IICamercaQCCollection, IIPermanentStreamServer>
where
    IICamercaQCCollection: ICameraQCCollection,
    IIPermanentStreamServer: IPermanentStreamServer,
{
    pub camera_qc_collection: IICamercaQCCollection,
    pub permanent_stream_server: IIPermanentStreamServer,
}

impl<IICamercaQCCollection, IIPermanentStreamServer>
    PutCameraUseCase<IICamercaQCCollection, IIPermanentStreamServer>
where
    IICamercaQCCollection: ICameraQCCollection,
    IIPermanentStreamServer: IPermanentStreamServer,
{
    pub fn new(
        camera_qc_collection: IICamercaQCCollection,
        permanent_stream_server: IIPermanentStreamServer,
    ) -> Self {
        Self {
            camera_qc_collection,
            permanent_stream_server,
        }
    }

    async fn apply_business_rules(
        &self,
        input: &PutCameraSanitizedInput,
    ) -> UseCaseInputValidationResult {
        tracing::info!("{:?}", input.0);
        let fields_validation_result: Vec<FieldValidationResult> = vec![
            non_empty(&input.0.id, "id", format!("{} cannot be empty", "id")),
            non_empty(&input.0.name, "name", format!("{} cannot be empty", "name")),
            non_empty(
                &input.0.source_url,
                "source_url",
                format!("{} cannot be empty", "source_url"),
            ),
            rtsp_url(
                &input.0.source_url,
                "source_url",
                "must be a valid rtmp url",
            ),
        ];
        let mut feedback: HashMap<String, Vec<String>> = HashMap::new();
        for vr in fields_validation_result {
            if let FieldValidationResult::Invalid(field_name, message) = vr {
                feedback.entry(field_name).or_insert(vec![]).push(message);
            }
        }

        if feedback.is_empty() {
            UseCaseInputValidationResult::Valid
        } else {
            UseCaseInputValidationResult::Invalid(UseCaseInvalidInputResult::new(
                "could not complete operation due to invalid date, please check feedback"
                    .to_string(),
                feedback,
            ))
        }
    }
}

impl<IICamercaQCCollection, IIPermanentStreamServer> IPutCameraUseCase
    for PutCameraUseCase<IICamercaQCCollection, IIPermanentStreamServer>
where
    IICamercaQCCollection: ICameraQCCollection + Sync,
    IIPermanentStreamServer: IPermanentStreamServer + Sync,
{
    async fn execute(&self, input: PutCameraInput) -> Result<PutCameraOutput, UseCaseError> {
        let sanitized_input: PutCameraSanitizedInput = input.try_into().map_err(|err| {
            UseCaseError::InternalDependencyError(InternalDependencyError::new(
                "Failed to sanitize input".to_owned(),
                err,
            ))
        })?;

        let business_rules_result = self.apply_business_rules(&sanitized_input).await;
        if let UseCaseInputValidationResult::Invalid(invalid_result) = business_rules_result {
            return Err(UseCaseError::BusinessError(BusinessError::new(
                invalid_result.message,
                invalid_result.feedback,
            )));
        }

        let current_camera_state = self
            .camera_qc_collection
            .find_camera_by_id(&sanitized_input.0.id)
            .await
            .map_err(|e| {
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "Failed to find camera in database".to_owned(),
                    format!("{:?}", e),
                ))
            })?;

        let update_camera_command = PutCameraCommandInput {
            id: Some(sanitized_input.0.id),
            name: sanitized_input.0.name,
            source_url: sanitized_input.0.source_url,
            permanent_stream_url: current_camera_state.permanent_stream_url,
        };

        let update_camera_command_result = self
            .camera_qc_collection
            .put_camera(update_camera_command)
            .await
            .map_err(|e| {
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "Failed to update camera in database".to_owned(),
                    format!("{:?}", e),
                ))
            })?;

        let put_camera_output = PutCameraOutput {
            id: update_camera_command_result.id,
            name: update_camera_command_result.name,
            source_url: update_camera_command_result.source_url,
            created_at: update_camera_command_result.created_at,
            updated_at: update_camera_command_result.updated_at,
        };

        Ok(put_camera_output)
    }
}
