use sea_orm::{Database, DatabaseConnection, Schema, ConnectionTrait};
use crate::models::target;
use std::path::Path;

pub async fn setup_database() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());

    println!("Database URL: {}", db_url);

    // If it's a file-based SQLite database, ensure the directory exists and create the file
    if db_url.starts_with("sqlite:") && !db_url.contains(":memory:") {
        // Handle both sqlite:// and sqlite:/// formats
        let path_str = if db_url.starts_with("sqlite:///") {
            db_url.strip_prefix("sqlite://").unwrap()
        } else if db_url.starts_with("sqlite://") {
            db_url.strip_prefix("sqlite://").unwrap()
        } else {
            db_url.strip_prefix("sqlite:").unwrap()
        };

        let path = Path::new(path_str);
        println!("Database file path: {:?}", path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            println!("Creating database directory: {:?}", parent);
            std::fs::create_dir_all(parent).map_err(|e| {
                sea_orm::DbErr::Custom(format!("Failed to create database directory '{}': {}", parent.display(), e))
            })?;
        }

        // Create the database file if it doesn't exist
        if !path.exists() {
            println!("Creating database file: {:?}", path);
            std::fs::File::create(path).map_err(|e| {
                sea_orm::DbErr::Custom(format!("Failed to create database file '{}': {}", path.display(), e))
            })?;
        } else {
            println!("Database file already exists: {:?}", path);
        }
    }

    let db = Database::connect(&db_url).await?;

    // Create table if it doesn't exist
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    db.execute(builder.build(schema.create_table_from_entity(target::Entity).if_not_exists())).await?;

    Ok(db)
}
