use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use sea_orm::DatabaseConnection;
use tokio::task;
use uuid::Uuid;

use crate::{
    database::core_user,
    helpers::model_mgmt_helpers::twin_subscription,
    queries::{model_queries, policy_queries},
    utilities::{
        app_error::AppError, redis_connection_wrapper::RedisConnWrapper,
        redis_helper::store_token_in_redis,
    },
};

use super::create_twin_extractor::ValidateCreateTwin;

pub async fn subscribe(
    Path(model_id): Path<Uuid>,
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
    State(redis_url): State<RedisConnWrapper>,
    Json(share_twin_data): Json<ValidateCreateTwin>,
) -> Result<Json<String>, AppError> {
    let (model, model_components) =
        model_queries::find_pubslished_model_by_id(&db, model_id).await?;

    let twin_id = twin_subscription(
        &db,
        &user,
        &model,
        model_components,
        redis_url.clone(),
        share_twin_data,
    )
    .await?;

    //Use model_id to find latest policy
    let policies = policy_queries::get_policies(&db, model_id, user.id).await?;

    let policy = policies.into_iter().next().ok_or_else(|| {
        eprintln!("Could not find policy by model id");
        AppError::new(StatusCode::NOT_FOUND, "Could not find policy by model id")
    })?;

    for policy_action in policy.policy_actions {
        let redis_url_clone = redis_url.clone();
        task::spawn(async move {
            // Save Policy Action to Redis
            let mut store_key = "Policy:Users:".to_string();
            store_key += &user.id.to_string();
            store_key += ":";
            store_key += &twin_id.to_string();
            store_key += ":";
            store_key += &policy_action.end_point.to_string();

            let expires_in = match &policy_action
                .reset_frequency
                .unwrap_or_default()
                .parse::<i32>()
                .unwrap_or_default()
            {
                1 => Some(24 * 60),
                2 => Some(7 * 26 * 60),
                3 => Some(4 * 7 * 26 * 60),
                4 => Some(12 * 4 * 7 * 26 * 60),
                _ => Some(usize::MAX as i32),
            };

            let _redis_response = store_token_in_redis(
                redis_url_clone,
                store_key.clone(),
                "0".to_string(),
                expires_in.unwrap() as usize,
            )
            .await;
        });
    }

    // Yield to other tasks (optional)
    task::yield_now().await;

    Ok(Json("Subscribed to model successfully!".to_owned()))
}
