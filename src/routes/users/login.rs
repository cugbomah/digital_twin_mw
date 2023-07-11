use crate::queries::role_queries::find_role_by_id;
use crate::queries::user_queries::find_user_by_email;
use crate::utilities::app_error::AppError;
use crate::utilities::hash::{hash_password, verify_password};
use crate::utilities::jwt::{create_token, validate_token};
use crate::utilities::redis_connection_wrapper::RedisConnWrapper;
use crate::utilities::redis_helper::{
    delete_token_from_redis, get_token_from_redis, store_token_in_redis,
};
use crate::utilities::token_duration_wrapper::TokenDurationWrapper;
use crate::utilities::token_wrapper::TokenWrapper;

use axum::http::{header, Response, StatusCode};

use axum::{extract::State, Json};

use sea_orm::DatabaseConnection;
// use tower_cookies::cookie::SameSite;
// use tower_cookies::{Cookie, Cookies};
use axum_extra::extract::cookie::{Cookie, SameSite};
use uuid::{uuid, Uuid};

use super::signin_user_extractor::ValidateSignInUser;
use super::{ResponseDataUser, ResponseUser};

const AUTH_TOKEN: &str = "authorization";

// pub async fn login(
//     State(db): State<DatabaseConnection>,
//     State(token_secret): State<TokenWrapper>,
//     State(token_duration): State<TokenDurationWrapper>,
//     State(redis_url): State<RedisConnWrapper>,
//     // cookies: Cookies,
//     Json(request_user): Json<ValidateSignInUser>,
// ) -> Result<Json<ResponseDataUser>, AppError> {
//     let mut hashed_password = String::new();
//     let mut role_name = String::new();
//     let mut user_role_id: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
//     let mut user_id: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

//     //First check if user exists in Redis
//     let mut store_key = "Users:".to_string();
//     store_key = store_key + &request_user.email.clone().unwrap();
//     let redis_token = get_token_from_redis(redis_url.clone(), store_key.clone()).await?;

//     //If user exists in Redis, return token
//     if redis_token.clone() != "" {
//         //Extract password from token
//         let (result, claim) = validate_token(&token_secret.0, &redis_token)?;
//         //dbg!(&result, &claim);
//         //If token is valid, verify claims
//         if result {
//             //If claims are valid, return token
//             if claim.email != request_user.email.clone().unwrap() {
//                 return Err(AppError::new(
//                     StatusCode::UNAUTHORIZED,
//                     "incorrect email and/or password",
//                 ));
//             }
//             //Verify password
//             hashed_password = claim.password;
//             role_name = claim.role;
//             user_id = claim.id;
//         } else {
//             return Err(AppError::new(
//                 StatusCode::UNAUTHORIZED,
//                 "incorrect email and/or password",
//             ));
//         }
//     } else {
//         //If user does not exist in Redis, check if user exists in DB
//         println!("User does not exist in Redis");
//         let user = find_user_by_email(&db, request_user.email.clone().unwrap()).await?;
//         hashed_password = user.password;
//         user_role_id = user.role_id;
//         user_id = user.id;
//     }

//     if !verify_password(&request_user.password.clone().unwrap(), &hashed_password)? {
//         return Err(AppError::new(
//             StatusCode::UNAUTHORIZED,
//             "incorrect email and/or password",
//         ));
//     }

//     //Create Token
//     if user_role_id.clone() != uuid!("00000000-0000-0000-0000-000000000000") {
//         let role = find_role_by_id(&db, user_role_id).await?;
//         role_name = role.name;
//     }

//     let token = create_token(
//         token_duration.0,
//         &token_secret.0,
//         user_id.clone(),
//         request_user.email.clone().unwrap(),
//         hash_password(&request_user.password.unwrap())?,
//         role_name.clone(),
//     )?;

//     //Set Cookie
//     let cookie = Cookie::build(AUTH_TOKEN, token.to_owned())
//         .path("/")
//         .max_age(tower_cookies::cookie::time::Duration::hours(1))
//         .same_site(SameSite::Lax)
//         .http_only(true)
//         .finish();
//     //cookies.add(Cookie::new(AUTH_TOKEN, token.clone()));

//     let mut response =
//         Response::new(serde_json::json!({"status": "success", "token": token}).to_string());
//     response
//         .headers_mut()
//         .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

//     //dbg!(response.headers());

//     //Delete Token from Redis
//     if redis_token.clone() != "" {
//         delete_token_from_redis(redis_url.clone(), store_key.clone()).await?;
//     }

//     store_token_in_redis(
//         redis_url.clone(),
//         store_key.clone(),
//         token.clone(),
//         token_duration.0.try_into().unwrap(),
//     )
//     .await?;

//     let response = ResponseUser {
//         id: user_id,
//         email: request_user.email.unwrap(),
//         token: token.clone(),
//         first_name: "".to_string(),
//         last_name: "".to_string(),
//     };

//     Ok(Json(ResponseDataUser { data: response }))
// }

pub async fn login(
    State(db): State<DatabaseConnection>,
    State(token_secret): State<TokenWrapper>,
    State(token_duration): State<TokenDurationWrapper>,
    State(redis_url): State<RedisConnWrapper>,
    // cookies: Cookies,
    Json(request_user): Json<ValidateSignInUser>,
) -> Result<Response<String>, AppError>
//Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>
{
    let hashed_password: String;
    let mut role_name = String::new();
    let mut user_role_id: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
    let user_id: Uuid;

    //First check if user exists in Redis
    let mut store_key = "Users:".to_string();
    store_key = store_key + &request_user.email.clone().unwrap();
    let redis_token = get_token_from_redis(redis_url.clone(), store_key.clone()).await?;

    //If user exists in Redis, return token
    if redis_token.clone() != "" {
        //Extract password from token
        let (result, claim) = validate_token(&token_secret.0, &redis_token).await?;
        //dbg!(&result, &claim);
        //If token is valid, verify claims
        if result {
            //If claims are valid, return token
            if claim.email != request_user.email.clone().unwrap() {
                return Err(AppError::new(
                    StatusCode::UNAUTHORIZED,
                    "Incorrect email and/or password",
                ));
            }
            //Verify password
            hashed_password = claim.password;
            role_name = claim.role;
            user_id = claim.id;
        } else {
            return Err(AppError::new(
                StatusCode::UNAUTHORIZED,
                "Incorrect email and/or password",
            ));
        }
    } else {
        //If user does not exist in Redis, check if user exists in DB
        println!("User does not exist in Redis");
        let user = find_user_by_email(&db, request_user.email.clone().unwrap()).await?;
        hashed_password = user.password;
        user_role_id = user.role_id;
        user_id = user.id;
    }

    if !verify_password(&request_user.password.clone().unwrap(), &hashed_password)? {
        return Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "Incorrect email and/or password",
        ));
    }

    //Create Token
    if user_role_id.clone() != uuid!("00000000-0000-0000-0000-000000000000") {
        let role = find_role_by_id(&db, user_role_id).await?;
        role_name = role.name;
    }

    let token = create_token(
        token_duration.0,
        &token_secret.0,
        user_id.clone(),
        request_user.email.clone().unwrap(),
        hash_password(&request_user.password.unwrap())?,
        role_name.clone(),
    )?;

    //The code below works
    let cookie = Cookie::build(AUTH_TOKEN, token.to_owned())
        .path("/")
        .max_age(tower_cookies::cookie::time::Duration::hours(1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    //Delete Token from Redis
    if redis_token.clone() != "" {
        delete_token_from_redis(redis_url.clone(), store_key.clone()).await?;
    }

    store_token_in_redis(
        redis_url.clone(),
        store_key.clone(),
        token.clone(),
        token_duration.0.try_into().unwrap(),
    )
    .await?;

    let response_user = ResponseUser {
        id: user_id,
        email: request_user.email.unwrap(),
        token: token.to_owned(),
        first_name: "".to_string(),
        last_name: "".to_string(),
    };
    let mut response = Response::new(serde_json::json!({ "data": response_user }).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok(response)
}
