use axum::{
    extract::{Path, State},
    Extension, Json,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    database::core_user::Model as UserModel, queries::twin_queries, utilities::app_error::AppError,
};

use super::{ResponseTwinDataModel, ResponseTwinModel};

pub async fn get_one_user_twin(
    Path(twin_id): Path<Uuid>,
    Extension(user): Extension<UserModel>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<ResponseTwinDataModel>, AppError> {
    let (twin, twin_status) = twin_queries::get_one_user_twin(&db, twin_id, user.id).await?;

    let status_name = twin_status
        .map(|twin_status| twin_status.name)
        .unwrap_or_else(|| "Stopped".to_string());

    let response_twin = ResponseTwinModel {
        id: twin.id,
        name: twin.name,
        status: status_name,
        twin_port: twin.twin_port,
    };

    Ok(Json(ResponseTwinDataModel {
        data: response_twin,
    }))
}
