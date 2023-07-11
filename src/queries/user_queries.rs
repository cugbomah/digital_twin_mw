use crate::{
    database::{
        core_user::Model as UserModel,
        core_user::{self, Entity as Users},
    },
    routes::users::{create_user_extractor::ValidateCreateUser, ResponseDataUser, ResponseUser},
    utilities::{
        app_error::AppError, hash::hash_password, jwt::create_token,
        redis_connection_wrapper::RedisConnWrapper, redis_helper::store_token_in_redis,
        token_duration_wrapper::TokenDurationWrapper, token_wrapper::TokenWrapper,
    },
};
use axum::{http::StatusCode, Json};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TryIntoModel,
};
use uuid::Uuid;

use super::role_queries::find_role_by_name;
use tower_cookies::{Cookie, Cookies};

const AUTH_TOKEN: &str = "authorization";

pub async fn create_user(
    db: DatabaseConnection,
    jwt_secret: TokenWrapper,
    token_duration: TokenDurationWrapper,
    redis_url: RedisConnWrapper,
    cookies: Cookies,
    request_user: ValidateCreateUser,
    role_name: String,
) -> Result<(StatusCode, Json<ResponseDataUser>), AppError> {
    let mut user = core_user::ActiveModel {
        ..Default::default()
    };
    //Get User Role
    let user_role = find_role_by_name(&db, role_name).await?;
    let new_user_id = Uuid::new_v4();
    // let utc_datetime: DateTime<Utc> = Utc::now();
    // let fixed_offset = utc_datetime.with_timezone(&FixedOffset::west_opt(0).unwrap());

    user.id = Set(new_user_id.clone());
    user.email = Set(request_user.email.unwrap());
    user.first_name = Set(request_user.first_name.unwrap());
    user.last_name = Set(request_user.last_name.unwrap());
    user.role_id = Set(user_role.id);
    user.password = Set(hash_password(&request_user.password.unwrap())?);
    user.created_by = Set(Some(new_user_id.clone()));
    user.updated_by = Set(Some(new_user_id.clone()));

    let user = save_active_user(&db, user).await?;

    //Create Token
    let token = create_token(
        token_duration.0,
        &jwt_secret.0,
        user.id.clone(),
        user.email.clone(),
        user.password.clone(),
        user_role.name,
    )?;

    //Set Cookie
    cookies.add(Cookie::new(AUTH_TOKEN, token.clone()));

    //Store Token in Redis
    let mut store_key = "Users:".to_string();
    store_key = store_key + &user.email.as_str();
    store_token_in_redis(
        redis_url,
        store_key,
        token.clone(),
        token_duration.0.try_into().unwrap(),
    )
    .await?;

    //Return Response
    let response_user = ResponseUser {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        token,
    };

    let response_data_user = ResponseDataUser {
        data: response_user,
    };

    Ok((StatusCode::CREATED, Json(response_data_user)))
}

pub async fn save_active_user(
    db: &DatabaseConnection,
    user: core_user::ActiveModel,
) -> Result<UserModel, AppError> {
    let user = user.insert(db).await.map_err(|error| {
        let error_message = error.to_string();

        if error_message
            .contains("duplicate key value violates unique constraint \"core_user_email_key\"")
        {
            AppError::new(
                StatusCode::BAD_REQUEST,
                "Email already taken, try again with a different email",
            )
        } else {
            eprintln!("Error creating user: {:?}", error_message);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong, please try again",
            )
        }
    })?;

    //convert_active_to_model(user)
    Ok(user)
}

pub async fn find_user_by_email(
    db: &DatabaseConnection,
    email: String,
) -> Result<UserModel, AppError> {
    Users::find()
        .filter(core_user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting user by email: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error logging in, please try again later",
            )
        })?
        .ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, "Incorrect email and/or password"))
}

fn convert_active_to_model(active_user: core_user::ActiveModel) -> Result<UserModel, AppError> {
    active_user.try_into_model().map_err(|error| {
        eprintln!("Error converting task active model to model: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    })
}
