use crate::{
    queries::user_queries::create_user,
    utilities::{
        app_error::AppError, redis_connection_wrapper::RedisConnWrapper,
        token_duration_wrapper::TokenDurationWrapper, token_wrapper::TokenWrapper,
    },
};
use axum::{extract::State, http::StatusCode, Json};
use sea_orm::DatabaseConnection;
use tower_cookies::Cookies;

use super::{create_user_extractor::ValidateCreateUser, ResponseDataUser};

pub async fn signup_user(
    State(db): State<DatabaseConnection>,
    State(jwt_secret): State<TokenWrapper>,
    State(token_duration): State<TokenDurationWrapper>,
    State(redis_url): State<RedisConnWrapper>,
    cookies: Cookies,
    Json(user): Json<ValidateCreateUser>,
) -> Result<(StatusCode, Json<ResponseDataUser>), AppError> {
    let role_name = "User".to_string();

    create_user(
        db,
        jwt_secret,
        token_duration,
        redis_url,
        cookies,
        user,
        role_name,
    )
    .await
}

pub async fn signup_owner(
    State(db): State<DatabaseConnection>,
    State(jwt_secret): State<TokenWrapper>,
    State(token_duration): State<TokenDurationWrapper>,
    State(redis_url): State<RedisConnWrapper>,
    cookies: Cookies,
    Json(user): Json<ValidateCreateUser>,
) -> Result<(StatusCode, Json<ResponseDataUser>), AppError> {
    let role_name = "Owner".to_string();
    create_user(
        db,
        jwt_secret,
        token_duration,
        redis_url,
        cookies,
        user,
        role_name,
    )
    .await
}
