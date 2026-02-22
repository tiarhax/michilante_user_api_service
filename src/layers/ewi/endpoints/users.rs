use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::layers::{
    business::usecases::list_users::{
        implementation::ListUsersUseCaseImp,
        interface::{IListUsersUseCase, UserListItem},
    },
    ewi::{appstate::{auth0::User, AppState}, error::AppError},
    ewm::main_database::qc_collection::user_qc_collection::UserQCCollection,
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserResultItem {
    user_id: String,
    email: String,
    name: String,
}

impl From<UserListItem> for UserResultItem {
    fn from(item: UserListItem) -> Self {
        UserResultItem {
            user_id: item.user_id,
            email: item.email,
            name: item.name,
        }
    }
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "users",
    responses(
        (status = 200, description = "List all users (Admin only)", body = Vec<UserResultItem>),
        (status = 403, description = "Forbidden - Admin role required")
    )
)]
pub async fn list_users(
    State(user_qc_collection): State<UserQCCollection>,
    user: User,
) -> Result<Json<Vec<UserResultItem>>, AppError> {
    tracing::info!("User: {:?}", user);
    if !user.roles.contains(&"Admin".to_string()) {
        return Err(AppError::Forbidden("Admin role required".to_string()));
    }

    let list_users_use_case = ListUsersUseCaseImp::new(user_qc_collection);

    let users = list_users_use_case
        .execute()
        .await
        .map_err(|err| AppError::from_use_case_error(err, None))?;

    let result = users
        .into_iter()
        .map(|u| u.into())
        .collect::<Vec<UserResultItem>>();

    Ok(Json(result))
}

pub fn setup_endpoints(router: Router<AppState>) -> Router<AppState> {
    router.route("/users", get(list_users))
}
