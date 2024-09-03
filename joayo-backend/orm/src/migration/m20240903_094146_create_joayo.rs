use sea_orm_migration::{prelude::*, schema::*};

use super::m20240822_184236_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Joayo {
    Table,
    JoayoId,
    CreatedAt,
    ImageUrl,
    ReferenceUrl,
    UserId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Joayo::Table)
                    .if_not_exists()
                    .col(uuid(Joayo::JoayoId).primary_key())
                    .col(timestamp(Joayo::CreatedAt))
                    .col(string_null(Joayo::ImageUrl))
                    .col(string_null(Joayo::ReferenceUrl))
                    .col(uuid(Joayo::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-joayo-user_id")
                            .from(Joayo::Table, Joayo::UserId)
                            .to(User::Table, User::UserId)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Joayo::Table).to_owned())
            .await
    }
}
