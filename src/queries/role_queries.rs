use axum::http::StatusCode;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TryIntoModel,
};

use crate::{
    database::core_role::{self, Entity as Roles, Model as RoleModel},
    utilities::app_error::AppError,
};

pub async fn find_role_by_id(
    db: &DatabaseConnection,
    id: uuid::Uuid,
) -> Result<RoleModel, AppError> {
    let role = Roles::find_by_id(id)
        .filter(core_role::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting role by id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your role, please try again",
            )
        })?;

    role.ok_or_else(|| {
        eprintln!("Could not find role by id");
        AppError::new(StatusCode::NOT_FOUND, "not found")
    })
}

pub async fn find_role_by_name(
    db: &DatabaseConnection,
    role_name: String,
) -> Result<RoleModel, AppError> {
    let role = Roles::find()
        .filter(core_role::Column::Name.eq(role_name))
        .one(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting role by name: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your role, please try again",
            )
        })?;

    role.ok_or_else(|| {
        eprintln!("Could not find role by name");
        AppError::new(StatusCode::NOT_FOUND, "not found")
    })
}
