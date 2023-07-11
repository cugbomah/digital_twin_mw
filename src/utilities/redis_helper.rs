use axum::http::StatusCode;

use redis::AsyncCommands;

use crate::utilities::app_error::AppError;

use super::redis_connection_wrapper::RedisConnWrapper;

pub async fn store_token_in_redis(
    redis_url: RedisConnWrapper,
    key: String,
    value: String,
    expires_in: usize,
) -> Result<(), AppError> {
    let mut con = get_redis_connection(redis_url).await?;

    con.set(key.clone(), value).await.map_err(|error| {
        eprintln!("Error setting token in redis: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Sign-Up Successful but we couldn't log you in, please try logging-in again",
        )
    })?;

    let expires_in = if expires_in != usize::MAX {
        expires_in * 60
    } else {
        3600 * 24 * 365
    };

    con.expire(key, expires_in).await.map_err(|error| {
        eprintln!("Error setting token in redis: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Sign-Up Successful but we couldn't log you in, please try logging-in again",
        )
    })?;

    Ok(())
}

// pub async fn get_token_from_redis(
//     redis_url: RedisConnWrapper,
//     key: String,
// ) -> Result<String, AppError> {
//     let mut con = get_redis_connection(redis_url).await?;
//     let token = con.get(key).await.map_err(|error| {
//         eprintln!("Error getting token from redis: {:?}", error);
//         AppError::new(
//             StatusCode::INTERNAL_SERVER_ERROR,
//             "Sign-Up Successful but we couldn't log you in, please try logging-in again",
//         )
//     })?;

//     Ok(token)
// }

pub async fn get_token_from_redis(
    redis_url: RedisConnWrapper,
    key: String,
) -> Result<String, AppError> {
    let mut con = get_redis_connection(redis_url).await?;
    let token = con.get(key).await.unwrap_or("".to_string());

    Ok(token)
}

pub async fn get_token_ttl(redis_url: RedisConnWrapper, key: String) -> Result<i32, AppError> {
    let mut con = get_redis_connection(redis_url).await?;
    let ttl = con.ttl(key).await.unwrap_or(0) / 60;

    Ok(ttl)
}

pub async fn delete_token_from_redis(
    redis_url: RedisConnWrapper,
    key: String,
) -> Result<(), AppError> {
    let mut con = get_redis_connection(redis_url).await?;
    let _: () = con.del(key).await.map_err(|error| {
        eprintln!("Error deleting token from redis: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Sign-Up Successful but we couldn't log you in, please try logging-in again",
        )
    })?;

    Ok(())
}

async fn get_redis_connection(
    redis_url: RedisConnWrapper,
) -> Result<redis::aio::Connection, AppError> {
    let client = redis::Client::open(redis_url.0).unwrap();

    //let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_async_connection().await.map_err(|error| {
        eprintln!("Error getting redis connection: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Sign-Up Successful but we couldn't log you in, please try logging-in again",
        )
    })?;

    Ok(con)
}
