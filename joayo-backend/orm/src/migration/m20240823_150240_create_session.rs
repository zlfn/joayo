use sea_orm_migration::{prelude::*, schema::*};

use super::m20240822_184236_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Session {
    Table,
    SessionId,
    UserId,
    CreatedAt,
    ExpiresAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Session::SessionId)
                        .uuid()
                        .not_null()
                        .primary_key()
                    )
                    .col(ColumnDef::new(Session::UserId)
                        .uuid()
                        .not_null()
                    )
                    .col(timestamp(Session::CreatedAt))
                    .col(timestamp(Session::ExpiresAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-session-user_id")
                            .from(Session::Table, Session::UserId)
                            .to(User::Table, User::UserId)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await
    }
}
