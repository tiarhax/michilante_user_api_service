use crate::layers::business::shared::errors::UseCaseError;

pub struct CreateCameraTempBlockingInput {
    pub camera_id: String,
    pub start_time: String,
    pub end_time: String,
    pub user_ids: Vec<String>,
}

pub trait ICreateCameraTempBlockingUseCase {
    fn execute(
        &self,
        input: CreateCameraTempBlockingInput,
    ) -> impl std::future::Future<Output = Result<(), UseCaseError>> + Send;
}
