use axum::http::StatusCode;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use uuid::Uuid;

use crate::routes::policys::{RequestPolicyValidated, ResponsePolicy, ResponsePolicyAction};
use crate::utilities::redis_connection_wrapper::RedisConnWrapper;
use crate::utilities::redis_helper::store_token_in_redis;
use crate::{
    database::core_policy::{self, ActiveModel, Entity as Models, Model as CorePolicy},
    database::core_policy_action::{
        self, ActiveModel as PolicyActActiveModel, Entity as PolicyActions,
        Model as PolicyActionModel,
    },
    database::core_user::Model as UserModel,
    routes::models::RequestModelValidated,
    utilities::app_error::AppError,
};

pub async fn find_policy_by_model_id(
    db: &DatabaseConnection,
    model_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<(core_policy::Model, Vec<core_policy_action::Model>)>, AppError> {
    let model = Models::find()
        .order_by_desc(core_policy::Column::PolicyVersion)
        // .filter(core_policy::Column::CreatedBy.eq(Some(user_id)))
        .filter(core_policy::Column::ModelId.eq(Some(model_id)))
        .find_with_related(PolicyActions)
        .all(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting policies by model id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your policies by the model id",
            )
        })?;

    // model.clone().into_iter().next().ok_or_else(|| {
    //     eprintln!("Could not find digital twin model by model id");
    //     AppError::new(StatusCode::NOT_FOUND, "not found")
    // })

    // for returned_model in model.clone() {
    //     println!("Policy Version: {:?}", returned_model.0.policy_version);
    // }

    //Print the policy_version of the first model if the model is not empty
    if !model.is_empty() {
        println!("Policy Version: {:?}", model[0].0.policy_version);
    }
    // println!("Policy Version: {:?}", model[0].0.policy_version);

    Ok(model)
}

pub async fn get_latest_policy_by_model_id(
    db: &DatabaseConnection,
    model_id: Uuid,
    // user_id: Uuid,
) -> Result<core_policy::Model, AppError> {
    let model = Models::find()
        .order_by_desc(core_policy::Column::PolicyVersion)
        // .filter(core_policy::Column::CreatedBy.eq(Some(user_id)))
        .filter(core_policy::Column::ModelId.eq(Some(model_id)))
        // .find_with_related(PolicyActions)
        .all(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting policies by model id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting your policies by the model id",
            )
        })?;

    Ok(model[0].clone())
}

//Use Transactions
pub async fn create_model_policy(
    db: &DatabaseConnection,
    model_id: Uuid,
    user: &UserModel,
    model: RequestPolicyValidated,
    redis_url: RedisConnWrapper,
) -> Result<CorePolicy, AppError> {
    let txn = db.begin().await.map_err(|error| {
        eprintln!("Error beginning transaction: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error beginning transaction",
        )
    })?;
    //Get Policies by model_id
    let policies = find_policy_by_model_id(db, model_id, user.id).await?;

    //Assign the length of policies to policy_version if policies is not empty
    let policy_version = if !policies.is_empty() {
        Some(policies.len() as i32 + 1)
    } else {
        Some(1)
    };

    //save model
    let new_policy_id = Uuid::new_v4();
    let mut new_model = core_policy::ActiveModel {
        id: Set(new_policy_id.clone()),
        name: Set(model.policy_info.name.unwrap()),
        description: Set(model.policy_info.description.unwrap()),
        policy_version: Set(policy_version.unwrap()),
        model_id: Set(model_id),
        created_by: Set(user.id.clone()),
        updated_by: Set(Some(user.id.clone())),
        ..Default::default()
    };

    if let Some(block_after) = model.policy_info.block_after {
        new_model.block_after = Set(block_after);
    }

    let new_model = new_model.insert(&txn).await.map_err(|error| {
        eprintln!("Error saving model policy: {:?}", error);
        AppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving model policy",
        )
    })?;

    //Save Model Components
    for comp in model.policy_action_info {
        let new_comp_id = Uuid::new_v4();
        let end_point = comp.end_point.unwrap().to_lowercase();
        let action_count_log: i32;

        let mut new_comp = PolicyActActiveModel {
            id: Set(new_comp_id.clone()),
            policy_id: Set(new_policy_id.clone()),
            end_point: Set(end_point.clone()),
            description: Set(comp.description.unwrap()),
            end_point_verb: Set(comp.end_point_verb.clone().unwrap().to_uppercase()),

            created_by: Set(user.id.clone()),
            updated_by: Set(Some(user.id.clone())),
            ..Default::default()
        };

        //Check if action_count is not null, then save it
        if let Some(action_count) = comp.action_count {
            new_comp.action_count = Set(action_count);
            action_count_log = action_count.clone();
        } else {
            action_count_log = 0;
        }

        //Check if reset_frequency is not null, then save it
        if let Some(reset_frequency_id) = comp.reset_frequency_id {
            new_comp.reset_frequency_id = Set(reset_frequency_id);
        }

        //save_active_coremodelcomp(db, new_comp).await?;
        new_comp.insert(&txn).await.map_err(|error| {
            eprintln!("Error saving model policy action: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error saving model policy action",
            )
        })?;

        //Save Policy Action Access Counts to Redis
        let mut store_key = "Policy:Models:".to_string();
        store_key = store_key + &new_policy_id.clone().to_string();
        store_key = store_key + ":Access:" + &end_point;
        let _redis_response = store_token_in_redis(
            redis_url.clone(),
            store_key.clone(),
            action_count_log.to_string(),
            usize::MAX,
        )
        .await?;

        //Save Policy Action Verb to Redis
        let mut store_key_verb = "Policy:Models:".to_string();
        store_key_verb = store_key_verb + &new_policy_id.clone().to_string();
        store_key_verb = store_key_verb + ":Verb:" + &end_point;
        let _redis_response = store_token_in_redis(
            redis_url.clone(),
            store_key_verb.clone(),
            comp.end_point_verb.unwrap().to_uppercase(),
            usize::MAX,
        )
        .await?;
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

pub async fn get_policies(
    db: &DatabaseConnection,
    model_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<ResponsePolicy>, AppError> {
    //Get latest model policy by calling policy_queries::find_policy_by_model_id and returning the first element of the vector
    let model = find_policy_by_model_id(&db, model_id, user_id).await?;

    //deconstruct model into policy and policy_actions
    let (policy, policy_actions) = model.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();

    //Iterate through policy and policy_actions, replacing policy.policy_actions with the values of an inner iteration of policy_actions mapped to ResponsePolicyAction, and collect into a new ResponsePolicy vector
    let policies = policy
        .into_iter()
        .zip(policy_actions.into_iter())
        .map(|(policy, policy_actions)| {
            //Extract policy_actions into a vector of ResponsePolicyAction
            let policy_actions = policy_actions
                .into_iter()
                .map(|policy_action| ResponsePolicyAction {
                    id: policy_action.id,
                    end_point: policy_action.end_point,
                    description: policy_action.description,
                    end_point_verb: policy_action.end_point_verb,
                    action_count: Some(policy_action.action_count),
                    reset_frequency: Some(policy_action.reset_frequency_id.to_owned().to_string()),
                })
                .collect::<Vec<ResponsePolicyAction>>();

            ResponsePolicy {
                id: policy.id,
                name: policy.name,
                description: policy.description,
                policy_version: Some(policy.policy_version),
                block_after: Some(policy.block_after),
                policy_actions,
            }
        })
        .collect::<Vec<ResponsePolicy>>();

    Ok(policies)
}

pub async fn get_policy_action_by_policyid_and_endpoint(
    db: &DatabaseConnection,
    policy_id: uuid::Uuid,
    end_point: String,
) -> Result<PolicyActionModel, AppError> {
    let policy_action = PolicyActions::find()
        .filter(core_policy_action::Column::DeletedAt.is_null())
        .filter(
            core_policy_action::Column::PolicyId
                .eq(policy_id)
                .and(core_policy_action::Column::EndPoint.eq(end_point)),
        )
        .one(db)
        .await
        .map_err(|error| {
            eprintln!("Error getting Policy Action by policy_id: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "There was an error getting the policy action, please try again",
            )
        })?;

    policy_action.ok_or_else(|| {
        eprintln!("Could not find Policy Action by policy_id");
        AppError::new(StatusCode::NOT_FOUND, "not found")
    })
}
