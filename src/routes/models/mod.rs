use serde::{Deserialize, Serialize};
use uuid::Uuid;

use self::{
    create_model_component_extractor::ValidateCreateModelComponent,
    create_model_extractor::ValidateCreateModel,
};

pub mod create_model;
pub mod create_model_component_extractor;
pub mod create_model_extractor;
pub mod delete_model;
pub mod get_all_models;
pub mod publish_model;
pub mod unpublish_model;

#[derive(Serialize, Deserialize)]
pub struct ResponseModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_published: bool,
    pub enable_data_sharing: bool,
    pub owner_id: Option<Uuid>,
    pub picture: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseDataModels {
    pub data: Vec<ResponseModel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestModelValidated {
    pub model_info: ValidateCreateModel,
    pub comp_info: Vec<ValidateCreateModelComponent>,
}
