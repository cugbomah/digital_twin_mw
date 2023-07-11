use axum::{
    extract::{Path, State},
    http::Response,
    Extension,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    database::core_user,
    helpers::model_mgmt_helpers::unpublish,
    queries::model_queries,
    utilities::{app_error::AppError, redis_connection_wrapper::RedisConnWrapper},
};

pub async fn delete_model(
    Path(model_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
    State(redis_url): State<RedisConnWrapper>,
) -> Result<Response<String>, AppError> {
    let (model, _returned_comp) = model_queries::find_model_by_id(&db, model_id, user.id).await?;

    //Check if model.deleted_at is None
    if model.deleted_at.is_none() {
        if !model.is_published {
            let response = model_queries::delete_model(&db, model_id, user.id.clone()).await?;
            Ok(response)
        } else {
            let _unpub_response = unpublish(&db, &user, &model, redis_url.clone()).await?;
            let del_response = model_queries::delete_model(&db, model_id, user.id.clone()).await?;
            Ok(del_response)
        }
    } else {
        //Model already deleted
        let response =
            Response::new(serde_json::json!({"status": "Model already deleted!"}).to_string());
        Ok(response)
    }
}
