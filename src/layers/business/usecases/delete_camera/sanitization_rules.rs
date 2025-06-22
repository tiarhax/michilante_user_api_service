use crate::layers::business::{shared::sanitization_rules::{pipe_all, strings::{remove_double_spaces, trim_both_sides}}, usecases::put_camera::interface::PutCameraInput};

#[derive(Debug)]
pub struct DeleteCameraSanitizedInput(pub String);

impl TryFrom<String> for DeleteCameraSanitizedInput {
    type Error = String;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let inner = pipe_all( vec![
            trim_both_sides,
            remove_double_spaces
        ], &value)?;

        Ok(Self(inner))
    }
}