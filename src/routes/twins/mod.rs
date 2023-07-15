use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod create_twin_extractor;
pub mod delete_twin;
pub mod get_all_user_twins;
pub mod get_one_user_twin;
pub mod subscribe_to_model;
pub mod twin_operations;
pub mod twin_usage;

#[derive(Serialize, Deserialize)]
pub struct ResponseTwinModel {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub twin_port: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseTwinDataModels {
    pub data: Vec<ResponseTwinModel>,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseTwinDataModel {
    pub data: ResponseTwinModel,
}
