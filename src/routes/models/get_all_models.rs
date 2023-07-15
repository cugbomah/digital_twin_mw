use axum::{extract::State, Extension, Json};
use sea_orm::DatabaseConnection;

use crate::{
    database::core_user::Model as UserModel, queries::model_queries, utilities::app_error::AppError,
};

use super::{ResponseDataModels, ResponseModel};

pub async fn get_all_publsihed_models(
    State(db): State<DatabaseConnection>,
) -> Result<Json<ResponseDataModels>, AppError> {
    let models: Vec<ResponseModel> = model_queries::get_all_published_models(&db)
        .await?
        .into_iter()
        .map(|db_model| ResponseModel {
            id: db_model.id,
            description: db_model.description,
            is_published: db_model.is_published,
            enable_data_sharing: db_model.enable_data_sharing,
            name: db_model.name,
            owner_id: db_model.created_by,
            picture: db_model.picture,
        })
        .collect();

    Ok(Json(ResponseDataModels { data: models }))
}

pub async fn get_all_owner_models(
    Extension(user): Extension<UserModel>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<ResponseDataModels>, AppError> {
    let models = model_queries::get_all_owner_models(&db, user.id, false)
        .await?
        .into_iter()
        .map(|db_model| ResponseModel {
            id: db_model.id,
            description: db_model.description,
            is_published: db_model.is_published,
            enable_data_sharing: db_model.enable_data_sharing,
            name: db_model.name,
            owner_id: db_model.created_by,
            picture: db_model.picture,
        })
        .collect::<Vec<ResponseModel>>();

    Ok(Json(ResponseDataModels { data: models }))
}
