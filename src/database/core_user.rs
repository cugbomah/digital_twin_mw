//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "core_user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "firstName")]
    pub first_name: String,
    #[sea_orm(column_name = "lastName")]
    pub last_name: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password: String,
    #[sea_orm(column_name = "roleId")]
    pub role_id: Uuid,
    pub status: bool,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "createdBy")]
    pub created_by: Option<Uuid>,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(column_name = "updatedBy")]
    pub updated_by: Option<Uuid>,
    #[sea_orm(column_name = "deletedAt")]
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(column_name = "deletedBy")]
    pub deleted_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::core_policy::Entity")]
    CorePolicy,
    #[sea_orm(has_many = "super::core_policy_action::Entity")]
    CorePolicyAction,
    #[sea_orm(has_many = "super::core_policy_violation::Entity")]
    CorePolicyViolation,
    #[sea_orm(
        belongs_to = "super::core_role::Entity",
        from = "Column::RoleId",
        to = "super::core_role::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CoreRole,
    #[sea_orm(has_many = "super::core_twin::Entity")]
    CoreTwin,
    #[sea_orm(has_many = "super::core_user_subscription::Entity")]
    CoreUserSubscription,
}

impl Related<super::core_policy::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CorePolicy.def()
    }
}

impl Related<super::core_policy_action::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CorePolicyAction.def()
    }
}

impl Related<super::core_policy_violation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CorePolicyViolation.def()
    }
}

impl Related<super::core_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreRole.def()
    }
}

impl Related<super::core_twin::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreTwin.def()
    }
}

impl Related<super::core_user_subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreUserSubscription.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
