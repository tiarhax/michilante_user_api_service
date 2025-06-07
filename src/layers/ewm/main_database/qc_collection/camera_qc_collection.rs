use std::collections::HashMap;

use aws_sdk_dynamodb::{types::AttributeValue};
use chrono::Utc;
use super::error::QCError;
#[derive(Clone)]
pub struct CameraListQueryResultItem {
    pub id: String,
    pub name: String,
}
impl TryFrom<&HashMap<String, AttributeValue>> for CameraListQueryResultItem {
    type Error = String;

    fn try_from(value: &HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let id = value
            .get("sortKey")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| "Missing or invalid 'id' field".to_string())?
            .to_string();

        let name = value
            .get("name")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| "Missing or invalid 'name' field".to_string())?
            .to_string();

        Ok(CameraListQueryResultItem { id, name })
    }
}
#[derive(Debug, Clone)]
pub struct ListCamerasQueryError(pub QCError);
#[derive(Debug, Clone)]
pub struct CreateCameraCommandError(pub QCError);

pub trait ICameraQCCollection {
    fn list_cameras(&self) -> impl std::future::Future<Output = Result<Vec<CameraListQueryResultItem>, ListCamerasQueryError>> + Send;
    fn create_camera(
        &self,
        command_input: CreateCameraCommandInput,
    ) -> impl std::future::Future<Output = Result<CreateCameraCommandOutput, CreateCameraCommandError>> + Send;
}

#[derive(Clone)]

pub struct CameraQCCollection {
    client: aws_sdk_dynamodb::Client,
    table: String,
}

impl CameraQCCollection {
    pub fn new(client: aws_sdk_dynamodb::Client, table: String) -> Self {
        Self { client, table }
    }
}
pub struct CreateCameraCommandInput {
    pub name: String,
    pub source_url: String,
}

pub struct CreateCameraCommandOutput {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}
impl ICameraQCCollection for CameraQCCollection {
    async fn list_cameras(&self) -> Result<Vec<CameraListQueryResultItem>, ListCamerasQueryError> {
        let results = self
            .client
            .query()
            .table_name(&self.table)
            .key_condition_expression("#partitionKey = :partitionKeyVal")
            .expression_attribute_names("#partitionKey", "partitionKey")
            .expression_attribute_values(":partitionKeyVal", AttributeValue::S("camera".to_string()))
            .send()
            .await
            .map_err(|err| {
                ListCamerasQueryError(QCError::new(
                    "failed to fetch items from database".to_string(),
                    Some(format!("{:?}", err)),
                ))
            })?;

        if let Some(items) = results.items {
            let results = items
                .iter()
                .map(|v| {
                    CameraListQueryResultItem::try_from(v).map_err(|err| {
                        ListCamerasQueryError(QCError::new(
                            "failed to parse item".to_string(),
                            Some(err),
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(results)
        } else {
            Ok(vec![])
        }
    }

    async fn create_camera(
        &self,
        command_input: CreateCameraCommandInput,
    ) -> Result<CreateCameraCommandOutput, CreateCameraCommandError> {
        let id = ulid::Ulid::new().to_string();
        let partition_key = "camera";
        let sort_key = id.clone();

        let result = CreateCameraCommandOutput {
            id,
            name: command_input.name,
            source_url: command_input.source_url,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.client
            .put_item()
            .table_name(&self.table)
            .item("partitionKey", AttributeValue::S(partition_key.to_string()))
            .item("sortKey", AttributeValue::S(sort_key.to_string()))
            .item("name", AttributeValue::S(result.name.clone()))
            .item("url", AttributeValue::S(result.source_url.clone()))
            .item(
                "createdAt",
                AttributeValue::S(result.created_at.to_rfc3339()),
            )
            .item(
                "updatedAt",
                AttributeValue::S(result.updated_at.to_rfc3339()),
            )
            .send()
            .await
            .map_err(|err| {
                CreateCameraCommandError(QCError::new(
                    "failed to persist item in database".to_string(),
                    Some(format!("{:?}", err)),
                ))
            })?;

        Ok(result)
    }
}
