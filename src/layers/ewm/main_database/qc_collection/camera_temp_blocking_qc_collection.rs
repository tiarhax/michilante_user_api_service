use aws_sdk_dynamodb::types::AttributeValue;

use crate::layers::ewm::main_database::qc_collection::error::QCError;

pub struct CameraTempBlocking {
    pub id: String,
    pub camera_id: String,
    pub end_date: String
}

pub struct CreateCameraTempBlockingInput {
    pub camera_id: String,
    pub start_time: String,
    pub end_time: String,
    pub user_ids: Vec<String>,
}

pub trait ICameraTempBlockingQCCollection {
    fn list_temp_blocking_for_user(&self, user_id: &str) -> impl std::future::Future<Output = Result<Vec<CameraTempBlocking>, ListCameraTempBlockingsQueryError>> + Send;
    
    fn create_temp_blocking(&self, input: CreateCameraTempBlockingInput) -> impl std::future::Future<Output = Result<(), CreateCameraTempBlockingError>> + Send;
}

pub struct CameraTempBlockingQCCollection {
    client: aws_sdk_dynamodb::Client,
    table: String,
}

impl CameraTempBlockingQCCollection {
    pub fn new(client: aws_sdk_dynamodb::Client, table: String) -> Self {
        Self { client, table }
    }
}
#[derive(Debug, Clone)]
pub struct ListCameraTempBlockingsQueryError(pub QCError);

#[derive(Debug, Clone)]
pub struct CreateCameraTempBlockingError(pub QCError);


impl ICameraTempBlockingQCCollection for CameraTempBlockingQCCollection {
    async fn list_temp_blocking_for_user(&self, user_id: &str) -> Result<Vec<CameraTempBlocking>, ListCameraTempBlockingsQueryError> {
        let response = self.client
            .query()
            .table_name(&self.table)
            .key_condition_expression("partitionKey = :pk")
            .expression_attribute_values(":pk", AttributeValue::S(format!("cameraTempBlocking/{}", user_id)))
            .send()
            .await
            .map_err(|e| ListCameraTempBlockingsQueryError(QCError::new(e.to_string(), Some(format!("{:?}", e)))))?;

        let mut results = Vec::new();
        if let Some(items) = response.items {
            for item in items {
                if let (Some(sort_key), Some(end_date)) = (
                    item.get("sortKey").and_then(|v| v.as_s().ok()),
                    item.get("end_date").and_then(|v| v.as_s().ok())
                ) {
                    results.push(CameraTempBlocking {
                        id: sort_key.clone(),
                        camera_id: sort_key.clone(),
                        end_date: end_date.clone(),
                    });
                }
            }
        }
        Ok(results)
    }

    async fn create_temp_blocking(&self, input: CreateCameraTempBlockingInput) -> Result<(), CreateCameraTempBlockingError> {
        for user_id in &input.user_ids {
            self.client
                .put_item()
                .table_name(&self.table)
                .item("partitionKey", AttributeValue::S(format!("cameraTempBlocking/{}", user_id)))
                .item("sortKey", AttributeValue::S(input.camera_id.clone()))
                .item("start_date", AttributeValue::S(input.start_time.clone()))
                .item("end_date", AttributeValue::S(input.end_time.clone()))
                .item("camera_id", AttributeValue::S(input.camera_id.clone()))
                .item("user_id", AttributeValue::S(user_id.clone()))
                .send()
                .await
                .map_err(|e| CreateCameraTempBlockingError(QCError::new(
                    "failed to create camera temp blocking".to_string(),
                    Some(format!("{:?}", e)),
                )))?;
        }
        Ok(())
    }
}