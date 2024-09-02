use sea_orm_migration::{prelude::*, schema::*};

use super::m20240822_184236_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum PasswordIdentity {
    Table,
    UserId,
    PasswordHash,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PasswordIdentity::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::UserId)
                        .uuid()
                        .not_null()
                        .primary_key()
                    )
                    .col(string(PasswordIdentity::PasswordHash))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-password_identity-user_id")
                            .from(PasswordIdentity::Table, PasswordIdentity::UserId)
                            .to(User::Table, User::UserId)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PasswordIdentity::Table).to_owned())
            .await
    }
}
