use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;

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

pub trait IUserQCCollection {
    fn list_users(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<UserListQueryResultItem>, ListUsersQueryError>>
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
