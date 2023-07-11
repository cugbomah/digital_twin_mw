use crate::utilities::app_error::AppError;
use axum::{
    async_trait,
    body::HttpBody,
    extract::FromRequest,
    http::{Request, StatusCode},
    BoxError, Json, RequestExt,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct ValidateCreatePolicy {
    #[validate(required(message = "missing policy name"))]
    pub name: Option<String>,
    #[validate(required(message = "missing policy description"))]
    pub description: Option<String>,
    #[validate(required(message = "missing policy model_id"))]
    #[serde(rename = "modelId")]
    pub model_id: Option<i32>,
    #[serde(rename = "blockAfter")]
    pub block_after: Option<i32>,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for ValidateCreatePolicy
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
    ) -> Result<ValidateCreatePolicy, Self::Rejection> {
        let Json(model) = req
            .extract::<Json<ValidateCreatePolicy>, _>()
            .await
            .map_err(|error| {
                eprintln!("Error extracting new model policy info: {:?}", error);
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong, please try again",
                )
            })?;

        if let Err(errors) = model.validate() {
            let field_errors = errors.field_errors();
            for (_, error) in field_errors {
                return Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    error.first().unwrap().clone().message.unwrap().to_string(), // feel safe unwrapping because we know there is at least one error, and we only care about the first for this api
                ));
            }
        }

        Ok(model)
    }
}
