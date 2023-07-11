use hyper::StatusCode;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    queries::policy_queries,
    utilities::{
        app_error::AppError,
        redis_connection_wrapper::RedisConnWrapper,
        redis_helper::{get_token_from_redis, get_token_ttl, store_token_in_redis},
    },
};

pub async fn check_policy(
    db: &DatabaseConnection,
    twin_id: Uuid,
    endpoint_id: String,
    user_id: Uuid,
    policy_id: Uuid,
    redis_url: RedisConnWrapper,
) -> Result<bool, AppError> {
    //Check redis for policy
    let mut store_key_model = "Policy:Models:".to_string();
    store_key_model = store_key_model + &policy_id.clone().to_string();
    store_key_model = store_key_model + ":Access:" + &endpoint_id.clone();

    let model_policy_token =
        get_token_from_redis(redis_url.clone(), store_key_model.clone()).await?;

    let model_action_count = match model_policy_token.clone() {
        token if token != "" => token.parse::<i32>().unwrap_or_default(),
        _ => {
            println!("Policy not found in redis");
            // Get policy_action from db using policy_id and endpoint_id
            let policy_action = policy_queries::get_policy_action_by_policyid_and_endpoint(
                db,
                policy_id,
                endpoint_id.clone(),
            )
            .await?;

            //Store policy_action in redis
            let _redis_response = store_token_in_redis(
                redis_url.clone(),
                store_key_model.clone(),
                policy_action.action_count.to_string().clone(),
                usize::MAX,
            )
            .await?;
            policy_action.action_count
        }
    };

    // dbg!(model_action_count);

    //User's redis key
    let mut store_key_user = "Policy:Users:".to_string();
    store_key_user += &user_id.to_string();
    store_key_user += ":";
    store_key_user += &twin_id.to_string();
    store_key_user += ":";
    store_key_user += &endpoint_id.to_string();

    let user_policy_token = get_token_from_redis(redis_url.clone(), store_key_user.clone()).await?;

    let user_action_count: i32 = match user_policy_token.clone() {
        token if token != "" => token.parse::<i32>().unwrap_or_default(),
        _ => {
            println!("Policy not found in redis");
            // Get policy_action from db using policy_id and endpoint_id
            let policy_action = policy_queries::get_policy_action_by_policyid_and_endpoint(
                db,
                policy_id,
                endpoint_id.clone(),
            )
            .await?;

            //Match policy_action.reset_frequency_id and multiply the value with policy_action.action_count and assign to an integer variable expires_in
            let expires_in = match policy_action.reset_frequency_id {
                1 => 24 * 60,
                2 => 7 * 26 * 60,
                3 => 4 * 7 * 26 * 60,
                4 => 12 * 4 * 7 * 26 * 60,
                _ => usize::MAX as i32,
            };

            //Store policy_action in redis
            let _redis_response = store_token_in_redis(
                redis_url.clone(),
                store_key_user.clone(),
                "0".to_string(),
                expires_in as usize,
            )
            .await?;
            0
        }
    };

    // dbg!(user_action_count);

    //Check if user_action_count is greater than or equal to model_action_count
    if user_action_count >= model_action_count {
        return Err(AppError::new(
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
            "Resource usage policy not satisfied!",
        ));
    }

    //Increment user_action_count
    let expires_in = get_token_ttl(redis_url.clone(), store_key_user.clone()).await?;

    let _redis_response = store_token_in_redis(
        redis_url,
        store_key_user,
        (user_action_count + 1).to_string(),
        expires_in as usize,
    )
    .await?;

    Ok(true)
}
