//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "core_twin")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(column_name = "modelId")]
    pub model_id: Uuid,
    #[sea_orm(column_name = "policyId")]
    pub policy_id: Option<Uuid>,
    #[sea_orm(column_name = "typeId")]
    pub type_id: i32,
    #[sea_orm(column_name = "twinStatusId")]
    pub twin_status_id: i32,
    #[sea_orm(column_name = "networkName")]
    pub network_name: Option<String>,
    #[sea_orm(column_name = "twinPort")]
    pub twin_port: Option<i32>,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "createdBy")]
    pub created_by: Uuid,
    #[sea_orm(column_name = "updatedAt")]
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "updatedBy")]
    pub updated_by: Option<Uuid>,
    #[sea_orm(column_name = "deletedAt")]
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(column_name = "deletedBy")]
    pub deleted_by: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::core_model::Entity",
        from = "Column::ModelId",
        to = "super::core_model::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CoreModel,
    #[sea_orm(
        belongs_to = "super::core_model_type::Entity",
        from = "Column::TypeId",
        to = "super::core_model_type::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CoreModelType,
    #[sea_orm(
        belongs_to = "super::core_policy::Entity",
        from = "Column::PolicyId",
        to = "super::core_policy::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CorePolicy,
    #[sea_orm(has_many = "super::core_policy_violation::Entity")]
    CorePolicyViolation,
    #[sea_orm(has_many = "super::core_twin_component::Entity")]
    CoreTwinComponent,
    #[sea_orm(
        belongs_to = "super::core_twin_status::Entity",
        from = "Column::TwinStatusId",
        to = "super::core_twin_status::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CoreTwinStatus,
    #[sea_orm(
        belongs_to = "super::core_user::Entity",
        from = "Column::CreatedBy",
        to = "super::core_user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CoreUser,
}

impl Related<super::core_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreModel.def()
    }
}

impl Related<super::core_model_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreModelType.def()
    }
}

impl Related<super::core_policy::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CorePolicy.def()
    }
}

impl Related<super::core_policy_violation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CorePolicyViolation.def()
    }
}

impl Related<super::core_twin_component::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreTwinComponent.def()
    }
}

impl Related<super::core_twin_status::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreTwinStatus.def()
    }
}

impl Related<super::core_user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}