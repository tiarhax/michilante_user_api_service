use std::collections::HashMap;

use super::error::QCError;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;
#[derive(Clone)]
pub struct CameraListQueryResultItem {
    pub id: String,
    pub name: String,
    pub source_url: String,
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

        let source_url = value
            .get("url")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| "Missing or invalid 'url' field".to_string())?
            .to_string();

        Ok(CameraListQueryResultItem {
            id,
            name,
            source_url,
        })
    }
}
#[derive(Debug, Clone)]
pub struct ListCamerasQueryError(pub QCError);
#[derive(Debug, Clone)]
pub struct CreateCameraCommandError(pub QCError);

#[derive(Debug, Clone)]
pub struct DeleteCameraCommandError(pub QCError);

#[derive(Debug, Clone)]
pub struct FindCamerabyIdError(pub QCError);

#[derive(Debug, Clone)]
pub struct CheckIfCameraExistsError(pub QCError);

pub trait ICameraQCCollection {
    fn list_cameras(
        &self,
    ) -> impl std::future::Future<
        Output = Result<Vec<CameraListQueryResultItem>, ListCamerasQueryError>,
    > + Send;
    fn put_camera(
        &self,
        command_input: PutCameraCommandInput,
    ) -> impl std::future::Future<Output = Result<CreateCameraCommandOutput, CreateCameraCommandError>>
           + Send;

    fn delete_camera_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<(), DeleteCameraCommandError>> + Send;

    fn find_camera_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<FindCameraByIdResult, FindCamerabyIdError>> + Send;

    fn camera_exists_by_id(
        &self,
        id: &str,
    ) -> impl std::future::Future<Output = Result<bool, CheckIfCameraExistsError>> + Send;
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
pub struct PutCameraCommandInput {
    pub id: Option<String>,
    pub name: String,
    pub source_url: String,
    pub permanent_stream_url: Option<String>,
}

pub struct CreateCameraCommandOutput {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub permanent_stream_url: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub struct FindCameraByIdResult {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub permanent_stream_url: Option<String>,
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
            .expression_attribute_values(
                ":partitionKeyVal",
                AttributeValue::S("camera".to_string()),
            )
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

    async fn put_camera(
        &self,
        command_input: PutCameraCommandInput,
    ) -> Result<CreateCameraCommandOutput, CreateCameraCommandError> {
        let id = command_input.id.unwrap_or(ulid::Ulid::new().to_string());
        let partition_key = "camera";
        let sort_key = id.clone();

        let result = CreateCameraCommandOutput {
            id,
            name: command_input.name,
            source_url: command_input.source_url,
            permanent_stream_url: command_input.permanent_stream_url,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let permanent_stream_url = match &result.permanent_stream_url {
            Some(s) => AttributeValue::S(s.clone()),
            None => AttributeValue::Null(true),
        };
        self.client
            .put_item()
            .table_name(&self.table)
            .item("partitionKey", AttributeValue::S(partition_key.to_string()))
            .item("sortKey", AttributeValue::S(sort_key.to_string()))
            .item("name", AttributeValue::S(result.name.clone()))
            .item("url", AttributeValue::S(result.source_url.clone()))
            .item("permanentStreamUrl", permanent_stream_url)
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

    async fn delete_camera_by_id(&self, id: &str) -> Result<(), DeleteCameraCommandError> {
        self.client
            .delete_item()
            .table_name(&self.table)
            .key("partitionKey", AttributeValue::S("camera".to_string()))
            .key("sortKey", AttributeValue::S(id.to_owned()))
            .send()
            .await
            .map_err(|err| {
                DeleteCameraCommandError(QCError::new(
                    "failed to delete camera from database".to_string(),
                    Some(format!("{:?}", err)),
                ))
            })?;

        Ok(())
    }

    async fn find_camera_by_id(
        &self,
        id: &str,
    ) -> Result<FindCameraByIdResult, FindCamerabyIdError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table)
            .key("partitionKey", AttributeValue::S("camera".to_string()))
            .key("sortKey", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|err| {
                FindCamerabyIdError(QCError::new(
                    "failed to fetch camera from database".to_string(),
                    Some(format!("{:?}", err)),
                ))
            })?;

        if let Some(item) = result.item {
            let id = item
                .get("sortKey")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| {
                    FindCamerabyIdError(QCError::new(
                        "Missing or invalid 'id' field".to_string(),
                        None,
                    ))
                })?
                .to_string();

            let name = item
                .get("name")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| {
                    FindCamerabyIdError(QCError::new(
                        "Missing or invalid 'name' field".to_string(),
                        None,
                    ))
                })?
                .to_string();

            let source_url = item
                .get("url")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| {
                    FindCamerabyIdError(QCError::new(
                        "Missing or invalid 'url' field".to_string(),
                        None,
                    ))
                })?
                .to_string();

            let permanent_stream_url = item
                .get("permanentStreamUrl")
                .and_then(|v| v.as_s().ok())
                .map(|s| s.to_string());

            let created_at = item
                .get("createdAt")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| {
                    FindCamerabyIdError(QCError::new(
                        "Missing or invalid 'createdAt' field".to_string(),
                        None,
                    ))
                })?;

            let updated_at = item
                .get("updatedAt")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| {
                    FindCamerabyIdError(QCError::new(
                        "Missing or invalid 'updatedAt' field".to_string(),
                        None,
                    ))
                })?;

            Ok(FindCameraByIdResult {
                id,
                name,
                source_url,
                permanent_stream_url,
                created_at,
                updated_at,
            })
        } else {
            Err(FindCamerabyIdError(QCError::new(
                "Camera not found".to_string(),
                None,
            )))
        }
    }


    async fn camera_exists_by_id(
            &self,
            id: &str,
        ) -> Result<bool, CheckIfCameraExistsError> {
        let result = self
            .client
            .get_item()
            .projection_expression("partitionKey, sortKey")
            .table_name(&self.table)
            .key("partitionKey", AttributeValue::S("camera".to_string()))
            .key("sortKey", AttributeValue::S(id.to_string()))
            .send()
            .await
            .map_err(|err| {
                CheckIfCameraExistsError(QCError::new(
                    "failed to check if camera exists in database".to_string(),
                    Some(format!("{:?}", err)),
                ))
            })?;

        Ok(result.item.is_some())
    }
}
