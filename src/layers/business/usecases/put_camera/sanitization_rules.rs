use crate::layers::business::{shared::sanitization_rules::{pipe_all, strings::{remove_double_spaces, trim_both_sides}}, usecases::put_camera::interface::PutCameraInput};

#[derive(Debug)]
pub struct PutCameraSanitizedInput(pub PutCameraInput);

impl TryFrom<PutCameraInput> for PutCameraSanitizedInput {
    type Error = String;
    
    fn try_from(value: PutCameraInput) -> Result<Self, Self::Error> {
        let inner = PutCameraInput {
            id: pipe_all( vec![
                trim_both_sides,
                remove_double_spaces
            ], &value.id)?,
            name: pipe_all( vec![
                trim_both_sides,
                remove_double_spaces
            ], &value.name)?,
            source_url: pipe_all(vec![
                trim_both_sides,
                remove_double_spaces
            ], &value.source_url)?
        };

        Ok(Self(inner))
    }
}