use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    database::core_user,
    queries::policy_queries,
    utilities::{app_error::AppError, redis_connection_wrapper::RedisConnWrapper},
};

use super::{RequestPolicyValidated, ResponsePolicy};

//Trying to handle as a transaction
pub async fn create_policy(
    Path(model_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
    State(redis_url): State<RedisConnWrapper>,
    Json(model): Json<RequestPolicyValidated>,
) -> Result<(StatusCode, Json<ResponsePolicy>), AppError> {
    //Invoke policy_queries::create_model_policy and map the values to struct ResponsePolicy with policy_actions vector mapped to PolicyAction
    let create_result =
        policy_queries::create_model_policy(&db, model_id, &user, model, redis_url.clone())
            .await
            .map(|model| {
                (
                    StatusCode::CREATED,
                    Json(ResponsePolicy {
                        id: model.id,
                        name: model.name,
                        description: model.description,
                        policy_version: Some(model.policy_version),
                        block_after: Some(model.block_after),
                        policy_actions: vec![],
                    }),
                )
            })?;
    Ok(create_result)
}
