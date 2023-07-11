use axum::{
    extract::{Path, State},
    Extension, Json,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    database::core_user,
    helpers::model_mgmt_helpers::unpublish,
    queries::model_queries,
    utilities::{app_error::AppError, redis_connection_wrapper::RedisConnWrapper},
};

pub async fn unpublish_model(
    Path(model_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
    State(redis_url): State<RedisConnWrapper>,
) -> Result<Json<String>, AppError> {
    let (model, _returned_comp) = model_queries::find_model_by_id(&db, model_id, user.id).await?;

    if !model.is_published {
        dbg!(chrono::Utc::now().timestamp_micros().to_string());
        Ok(Json("Model already unpublished!".to_string()))
    } else {
        let response = unpublish(&db, &user, &model, redis_url.clone()).await?;
        Ok(response)
    }
}
