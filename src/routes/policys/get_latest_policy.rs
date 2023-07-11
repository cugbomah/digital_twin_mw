use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{database::core_user, queries::policy_queries, utilities::app_error::AppError};

use super::{ResponsePolicy, ResponsePolicyAction};

pub async fn get_all_model_policies(
    Path(model_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<ResponsePolicy>>, AppError> {
    let policies = policy_queries::get_policies(&db, model_id, user.id).await?;
    Ok(Json(policies))
}

pub async fn get_latest_model_policy(
    Path(model_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<ResponsePolicy>, AppError> {
    let policies = policy_queries::get_policies(&db, model_id, user.id).await?;

    let policy = policies.into_iter().next().ok_or_else(|| {
        eprintln!("Could not find policy by model id");
        AppError::new(StatusCode::NOT_FOUND, "Could not find policy by model id")
    })?;
    Ok(Json(policy))
}
