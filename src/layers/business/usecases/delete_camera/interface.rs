use crate::layers::business::shared::errors::UseCaseError;

pub trait IDeleteCameraUseCase {
    fn execute(&self, id: String) -> impl std::future::Future<Output = Result<(), UseCaseError>> + Send;
}

