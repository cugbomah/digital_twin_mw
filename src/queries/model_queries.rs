use axum::http::{Response, StatusCode};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait, TryIntoModel,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    database::core_model::{self, Entity as Models, Model as CoreModel},
    database::core_model_component::{self, Entity as ModelComponents},
    database::{
        core_model::ActiveModel, core_model_component::ActiveModel as CompActiveModel,
        core_user::Model as UserModel,
    },
    routes::models::RequestModelValidated,
    utilities::app_error::AppError,
};

//Use Transactions
pub async fn create_model(
    db: &DatabaseConnection,
    user: &UserModel,
    model: RequestModelValidated,
) -> Result<CoreModel, AppError> {
    let txn = db.begin().await.map_err(|error| {
        eprintln!("Error beginning transaction: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error beginning transaction",
        )
    })?;
    //save model
    let new_model_id = Uuid::new_v4();
    let mut new_model = core_model::ActiveModel {
        id: Set(new_model_id.clone()),
        name: Set(model.model_info.name.unwrap()),
        description: Set(model.model_info.description.unwrap()),
        type_id: Set(model.model_info.type_id.unwrap()),
        created_by: Set(Some(user.id.clone())),
        updated_by: Set(Some(user.id.clone())),
        ..Default::default()
    };

    if let Some(picture) = model.model_info.picture {
        new_model.picture = Set(Some(picture));
    }

    if let Some(enable_data_sharing) = model.model_info.enable_data_sharing {
        new_model.enable_data_sharing = Set(enable_data_sharing);
    }

    let new_model = new_model.insert(&txn).await.map_err(|error| {
        eprintln!("Error saving model: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error saving model")
    })?;

    //Save Model Components
    for comp in model.comp_info {
        let new_comp_id = Uuid::new_v4();
        let mut new_comp = core_model_component::ActiveModel {
            id: Set(new_comp_id.clone()),
            name: Set(comp.name.unwrap()),
            image_source: Set(comp.image_source.unwrap()),
            model_id: Set(new_model_id.clone()),
            created_by: Set(Some(user.id.clone())),
            updated_by: Set(Some(user.id.clone())),
            ..Default::default()
        };

        //Check if container_port is not null, then save it
        if let Some(container_port) = comp.container_port {
            new_comp.container_port = Set(Some(container_port));

            //if is exposed is not null, save it
            if let Some(is_exposed) = comp.is_exposed {
                new_comp.is_exposed = Set(is_exposed);
            } else {
                //if is_exposed is null, set it to false
                new_comp.is_exposed = Set(false);
            }
        }

        //Check if component alias is not null, then save it
        if let Some(component_alias) = comp.component_alias {
            new_comp.component_alias = Set(Some(component_alias));
        }

        //save_active_coremodelcomp(db, new_comp).await?;
        new_comp.insert(&txn).await.map_err(|error| {
            eprintln!("Error saving model component: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error saving model component",
            )
        })?;
    }

    txn.commit().await.map_err(|error| {
        eprintln!("Error committing transaction: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error committing transaction",
        )
    })?;

    Ok(new_model)
}

pub async fn get_all_published_models(db: &DatabaseConnection) -> Result<Vec<CoreModel>, AppError> {
    let mut query = Models::find().filter(core_model::Column::IsPublished.eq(Some(true)));
    query = query.filter(core_model::Column::DeletedAt.is_null());

    query.all(db).await.map_err(|error| {
        eprintln!("Error getting all published models: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting all published models",
        )
    })
}

pub async fn get_all_owner_models(
    db: &DatabaseConnection,
    owner_id: Uuid,
    get_deleted: bool,
) -> Result<Vec<CoreModel>, AppError> {
    let mut query = Models::find().filter(core_model::Column::CreatedBy.eq(Some(owner_id)));

    if !get_deleted {
        query = query.filter(core_model::Column::DeletedAt.is_null());
    }

    query.all(db).await.map_err(|error| {
        eprintln!("Error getting all your models: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting all your models",
        )
    })
}

pub async fn save_active_coremodel(
    db: &DatabaseConnection,
    model: core_model::ActiveModel,
) -> Result<CoreModel, AppError> {
    let model = model.save(db).await.map_err(|error| {
        eprintln!("Error saving model: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error saving model")
    })?;

    convert_active_to_model(model)
    //Ok(model)
}

pub async fn find_model_by_id(
    db: &DatabaseConnection,
    id: Uuid,
    user_id: Uuid,
) -> Result<(core_model::Model, Vec<core_model_component::Model>), AppError> {
    let model = Models::find_by_id(id)
        .filter(core_model::Column::CreatedBy.eq(Some(user_id)))
        .filter(core_model::Column::DeletedAt.is_null())
        .find_with_related(ModelComponents)
        .all(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting model by id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your model",
            )
        })?;

    model.clone().into_iter().next().ok_or_else(|| {
        eprintln!("Could not find model by id");
        AppError::new(StatusCode::NOT_FOUND, "not found")
    })
}

pub async fn find_pubslished_model_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<(core_model::Model, Vec<core_model_component::Model>), AppError> {
    let model = Models::find_by_id(id)
        .filter(core_model::Column::DeletedAt.is_null())
        .filter(core_model::Column::IsPublished.eq(Some(true)))
        .find_with_related(ModelComponents)
        .all(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting model by id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your model",
            )
        })?;

    model.clone().into_iter().next().ok_or_else(|| {
        eprintln!("Could not find model by id");
        AppError::new(StatusCode::NOT_FOUND, "not found")
    })
}

fn convert_active_to_model(
    active_coremodel: core_model::ActiveModel,
) -> Result<CoreModel, AppError> {
    active_coremodel.try_into_model().map_err(|error| {
        eprintln!("Error converting active model to model: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    })
}

pub async fn delete_model(
    db: &DatabaseConnection,
    id: Uuid,
    user_id: Uuid,
) -> Result<Response<String>, AppError> {
    let txn = db.begin().await.map_err(|error| {
        eprintln!("Error beginning transaction: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error beginning transaction",
        )
    })?;

    //Get Model Components
    let (model, model_components) = find_model_by_id(db, id, user_id).await?; //.into_active_model();
    let mut model: ActiveModel = model.into();
    model.name = Set(model.name.unwrap().to_string()
        + "_"
        + &chrono::Utc::now().timestamp_micros().to_string());
    model.is_published = Set(false);
    model.deleted_by = Set(Some(user_id.clone()));
    model.deleted_at = Set(Some(chrono::Utc::now().into()));

    model.save(&txn).await.map_err(|error| {
        eprintln!("Error saving model component: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving model component",
        )
    })?;

    //Delete Model Components
    for comp in model_components {
        let mut model_comp: CompActiveModel = comp.into();
        model_comp.name = Set(model_comp.name.unwrap().to_string()
            + "_"
            + &chrono::Utc::now().timestamp_micros().to_string());
        model_comp.image_source = Set(model_comp.image_source.unwrap().to_string()
            + "_"
            + &chrono::Utc::now().timestamp_micros().to_string());
        model_comp.deleted_by = Set(Some(user_id.clone()));
        model_comp.deleted_at = Set(Some(chrono::Utc::now().into()));

        model_comp.save(&txn).await.map_err(|error| {
            eprintln!("Error deleting model component: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error deleting model component",
            )
        })?;
    }
    txn.commit().await.map_err(|error| {
        eprintln!("Error committing transaction: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error committing transaction",
        )
    })?;

    let response = Response::new(json!({"status": "success"}).to_string());
    Ok(response)
}
