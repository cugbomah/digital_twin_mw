use axum::{extract::State, http::StatusCode, Extension, Json};
use sea_orm::DatabaseConnection;

use crate::{database::core_user, queries::model_queries, utilities::app_error::AppError};

use super::{
    create_model_component_extractor::ValidateCreateModelComponent,
    create_model_extractor::ValidateCreateModel, RequestModelValidated, ResponseModel,
};

// pub async fn create_model(
//     Extension(user): Extension<core_user::Model>,
//     State(db): State<DatabaseConnection>,
//     Json(model): Json<ValidateCreateModel>,
// ) -> Result<(StatusCode, Json<ResponseModel>), AppError> {
//     model_queries::create_model(&db, &user, model)
//         .await
//         .map(|model| {
//             (
//                 StatusCode::CREATED,
//                 Json(ResponseModel {
//                     id: model.id,
//                     name: model.name,
//                     description: model.description,
//                     owner_id: model.created_by,
//                     picture: model.picture,
//                 }),
//             )
//         })
// }

//Trying to handle as a transaction
pub async fn create_model(
    Extension(user): Extension<core_user::Model>,
    State(db): State<DatabaseConnection>,
    Json(model): Json<RequestModelValidated>,
) -> Result<(StatusCode, Json<ResponseModel>), AppError> {
    model_queries::create_model(&db, &user, model)
        .await
        .map(|model| {
            (
                StatusCode::CREATED,
                Json(ResponseModel {
                    id: model.id,
                    name: model.name,
                    description: model.description,
                    is_published: model.is_published,
                    enable_data_sharing: model.enable_data_sharing,
                    owner_id: model.created_by,
                    picture: model.picture,
                }),
            )
        })

    // dbg!(&model.model_info);
    // dbg!(&model.comp_info);

    //iterate through model.comp_info and create a model for each one
    //then create a model component for each one
    //then return the model
    //
    //if any of the model components fail, rollback the transaction
    //if the model fails, rollback the transaction
    //if the model components succeed but the model fails, rollback the transaction
    //if the model succeeds but the model components fail, rollback the transaction
    //if the model and model components succeed, commit the transaction
}
