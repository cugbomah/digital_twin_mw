use axum::http::StatusCode;
use chrono::Duration;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::app_error::AppError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    exp: usize,
    pub id: uuid::Uuid,
    pub email: String,
    pub password: String,
    pub role: String,
}

pub fn create_token(
    token_duration: i64,
    secret: &str,
    id: uuid::Uuid,
    email: String,
    password: String,
    role: String,
) -> Result<String, AppError> {
    // add at least an hour for this timestamp
    let now = chrono::Utc::now();
    let duration = Duration::minutes(token_duration);

    let expires_at = now + duration;
    let exp = expires_at.timestamp() as usize;
    let claims = Claims {
        exp,
        id,
        email,
        password,
        role,
    };
    let token_header = Header::default();
    let key = EncodingKey::from_secret(secret.as_bytes());

    encode(&token_header, &claims, &key).map_err(|error| {
        eprintln!("Error creating token: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "There was an error, please try again later",
        )
    })
}

pub async fn validate_token(secret: &str, token: &str) -> Result<(bool, Claims), AppError> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    decode::<Claims>(token, &key, &validation)
        .map_err(|error| match error.kind() {
            jsonwebtoken::errors::ErrorKind::InvalidToken
            | jsonwebtoken::errors::ErrorKind::InvalidSignature
            | jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                eprintln!("Error validating token: {:?}", error);

                AppError::new(StatusCode::UNAUTHORIZED, "not authenticated!")
            }
            _ => {
                eprintln!("Error validating token: {:?}", error);
                AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error validating token")
            }
        })
        .map(|claim| (true, claim.claims))
}
