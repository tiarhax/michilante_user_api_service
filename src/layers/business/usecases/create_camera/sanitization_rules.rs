use crate::layers::business::shared::sanitization_rules::{pipe_all, strings::{remove_double_spaces, trim_both_sides}};

use super::CreateCameraInput;
#[derive(Debug)]
pub struct CreateCameraSanitizedInput(pub CreateCameraInput);


impl TryFrom<CreateCameraInput> for CreateCameraSanitizedInput {
    type Error = String;
    
    fn try_from(value: CreateCameraInput) -> Result<Self, Self::Error> {
        let inner = CreateCameraInput {
            name: pipe_all( vec![
                trim_both_sides,
                remove_double_spaces
            ], &value.name)?,
            source_url: pipe_all(vec![
                trim_both_sides,
                remove_double_spaces
            ], &value.source_url)?
        };

        Ok(CreateCameraSanitizedInput(inner))
    }
}