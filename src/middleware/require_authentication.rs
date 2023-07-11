use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::{header, Request, StatusCode},
    middleware::Next,
    response::Response,
    TypedHeader,
};
use chrono::{DateTime, FixedOffset, Utc};

use crate::{
    database::core_user,
    routes::users::ResponseUser,
    utilities::{app_error::AppError, jwt::validate_token, token_wrapper::TokenWrapper},
};
use axum_extra::extract::cookie::CookieJar;

pub async fn require_authentication<B>(
    State(token_secret): State<TokenWrapper>,
    cookie_jar: CookieJar,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // println!("Authenticating user");
    // dbg!(request.uri().path());
    // dbg!(request.uri().path().starts_with("/api"));

    let token = cookie_jar
        .get("authorization")
        .map(|cookie| {
            // println!("Cookie value: {}", cookie.value().to_string());
            cookie.value().to_string()
        })
        .or_else(|| {
            request
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    println!("Auth value: {}", auth_value.clone());
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });
    // dbg!(token.clone());
    let token = token.ok_or_else(|| {
        //eprintln!("Error extracting token from headers: {:?}", error);
        AppError::new(StatusCode::UNAUTHORIZED, "not authenticated!")
    })?;

    //We got this far because the token exists, now we need to validate it
    let (result, claim) = validate_token(&token_secret.0, &token).await?;
    if !result {
        return Err(AppError::new(
            StatusCode::UNAUTHORIZED,
            "not authenticated!",
        ));
    }

    // dbg!(&claim);

    let user_role = claim.role.to_lowercase();
    let user_role = user_role.as_str();
    let mut path = "/".to_owned();
    path.push_str(user_role);

    let utc_datetime: DateTime<Utc> = Utc::now();
    let fixed_offset = utc_datetime.with_timezone(&FixedOffset::west_opt(0).unwrap());
    let user = core_user::Model {
        id: claim.id,
        email: claim.email,
        first_name: "".to_string(),
        last_name: "".to_string(),
        password: "".to_string(),
        role_id: claim.id,
        status: true,
        created_at: fixed_offset,
        created_by: None,
        updated_at: None,
        updated_by: None,
        deleted_at: None,
        deleted_by: None,
    };

    if user_role != "admin" {
        if request.uri().path().starts_with(path.as_str()) {
            request.extensions_mut().insert(user);
            return Ok(next.run(request).await);
        } else {
            if request.uri().path().starts_with("/api") {
                //Grant access to all authenticated users to the api
                request.extensions_mut().insert(user);
                return Ok(next.run(request).await);
            } else {
                return Err(AppError::new(
                    StatusCode::UNAUTHORIZED,
                    "You are not authorized for this",
                ));
            }
        }
    }

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}
