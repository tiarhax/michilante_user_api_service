pub mod sanitization_rules;
use std::collections::HashMap;

use chrono::Utc;
use sanitization_rules::CreateCameraSanitizedInput;

use crate::layers::{
    business::shared::{
        business_rules::{
            FieldValidationResult, UseCaseInputValidationResult, UseCaseInvalidInputResult,
        },
        errors::{BusinessError, InternalDependencyError, UseCaseError},
        validation_rules::{rtsp_url::rtsp_url, strings::non_empty},
    },
    ewm::{main_database::qc_collection::camera_qc_collection::{
        CreateCameraCommandInput, ICameraQCCollection,
    }, permanent_stream_server::{AddStreamInput, IPermanentStreamServer}},
};
#[derive(Debug)]
pub struct CreateCameraInput {
    pub name: String,
    pub source_url: String,
}

pub struct CreateCameraOutput {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub trait ICreateCameraUseCase {
    fn execute(
        input: CreateCameraInput,
    ) -> impl std::future::Future<Output = Result<CreateCameraOutput, UseCaseError>> + Send;
}

pub struct CreateCameraUseCase<IICamercaQCCollection, IIPermanentStreamServer>
where
    IICamercaQCCollection: ICameraQCCollection,
    IIPermanentStreamServer: IPermanentStreamServer
{
    camera_qc_collection: IICamercaQCCollection,
    permanent_stream_server: IIPermanentStreamServer
}

impl<IICamercaQCCollection, IIPermanentStreamServer> CreateCameraUseCase<IICamercaQCCollection, IIPermanentStreamServer>
where
    IICamercaQCCollection: ICameraQCCollection,
    IIPermanentStreamServer: IPermanentStreamServer
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
}

impl<IICamercaQCCollection, IIPermanentStreamServer> CreateCameraUseCase<IICamercaQCCollection, IIPermanentStreamServer>
where
    IICamercaQCCollection: ICameraQCCollection,
    IIPermanentStreamServer: IPermanentStreamServer
{
    pub async fn execute(
        &self,
        input: CreateCameraInput,
    ) -> Result<CreateCameraOutput, UseCaseError> {
        let sanitized_input: CreateCameraSanitizedInput = input.try_into().map_err(|err| {
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

        let creation_command = CreateCameraCommandInput {
            name: sanitized_input.0.name,
            source_url: sanitized_input.0.source_url,
        };

        let camera_command_result = self
            .camera_qc_collection
            .create_camera(creation_command)
            .await
            .map_err(|err| {
                UseCaseError::InternalDependencyError(InternalDependencyError::new(
                    "Failed to create camera".to_owned(),
                    format!("{:?}", err),
                ))
            })?;
        let id =  ulid::Ulid::new().to_string();
        let add_stream_request = AddStreamInput {
            id,
            name: camera_command_result.id.clone(),
            url: camera_command_result.source_url.clone(),
        };
        self.permanent_stream_server.put_stream(add_stream_request).await.map_err(|err| {
            let debug_message = format!("{:?}", err);
            tracing::error!("Failed to add permanent stream: {}", debug_message);
            UseCaseError::InternalDependencyError(InternalDependencyError::new(
                "Failed to add permanent stream".to_owned(),
                debug_message,
            ))
        })?;
        let create_camera_output = CreateCameraOutput {
            id: camera_command_result.id,
            name: camera_command_result.name,
            source_url: camera_command_result.source_url,
            created_at: camera_command_result.created_at,
            updated_at: camera_command_result.updated_at,
        };  
        Ok(create_camera_output)
    }

    async fn apply_business_rules(
        &self,
        input: &CreateCameraSanitizedInput,
    ) -> UseCaseInputValidationResult {
        tracing::info!("{:?}", input.0);
        let fields_validation_result: Vec<FieldValidationResult> = vec![
            non_empty(&input.0.name, "name", format!("{} cannot be empty", "name")),
            non_empty(
                &input.0.source_url,
                "source_url",
                format!("{} cannot be empty", "source_url"),
            ),
            rtsp_url(&input.0.source_url , "source_url", "must be a valid rtmp url")
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
