use std::collections::HashMap;

use aws_sdk_dynamodb::types::{AttributeValue, KeysAndAttributes};

use super::error::QCError;

#[derive(Debug, Clone)]
pub struct UserListQueryResultItem {
    pub user_id: String,
    pub email: String,
    pub name: String,
}

impl TryFrom<&HashMap<String, AttributeValue>> for UserListQueryResultItem {
    type Error = String;

    fn try_from(value: &HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let user_id = value
            .get("sortKey")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| "Missing or invalid 'sortKey' field".to_string())?
            .to_string();

        let email = value
            .get("email")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| "Missing or invalid 'email' field".to_string())?
            .to_string();

        let name = value
            .get("name")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| "Missing or invalid 'name' field".to_string())?
            .to_string();

        Ok(UserListQueryResultItem {
            user_id,
            email,
            name,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ListUsersQueryError(pub QCError);

#[derive(Debug, Clone)]
pub struct FindUserByIdQueryError(pub QCError);

#[derive(Debug, Clone)]
pub struct FindUsersByIdsQueryError(pub QCError);

pub trait IUserQCCollection {
    fn list_users(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<UserListQueryResultItem>, ListUsersQueryError>>
           + Send;

    fn find_user_by_id(
        &self,
        user_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<UserListQueryResultItem>, FindUserByIdQueryError>>
           + Send;

    fn find_users_by_ids(
        &self,
        user_ids: Vec<String>,
    ) -> impl std::future::Future<Output = Result<Vec<UserListQueryResultItem>, FindUsersByIdsQueryError>>
           + Send;
}

#[derive(Clone)]
pub struct UserQCCollection {
    client: aws_sdk_dynamodb::Client,
    table: String,
}

impl UserQCCollection {
    pub fn new(client: aws_sdk_dynamodb::Client, table: String) -> Self {
        Self { client, table }
    }
}

impl IUserQCCollection for UserQCCollection {
    async fn find_user_by_id(&self, user_id: &str) -> Result<Option<UserListQueryResultItem>, FindUserByIdQueryError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.table)
            .key("partitionKey", AttributeValue::S("user".to_string()))
            .key("sortKey", AttributeValue::S(user_id.to_string()))
            .send()
            .await
            .map_err(|err| {
                FindUserByIdQueryError(QCError::new(
                    "failed to fetch user from database".to_string(),
                    Some(format!("{:?}", err)),
                ))
            })?;

        if let Some(item) = result.item {
            let user = UserListQueryResultItem::try_from(&item).map_err(|err| {
                FindUserByIdQueryError(QCError::new(
                    "failed to parse user item".to_string(),
                    Some(err),
                ))
            })?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn find_users_by_ids(&self, user_ids: Vec<String>) -> Result<Vec<UserListQueryResultItem>, FindUsersByIdsQueryError> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        let keys: Vec<HashMap<String, AttributeValue>> = user_ids
            .iter()
            .map(|user_id| {
                let mut key = HashMap::new();
                key.insert("partitionKey".to_string(), AttributeValue::S("user".to_string()));
                key.insert("sortKey".to_string(), AttributeValue::S(user_id.clone()));
                key
            })
            .collect();

        let keys_and_attributes = KeysAndAttributes::builder()
            .set_keys(Some(keys))
            .build()
            .map_err(|err| {
                FindUsersByIdsQueryError(QCError::new(
                    "failed to build keys and attributes".to_string(),
                    Some(format!("{:?}", err)),
                ))
            })?;

        let result = self
            .client
            .batch_get_item()
            .request_items(&self.table, keys_and_attributes)
            .send()
            .await
            .map_err(|err| {
                FindUsersByIdsQueryError(QCError::new(
                    "failed to batch fetch users from database".to_string(),
                    Some(format!("{:?}", err)),
                ))
            })?;

        let mut users = Vec::new();
        if let Some(responses) = result.responses {
            if let Some(items) = responses.get(&self.table) {
                for item in items {
                    let user = UserListQueryResultItem::try_from(item).map_err(|err| {
                        FindUsersByIdsQueryError(QCError::new(
                            "failed to parse user item".to_string(),
                            Some(err),
                        ))
                    })?;
                    users.push(user);
                }
            }
        }

        Ok(users)
    }

    async fn list_users(&self) -> Result<Vec<UserListQueryResultItem>, ListUsersQueryError> {
        let results = self
            .client
            .query()
            .table_name(&self.table)
            .key_condition_expression("#partitionKey = :partitionKeyVal")
            .expression_attribute_names("#partitionKey", "partitionKey")
            .expression_attribute_values(
                ":partitionKeyVal",
                AttributeValue::S("user".to_string()),
            )
            .send()
            .await
            .map_err(|err| {
                ListUsersQueryError(QCError::new(
                    "failed to fetch users from database".to_string(),
                    Some(format!("{:?}", err)),
                ))
            })?;

        if let Some(items) = results.items {
            let results = items
                .iter()
                .map(|v| {
                    UserListQueryResultItem::try_from(v).map_err(|err| {
                        ListUsersQueryError(QCError::new(
                            "failed to parse user item".to_string(),
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
}
