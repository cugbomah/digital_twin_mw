pub mod create_user_extractor;
pub mod login;
pub mod logout;
pub mod signin_user_extractor;
pub mod signup;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct ResponseDataUser {
    pub data: ResponseUser,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseUser {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct RequestCreateUser {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
}
