use crate::layers::business::{shared::sanitization_rules::{pipe_all, strings::{remove_double_spaces, trim_both_sides}}, usecases::put_camera::interface::PutCameraInput};

#[derive(Debug)]
pub struct GetCameraStreamUrlSanitizedInput(pub String);

impl TryFrom<&str> for GetCameraStreamUrlSanitizedInput {
    type Error = String;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let v = value.to_owned();
        let inner: String = pipe_all(vec![
            trim_both_sides,
        ], &v).map_err(|e| e.to_string())?;
        
        Ok(GetCameraStreamUrlSanitizedInput(inner))
    }
}