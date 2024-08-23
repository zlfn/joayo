use sea_orm_migration::prelude::*;

pub struct Migrator;
mod m20240822_184236_create_user;
mod m20240822_184335_create_password_identity;
mod m20240823_150240_create_session;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240822_184236_create_user::Migration),
            Box::new(m20240822_184335_create_password_identity::Migration),
            Box::new(m20240823_150240_create_session::Migration),
        ]
    }
}
