use axum::{
    extract::{Path, State},
    Extension, Json,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    database::core_user,
    helpers::model_mgmt_helpers::publish,
    queries::model_queries,
    utilities::{app_error::AppError, redis_connection_wrapper::RedisConnWrapper},
};

pub async fn publish_model(
    Path(model_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
    State(redis_url): State<RedisConnWrapper>,
) -> Result<Json<String>, AppError> {
    let (model, model_components) = model_queries::find_model_by_id(&db, model_id, user.id).await?;

    if model.is_published {
        Ok("Model already published!".to_string().into())
    } else {
        //Call helper function to publish model
        let response = publish(&db, &user, &model, model_components, redis_url.clone()).await?;
        Ok(response.into())
    }
}
