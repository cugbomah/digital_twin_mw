//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "core_model_component")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(column_name = "componentAlias")]
    pub component_alias: Option<String>,
    #[sea_orm(column_name = "imageSource", unique)]
    pub image_source: String,
    #[sea_orm(column_name = "containerPort")]
    pub container_port: Option<i32>,
    #[sea_orm(column_name = "isExposed")]
    pub is_exposed: bool,
    #[sea_orm(column_name = "modelId")]
    pub model_id: Uuid,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(column_name = "createdBy")]
    pub created_by: Option<Uuid>,
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
}

impl Related<super::core_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreModel.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
