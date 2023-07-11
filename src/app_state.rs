use crate::utilities::{
    redis_connection_wrapper::RedisConnWrapper, token_duration_wrapper::TokenDurationWrapper,
    token_wrapper::TokenWrapper,
};
use axum::extract::FromRef;
use sea_orm::DatabaseConnection;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: TokenWrapper,
    pub token_duration: TokenDurationWrapper,
    pub redis_url: RedisConnWrapper,
    //pub user_extension: Option<UserModel>,
}
