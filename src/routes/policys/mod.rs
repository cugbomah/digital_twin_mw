use serde::{Deserialize, Serialize};
use uuid::Uuid;

use self::{
    create_policy_action_extractor::ValidateCreatePolicyAction,
    create_policy_extractor::ValidateCreatePolicy,
};

pub mod create_policy;
pub mod create_policy_action_extractor;
pub mod create_policy_extractor;
pub mod get_latest_policy;

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestPolicyValidated {
    pub policy_info: ValidateCreatePolicy,
    pub policy_action_info: Vec<ValidateCreatePolicyAction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponsePolicyAction {
    pub id: Uuid,
    pub end_point: String,
    pub description: String,
    pub end_point_verb: String,
    pub action_count: Option<i32>,
    pub reset_frequency: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponsePolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub policy_version: Option<i32>,
    pub block_after: Option<i32>,
    pub policy_actions: Vec<ResponsePolicyAction>,
}
