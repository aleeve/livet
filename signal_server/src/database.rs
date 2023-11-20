/// Inspired by this blogpost:
/// https://tms-dev-blog.com/rust-sqlx-basics-with-sqlite/#Creating_an_SQLite_database

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool, Pool};
use tracing::{log::{log, Level}, instrument};
use anyhow::{Result, bail};
use protocol::{Musician, Band};

const DB_URL: &str = "sqlite://sqlite.db";

#[instrument]
pub async fn setup_database() -> Result<Pool<Sqlite>>{
    let db = create_database().await?;
    add_tables(&db).await;
    add_musician(Musician { id: 1, name: Some("Alex".into()) }, &db).await;
    Ok(db)
}

async fn create_database() -> Result<Pool<Sqlite>> {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        log!(Level::Info, "Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => log!(Level::Info,"Create db success"),
            Err(error) => bail!("Couldn't create database: {}", error),
        }
    } else {
        log!(Level::Info, "Database already exists");
    }
    let pool: Pool<Sqlite> = SqlitePool::connect(DB_URL).await?;
    Ok(pool)
}
async fn add_tables(db: &Pool<Sqlite>) -> Result<()> {
    log!(Level::Info, "Adding tables");
    let query = include_str!("sql/create_table.sql");
    let result = sqlx::query(query).execute(db).await?;
    Ok(())
}

async fn add_musician(musician: Musician, db: &Pool<Sqlite>) -> Result<()> {
    let result = sqlx::query!("
        insert or replace into musicians
        values ($1,$2)
    ", musician.id, musician.name).execute(db).await?;
    log!(Level::Info, "{:?}", result);
    Ok(())
}

async fn add_band(band: Band, db: &Pool<Sqlite>) -> Result<()> {
    let result = sqlx::query!("
        insert or replace into musicians
        values ($1,$2)
    ", band.id, band.name).execute(db).await?;
    log!(Level::Info, "{:?}", result);
    Ok(())
}

async fn add_session(band: Band, db: &Pool<Sqlite>) -> Result<()> {
    let result = sqlx::query!("
        insert or replace into musicians
        values ($1,$2)
    ", band.id, band.name).execute(db).await?;
    log!(Level::Info, "{:?}", result);
    Ok(())
}


