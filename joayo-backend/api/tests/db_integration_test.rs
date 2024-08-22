use sea_orm::{ActiveValue, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

#[tokio::test]
async fn db_connect_test() {

    let db: DatabaseConnection = Database::connect("postgres://joayo:joayo@0.0.0.0/test")
        .await
        .expect("Failed to connect database");

    /*Migrator::fresh(&db)
        .await
        .expect("Failed to migrate database");*/
}
