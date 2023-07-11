use axum::{
    http::{header, Response},
    Extension,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde_json::json;

use crate::{database::core_user, utilities::app_error::AppError};

const AUTH_TOKEN: &str = "authorization";

// pub async fn logout() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
//     let cookie = Cookie::build(AUTH_TOKEN, "")
//         .path("/")
//         .max_age(tower_cookies::cookie::time::Duration::hours(-1))
//         .same_site(SameSite::Lax)
//         .http_only(true)
//         .finish();

//     let mut response = Response::new(json!({"status": "success"}).to_string());
//     response
//         .headers_mut()
//         .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
//     Ok(response)
// }

pub async fn logout(
    Extension(user): Extension<core_user::Model>,
) -> Result<Response<String>, AppError> {
    let cookie = Cookie::build(AUTH_TOKEN, "")
        .path("/")
        .max_age(tower_cookies::cookie::time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    let mut response = Response::new(json!({"status": "success"}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    // println!("Logging Out User: {:?}", user);
    Ok(response)
}
