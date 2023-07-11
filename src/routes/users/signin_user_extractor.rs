use crate::utilities::app_error::AppError;
use axum::{
    async_trait,
    body::HttpBody,
    extract::FromRequest,
    http::{Request, StatusCode},
    BoxError, Json, RequestExt,
};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct ValidateSignInUser {
    #[validate(
        email(message = "invalid email"),
        required(message = "missing user email")
    )]
    pub email: Option<String>,
    #[validate(required(message = "missing user password"))]
    pub password: Option<String>,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for ValidateSignInUser
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(
        req: Request<B>,
        _state: &S,
    ) -> Result<ValidateSignInUser, Self::Rejection> {
        let Json(user) = req
            .extract::<Json<ValidateSignInUser>, _>()
            .await
            .map_err(|error| {
                eprintln!("Error extracting new user info: {:?}", error);
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong, please try again",
                )
            })?;

        if let Err(errors) = user.validate() {
            let field_errors = errors.field_errors();
            for (_, error) in field_errors {
                return Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    error.first().unwrap().clone().message.unwrap().to_string(), // feel safe unwrapping because we know there is at least one error, and we only care about the first for this api
                ));
            }
        }

        Ok(user)
    }
}
