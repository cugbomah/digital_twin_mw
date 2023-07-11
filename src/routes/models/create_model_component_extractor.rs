use crate::utilities::app_error::AppError;
use axum::{
    async_trait,
    body::HttpBody,
    extract::FromRequest,
    http::{Request, StatusCode},
    BoxError, Json, RequestExt,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct ValidateCreateModelComponent {
    #[validate(required(message = "missing model component name"))]
    pub name: Option<String>,
    #[validate(required(message = "missing model component image source"))]
    #[serde(rename = "imageSource")]
    pub image_source: Option<String>,
    #[serde(rename = "componentAlias")]
    pub component_alias: Option<String>,
    #[serde(rename = "containerPort")]
    pub container_port: Option<i32>,
    #[serde(rename = "isExposed")]
    pub is_exposed: Option<bool>,
    #[validate(required(message = "missing model id"))]
    #[serde(rename = "modelId")]
    pub model_id: Option<Uuid>,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for ValidateCreateModelComponent
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
    ) -> Result<ValidateCreateModelComponent, Self::Rejection> {
        let Json(model) = req
            .extract::<Json<ValidateCreateModelComponent>, _>()
            .await
            .map_err(|error| {
                eprintln!("Error extracting new model info: {:?}", error);
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
