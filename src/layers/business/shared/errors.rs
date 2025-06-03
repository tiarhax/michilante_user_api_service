use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BusinessError {
    pub message: String,
    pub details: HashMap<String, Vec<String>>
}

impl BusinessError {
    pub fn new(message: String, details: HashMap<String, Vec<String>>) -> Self {
        Self {
            message,
            details,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InternalDependencyError {
    pub message: String,
    pub debug_details: String
}

impl InternalDependencyError {
    pub fn new(message: String, debug_details: String) -> Self {
        Self {
            message,
            debug_details,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UseCaseError {
    BusinessError(BusinessError),
    InternalDependencyError(InternalDependencyError)
}