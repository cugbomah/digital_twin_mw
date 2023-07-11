use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use sea_orm::{DatabaseConnection, IntoActiveModel, Set};
use uuid::Uuid;

use crate::utilities::{app_error::AppError, redis_connection_wrapper::RedisConnWrapper};
use crate::{database::core_user, queries::twin_queries, utilities::docker_helper};

pub async fn start_twins(
    Path(twin_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<String>, AppError> {
    let (twin, twin_component) = twin_queries::find_twin_by_id(&db, twin_id, user.id).await?;

    //Check if twin.twin_status_id is equal to 1 (Stopped) then start twin
    if twin.twin_status_id == 1 {
        //Call helper function to start twin
        let response = docker_helper::start_docker_model(twin_component, &user.email).await?;

        //Update twin status to "Running"
        let mut twin = twin.into_active_model();
        twin.twin_status_id = Set(2);
        twin_queries::save_active_coretwin(&db, twin).await?;

        Ok(response)
    } else {
        Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "Twin is already running",
        ))
    }
}

pub async fn stop_twins(
    Path(twin_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<String>, AppError> {
    let (twin, twin_component) = twin_queries::find_twin_by_id(&db, twin_id, user.id).await?;

    //Check if twin.twin_status_id is equal to 2 (Running) then stop twin
    if twin.twin_status_id == 2 {
        //Call helper function to start twin
        let response = docker_helper::stop_docker_model(twin_component, &user.email).await?;

        //Update twin status to "Stopped"
        let mut twin = twin.into_active_model();
        twin.twin_status_id = Set(1);
        twin_queries::save_active_coretwin(&db, twin).await?;

        Ok(response)
    } else {
        Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "Twin is already stopped",
        ))
    }
}
