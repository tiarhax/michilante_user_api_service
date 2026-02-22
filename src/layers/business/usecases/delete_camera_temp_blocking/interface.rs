use crate::layers::business::shared::errors::UseCaseError;

pub struct DeleteCameraTempBlockingInput {
    pub camera_id: String,
    pub user_id: String,
}

pub trait IDeleteCameraTempBlockingUseCase {
    fn execute(
        &self,
        input: DeleteCameraTempBlockingInput,
    ) -> impl std::future::Future<Output = Result<(), UseCaseError>> + Send;
}
