use axum::http::{Response, StatusCode};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait, TryIntoModel,
};
use serde_json::json;
use uuid::Uuid;

use crate::database::core_twin::{ActiveModel, Model as CoreTwinModel};
use crate::database::core_user::Model as UserModel;
use crate::database::core_user_subscription;
use crate::database::{
    core_model, core_model_component,
    core_twin::{self, Entity as Models},
    core_twin_component::{
        self, ActiveModel as CompActiveModel, Entity as TwinComponents, Model as CoreTwinCompModel,
    },
    core_twin_status::{
        self, ActiveModel as StatActiveModel, Entity as TwinStats, Model as CoreTwinStatModel,
    },
};
use crate::utilities::app_error::AppError;

//Use Transactions
pub async fn create_twin(
    db: &DatabaseConnection,
    user: &UserModel,
    model: &core_model::Model,
    model_component: Vec<core_model_component::Model>,
) -> Result<(CoreTwinModel, Vec<core_twin_component::Model>), AppError> {
    let mut twin_components: Vec<core_twin_component::Model> = Vec::new();

    let txn = db.begin().await.map_err(|error| {
        eprintln!("Error beginning transaction: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error beginning transaction",
        )
    })?;
    //Get Policy_id by calling policy_queries::get_latest_policy_by_model_id
    let policy_id = crate::queries::policy_queries::get_latest_policy_by_model_id(
        &db,
        model.id.clone(),
        // user.id.clone(),
    )
    .await?
    .id;

    //save Twin
    let new_twin_id = Uuid::new_v4();
    let new_twin = core_twin::ActiveModel {
        id: Set(new_twin_id.clone()),
        model_id: Set(model.id.clone()),
        name: Set(model.name.clone()),
        type_id: Set(model.type_id.clone()),
        policy_id: Set(Some(policy_id)),
        created_by: Set(user.id.clone()),
        updated_by: Set(Some(user.id.clone())),
        ..Default::default()
    };

    let new_twin = new_twin.insert(&txn).await.map_err(|error| {
        eprintln!("Error saving twin: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error saving twin")
    })?;

    //Save Twin Components
    for comp in model_component {
        let new_comp_id = Uuid::new_v4();
        let new_comp = core_twin_component::ActiveModel {
            id: Set(new_comp_id.clone()),
            name: Set(comp.name),
            twin_id: Set(new_twin_id.clone()),
            image_source: Set(comp.image_source),
            is_exposed: Set(comp.is_exposed),
            container_port: Set(comp.container_port),
            component_alias: Set(comp.component_alias),
            created_by: Set(Some(user.id.clone())),
            updated_by: Set(Some(user.id.clone())),
            ..Default::default()
        };

        //save_active_coremodelcomp(db, new_comp).await?;
        let returned_comp = new_comp.insert(&txn).await.map_err(|error| {
            eprintln!("Error saving twin component: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error saving twin component",
            )
        })?;

        twin_components.push(returned_comp);
    }

    //Create entry in core_user_subscription table
    let new_sub_id = Uuid::new_v4();
    let new_subscription = core_user_subscription::ActiveModel {
        id: Set(new_sub_id.clone()),
        user_id: Set(user.id.clone()),
        model_id: Set(model.id.clone()),
        ..Default::default()
    };

    let _sub_respone = new_subscription.insert(&txn).await.map_err(|error| {
        eprintln!("Error saving user subscription: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving user subscription",
        )
    })?;

    txn.commit().await.map_err(|error| {
        eprintln!("Error committing transaction: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error committing transaction",
        )
    })?;

    Ok((new_twin, twin_components))
}

pub async fn save_active_coretwin(
    db: &DatabaseConnection,
    model: core_twin::ActiveModel,
) -> Result<CoreTwinModel, AppError> {
    let model = model.save(db).await.map_err(|error| {
        eprintln!("Error saving twin: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error saving twin")
    })?;

    convert_active_to_model(model)
    //Ok(model)
}

fn convert_active_to_model(
    active_coremodel: core_twin::ActiveModel,
) -> Result<CoreTwinModel, AppError> {
    active_coremodel.try_into_model().map_err(|error| {
        eprintln!("Error converting active model to model: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    })
}

//Save core_twin_component
pub async fn save_active_coretwin_component(
    db: &DatabaseConnection,
    model: core_twin_component::ActiveModel,
) -> Result<CoreTwinCompModel, AppError> {
    let model = model.save(db).await.map_err(|error| {
        eprintln!("Error saving twin component: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving twin component",
        )
    })?;

    convert_active_to_model_comp(model)
    //Ok(model)
}

fn convert_active_to_model_comp(
    active_coremodel: core_twin_component::ActiveModel,
) -> Result<CoreTwinCompModel, AppError> {
    active_coremodel.try_into_model().map_err(|error| {
        eprintln!("Error converting active model to model: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    })
}

pub async fn find_twin_by_id(
    db: &DatabaseConnection,
    id: Uuid,
    user_id: Uuid,
) -> Result<(core_twin::Model, Vec<core_twin_component::Model>), AppError> {
    let model = Models::find_by_id(id)
        .filter(core_twin::Column::CreatedBy.eq(Some(user_id)))
        .filter(core_twin::Column::DeletedAt.is_null())
        .find_with_related(TwinComponents)
        .all(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting twin by id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your digital twin model",
            )
        })?;

    model.clone().into_iter().next().ok_or_else(|| {
        eprintln!("Could not find digital twin model by id");
        AppError::new(StatusCode::NOT_FOUND, "not found")
    })
}

pub async fn find_twin_by_model_id(
    db: &DatabaseConnection,
    model_id: Uuid,
    user_id: Uuid,
) -> Result<(core_twin::Model, Vec<core_twin_component::Model>), AppError> {
    let model = Models::find()
        .filter(core_twin::Column::CreatedBy.eq(Some(user_id)))
        .filter(core_twin::Column::ModelId.eq(Some(model_id)))
        .filter(core_twin::Column::DeletedAt.is_null())
        .find_with_related(TwinComponents)
        .all(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting twin by model id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your digital twin model",
            )
        })?;

    model.clone().into_iter().next().ok_or_else(|| {
        eprintln!("Could not find digital twin model by model id");
        AppError::new(StatusCode::NOT_FOUND, "not found")
    })
}

pub async fn get_all_user_twins(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<(core_twin::Model, Option<core_twin_status::Model>)>, AppError> {
    let model = Models::find()
        .filter(core_twin::Column::CreatedBy.eq(Some(user_id)))
        .filter(core_twin::Column::DeletedAt.is_null())
        .find_also_related(TwinStats)
        .all(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting twins by user id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your twin models",
            )
        })?;

    // model.clone().into_iter().next().ok_or_else(|| {
    //     eprintln!("Could not find digital twin models by user id");
    //     AppError::new(StatusCode::NOT_FOUND, "not found")
    // })
    Ok(model)
}

pub async fn get_one_user_twin(
    db: &DatabaseConnection,
    id: Uuid,
    user_id: Uuid,
) -> Result<(core_twin::Model, Option<core_twin_status::Model>), AppError> {
    let model = Models::find_by_id(id)
        .filter(core_twin::Column::CreatedBy.eq(Some(user_id)))
        .filter(core_twin::Column::DeletedAt.is_null())
        .find_also_related(TwinStats)
        .all(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting twins by user id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your twin models",
            )
        })?;

    model.clone().into_iter().next().ok_or_else(|| {
        eprintln!("Could not find digital twin models by user id");
        AppError::new(StatusCode::NOT_FOUND, "not found")
    })
    // Ok(model)
}

pub async fn delete_twin(
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
    let (model, model_components) = find_twin_by_id(db, id, user_id).await?; //.into_active_model();
    let mut model: ActiveModel = model.into();
    model.twin_status_id = Set(4); //Set to deleted
    model.deleted_by = Set(Some(user_id.clone()));
    model.deleted_at = Set(Some(chrono::Utc::now().into()));

    model.save(&txn).await.map_err(|error| {
        eprintln!("Error deleting digital twin model: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error deleting digital twin model",
        )
    })?;

    //Delete Model Components
    for comp in model_components {
        let mut model_comp: CompActiveModel = comp.into();
        model_comp.deleted_by = Set(Some(user_id.clone()));
        model_comp.deleted_at = Set(Some(chrono::Utc::now().into()));

        model_comp.save(&txn).await.map_err(|error| {
            eprintln!("Error deleting diigtal twin model component: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error deleting digital twin model component",
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

pub async fn delete_twin_by_models(
    db: &DatabaseConnection,
    model: core_twin::Model,
    model_components: Vec<core_twin_component::Model>,
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
    let mut model: ActiveModel = model.into();
    model.twin_status_id = Set(4); //Set to deleted
    model.deleted_by = Set(Some(user_id.clone()));
    model.deleted_at = Set(Some(chrono::Utc::now().into()));

    model.save(&txn).await.map_err(|error| {
        eprintln!("Error deleting digital twin model: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error deleting digital twin model",
        )
    })?;

    //Delete Model Components
    for comp in model_components {
        let mut model_comp: CompActiveModel = comp.into();
        model_comp.deleted_by = Set(Some(user_id.clone()));
        model_comp.deleted_at = Set(Some(chrono::Utc::now().into()));

        model_comp.save(&txn).await.map_err(|error| {
            eprintln!("Error deleting diigtal twin model component: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error deleting digital twin model component",
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
