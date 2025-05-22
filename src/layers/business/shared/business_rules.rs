use std::collections::HashMap;

pub struct UseCaseInvalidInputResult {
    pub message: String,
    pub feedback: HashMap<String, Vec<String>>
}

impl UseCaseInvalidInputResult {
    pub fn new(message: String, feedback: HashMap<String, Vec<String>>) -> Self {
        Self { message, feedback }
    }
}
pub enum UseCaseInputValidationResult {
    Valid,
    Invalid(UseCaseInvalidInputResult)
}

pub enum FieldValidationResult {
    Valid,
    Invalid(String, String)
}

pub enum GeneralValidationResult {
    Valid,
    Invalid(String)
}