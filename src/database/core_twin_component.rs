//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "core_twin_component")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(column_name = "componentAlias")]
    pub component_alias: Option<String>,
    #[sea_orm(column_name = "containerPort")]
    pub container_port: Option<i32>,
    #[sea_orm(column_name = "containerName")]
    pub container_name: Option<String>,
    #[sea_orm(column_name = "hostPort")]
    pub host_port: Option<i32>,
    #[sea_orm(column_name = "isExposed")]
    pub is_exposed: bool,
    #[sea_orm(column_name = "twinId")]
    pub twin_id: Uuid,
    #[sea_orm(column_name = "imageSource")]
    pub image_source: String,
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
        belongs_to = "super::core_twin::Entity",
        from = "Column::TwinId",
        to = "super::core_twin::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CoreTwin,
}

impl Related<super::core_twin::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CoreTwin.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}