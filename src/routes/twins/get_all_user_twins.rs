use axum::{extract::State, Extension, Json};
use sea_orm::DatabaseConnection;

use crate::{
    database::core_user::Model as UserModel, queries::twin_queries, utilities::app_error::AppError,
};

use super::{ResponseTwinDataModels, ResponseTwinModel};

pub async fn get_all_user_twins(
    Extension(user): Extension<UserModel>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<ResponseTwinDataModels>, AppError> {
    let model = twin_queries::get_all_user_twins(&db, user.id).await?;

    //deconstruct model into twin and twin_status
    let (twin, twin_status) = model.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();

    //Iterate through twin and twin_status, replacing twin.twin_status_id with twin_status.name, and collect into a new ResponseTwinModel vector
    let twins = twin
        .into_iter()
        .zip(twin_status.into_iter())
        .map(|(twin, twin_status)| {
            //Extract twin_status.name from twin_status
            let status_name = twin_status
                .into_iter()
                .map(|twin_status| twin_status.name)
                .collect::<Vec<String>>()
                .join("");

            // dbg!(status_name);
            ResponseTwinModel {
                id: twin.id,
                name: twin.name,
                status: status_name,
                twin_port: twin.twin_port,
            }
        })
        .collect::<Vec<ResponseTwinModel>>();

    Ok(Json(ResponseTwinDataModels { data: twins }))
}
