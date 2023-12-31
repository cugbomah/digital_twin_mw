//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "core_shared_model_data")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_name = "modelId")]
    pub model_id: Uuid,
    #[sea_orm(column_name = "inputData")]
    pub input_data: Option<String>,
    #[sea_orm(column_name = "outputResponse")]
    pub output_response: Option<String>,
    #[sea_orm(column_name = "createdAt")]
    pub created_at: DateTimeWithTimeZone,
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
