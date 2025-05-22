#[derive(Clone, Debug)]
pub struct QCError {
    pub message: String,
    pub debug_details: Option<String>
}

impl QCError {
    pub fn new(message: String, debug_details: Option<String>) -> Self {
        QCError { message, debug_details }
    }
}