use crate::layers::business::shared::sanitization_rules::{pipe_all, strings::{remove_double_spaces, trim_both_sides}};

use super::UpsertCameraInput;
#[derive(Debug)]
pub struct UpsertCameraSanitizedInput(pub UpsertCameraInput);


impl TryFrom<UpsertCameraInput> for UpsertCameraSanitizedInput {
    type Error = String;
    
    fn try_from(value: UpsertCameraInput) -> Result<Self, Self::Error> {
        let inner = UpsertCameraInput {
            name: pipe_all( vec![
                trim_both_sides,
                remove_double_spaces
            ], &value.name)?,
            source_url: pipe_all(vec![
                trim_both_sides,
                remove_double_spaces
            ], &value.source_url)?
        };

        Ok(UpsertCameraSanitizedInput(inner))
    }
}