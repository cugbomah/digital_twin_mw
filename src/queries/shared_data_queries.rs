use crate::database::{self, core_shared_model_data};
use axum::http::StatusCode;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utilities::app_error::AppError;

#[derive(Debug, Deserialize, Serialize)]
pub struct SharedData {
    // pub id: Uuid,
    #[serde(rename = "modelId")]
    pub model_id: Option<Uuid>,
    #[serde(rename = "inputData")]
    pub input_data: Option<String>,
    #[serde(rename = "outputResponse")]
    pub output_response: Option<String>,
}

pub async fn store_usage_data(
    db: DatabaseConnection,
    shared_data: SharedData,
) -> Result<(StatusCode, ()), AppError> {
    let mut stored_data = core_shared_model_data::ActiveModel {
        ..Default::default()
    };
    let new_entry_id = Uuid::new_v4();

    stored_data.id = Set(new_entry_id.clone());
    stored_data.model_id = Set(shared_data.model_id.unwrap());
    stored_data.input_data = Set(Some(shared_data.input_data.unwrap()));
    stored_data.output_response = Set(Some(shared_data.output_response.unwrap()));

    let _stored_data = save_active_shared_data(&db, stored_data).await?;

    Ok((StatusCode::OK, ()))
}

async fn save_active_shared_data(
    db: &DatabaseConnection,
    shared_data: core_shared_model_data::ActiveModel,
) -> Result<(), AppError> {
    let _shared_data = shared_data.insert(db).await.map_err(|error| {
        let error_message = error.to_string();

        if error_message.contains("duplicate key value violates unique constraint") {
            AppError::new(StatusCode::BAD_REQUEST, "Duplicate entry")
        } else {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong, please try again",
            )
        }
    })?;

    Ok(())
}
