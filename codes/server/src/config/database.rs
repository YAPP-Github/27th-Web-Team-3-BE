use crate::domain::{
    member::entity::{member, member_response, member_retro, member_retro_room},
    retrospect::entity::{
        response, response_comment, response_like, retro_reference, retro_room, retrospect,
    },
    team::entity::{member_team, team},
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema, Statement};
use std::env;
use tracing::info;

pub async fn establish_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    info!("Successfully connected to the database.");

    // Check if schema update is enabled
    let should_update_schema = env::var("DB_SCHEMA_UPDATE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if should_update_schema {
        // Auto-create tables (Schema Sync)
        create_tables(&db).await?;
    } else {
        info!("Skipping database schema synchronization (DB_SCHEMA_UPDATE is not true).");
    }

    Ok(db)
}

async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    info!("Starting database schema synchronization...");

    // List of entities to create
    // Order matters for foreign keys! (Parent first, then Child)

    // 1. Independent Entities
    create_table_if_not_exists(db, &schema, member::Entity).await?;
    create_table_if_not_exists(db, &schema, retro_room::Entity).await?;
    create_table_if_not_exists(db, &schema, team::Entity).await?;

    // 2. Dependent Entities (Level 1)
    create_table_if_not_exists(db, &schema, retrospect::Entity).await?;
    create_table_if_not_exists(db, &schema, member_team::Entity).await?;

    // 3. Dependent Entities (Level 2)
    create_table_if_not_exists(db, &schema, response::Entity).await?;
    create_table_if_not_exists(db, &schema, retro_reference::Entity).await?;
    create_table_if_not_exists(db, &schema, member_retro_room::Entity).await?;

    // 4. Dependent Entities (Level 3 & Join Tables)
    create_table_if_not_exists(db, &schema, response_comment::Entity).await?;
    create_table_if_not_exists(db, &schema, response_like::Entity).await?;
    create_table_if_not_exists(db, &schema, member_response::Entity).await?;
    create_table_if_not_exists(db, &schema, member_retro::Entity).await?;

    info!("Database schema synchronization completed.");
    Ok(())
}

async fn create_table_if_not_exists<E>(
    db: &DatabaseConnection,
    schema: &Schema,
    entity: E,
) -> Result<(), DbErr>
where
    E: sea_orm::EntityTrait,
{
    let backend = db.get_database_backend();
    let create_stmt: Statement =
        backend.build(schema.create_table_from_entity(entity).if_not_exists());

    match db.execute(create_stmt).await {
        Ok(_) => {
            // Log success (optional, verbose)
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to create table: {}", e);
            Err(e)
        }
    }
}
