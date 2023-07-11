use axum::{
    extract::{Path, State},
    http::{Response, StatusCode},
    Extension, Json,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    database::core_user,
    queries::twin_queries::{self},
    utilities::{
        app_error::AppError, docker_helper::remove_docker_model,
        redis_connection_wrapper::RedisConnWrapper,
    },
};

pub async fn soft_delete_twin(
    Path(twin_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
    State(redis_url): State<RedisConnWrapper>,
) -> Result<Response<String>, AppError> {
    let (twin, twin_components) = twin_queries::find_twin_by_id(&db, twin_id, user.id).await?;

    //Call docker_helper function to remove docker model
    let _remove_msg = remove_docker_model(&twin, twin_components.clone(), redis_url.clone())
        .await
        .map_err(|error| {
            eprintln!("Error removing docker model: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error removing docker model",
            )
        })?;

    //Delete twin from database
    let response = twin_queries::delete_twin_by_models(&db, twin, twin_components, user.id).await?;

    Ok(response)
}
