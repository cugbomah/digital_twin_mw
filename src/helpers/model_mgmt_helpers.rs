use axum::{http::StatusCode, Json};
use sea_orm::{DatabaseConnection, IntoActiveModel, Set};
use uuid::Uuid;

use crate::{
    database::{core_model, core_model_component, core_twin, core_user},
    queries::{model_queries, twin_queries},
    routes::twins::create_twin_extractor::ValidateCreateTwin,
    utilities::{
        app_error::AppError,
        docker_helper::{create_docker_model, remove_docker_model, stop_docker_model},
        redis_connection_wrapper::RedisConnWrapper,
    },
};

pub async fn publish(
    db: &DatabaseConnection,
    user: &core_user::Model,
    model: &core_model::Model,
    model_components: Vec<core_model_component::Model>,
    redis_url: RedisConnWrapper,
) -> Result<Json<String>, AppError> {
    let (response, twin) =
        create_twin_infrastructure(&db, &user, &model, model_components, redis_url.clone()).await?;
    let mut model = model.clone().into_active_model();
    model.is_published = Set(true);

    model_queries::save_active_coremodel(&db, model).await?;

    let mut twin = twin.into_active_model();
    twin.twin_status_id = Set(2); //Set twin status to "Started"
    twin_queries::save_active_coretwin(&db, twin).await?;

    //Check if response is equal to "Docker containers removed successfully"
    if response.contains("Containers created successfully") {
        return Ok(Json("Model published successfully!".to_string()));
    } else {
        return Ok(Json("Model published successfully but there was an error starting the containers. Please try starting the model again".to_string()));
    }
}

pub async fn unpublish(
    db: &DatabaseConnection,
    user: &core_user::Model,
    model: &core_model::Model,
    redis_url: RedisConnWrapper,
) -> Result<Json<String>, AppError> {
    //Match model.type_id and determine whether to call create_docker_model or create_wasmed_model
    let response = match model.type_id {
        1 => {
            let (twin, twin_components) =
                twin_queries::find_twin_by_model_id(&db, model.id.clone(), user.id).await?;

            let _remove_msg =
                remove_docker_model(&twin, twin_components.clone(), redis_url.clone())
                    .await
                    .map_err(|error| {
                        eprintln!("Error removing docker model: {:?}", error);
                        AppError::new(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Error removing docker model",
                        )
                    })?;

            let _delete_msg = twin_queries::delete_twin(&db, twin.id.clone(), user.id).await?;

            // dbg!(delete_msg);

            let mut model = model.clone().into_active_model();
            model.is_published = Set(false);

            model_queries::save_active_coremodel(&db, model).await?;

            Ok(Json("Model unpublished successfully!".to_string()))
        }
        _ => {
            //Unpublish wasmed model
            Ok(Json("Model unpublished successfully!".to_string()))
        }
    }?;
    Ok(response)
}

pub async fn decomission_model() {
    todo!("Decomission model not yet implemented");
}

pub async fn twin_subscription(
    db: &DatabaseConnection,
    user: &core_user::Model,
    model: &core_model::Model,
    model_components: Vec<core_model_component::Model>,
    redis_url: RedisConnWrapper,
    share_data: ValidateCreateTwin,
) -> Result<Uuid, AppError> {
    let (response, twin) =
        create_twin_infrastructure(&db, &user, &model, model_components, redis_url.clone()).await?;

    let mut twin = twin.into_active_model();
    twin.twin_status_id = Set(2); //Set twin status to "Started"

    if model.enable_data_sharing {
        if let Some(enable_data_sharing) = share_data.enable_data_sharing {
            twin.enable_data_sharing = Set(enable_data_sharing);
        }
    }

    twin_queries::save_active_coretwin(&db, twin.clone()).await?;

    let twin_id: Option<uuid::Uuid> = match twin.id.clone().into_value() {
        Some(twin_id) => Some(twin.id.unwrap().clone()),
        _ => None,
    };

    Ok(twin_id.unwrap())
}

async fn create_wasmed_model() -> Result<Json<String>, AppError> {
    todo!("Create wasmed model not yet implemented");
}

async fn create_twin_infrastructure(
    db: &DatabaseConnection,
    user: &core_user::Model,
    model: &core_model::Model,
    model_components: Vec<core_model_component::Model>,
    redis_url: RedisConnWrapper,
) -> Result<(Json<String>, core_twin::Model), AppError> {
    let (twin, twin_components) =
        twin_queries::create_twin(&db, &user, &model, model_components).await?;

    //Match model.type_id and determine whether to call create_docker_model or create_wasmed_model
    let response = match model.type_id {
        1 => {
            let create_msg = create_docker_model(
                &db,
                &twin,
                twin_components.clone(),
                redis_url.clone(),
                &user.email,
            )
            .await
            .map_err(|error| {
                eprintln!("Error creating docker model: {:?}", error);
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error creating docker model",
                )
            })?;

            create_msg

            // let stop_msg = stop_docker_model(twin_components.clone(), &user.email)
            //     .await
            //     .map_err(|error| {
            //         eprintln!("Error stopping docker model: {:?}", error);
            //         AppError::new(
            //             StatusCode::INTERNAL_SERVER_ERROR,
            //             "Error stopping docker model",
            //         )
            //     })?;

            // stop_msg
        }
        _ => create_wasmed_model().await?,
    };
    Ok((response, twin))
}
