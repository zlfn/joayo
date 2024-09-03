use sea_orm_migration::{prelude::*, schema::*};

use super::m20240903_094146_create_joayo::Joayo;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum UploadResult {
    Table,
    JoayoId,
    OriginalSize,
    FileSize,
    ObjectUrl,
    Result,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UploadResult::Table)
                    .if_not_exists()
                    .col(uuid(UploadResult::JoayoId).primary_key())
                    .col(integer(UploadResult::OriginalSize))
                    .col(integer_null(UploadResult::FileSize))
                    .col(string_null(UploadResult::ObjectUrl))
                    .col(string(UploadResult::Result))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-upload_result-joayo_id")
                            .from(UploadResult::Table, UploadResult::JoayoId)
                            .to(Joayo::Table, Joayo::JoayoId)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UploadResult::Table).to_owned())
            .await
    }
}
