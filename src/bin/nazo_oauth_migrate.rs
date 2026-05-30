#![forbid(unsafe_code)]

use std::env;

use diesel::{Connection, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn main() -> anyhow::Result<()> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@127.0.0.1:5432/oauth".into())
        .replace("postgresql+psycopg://", "postgresql://");
    let mut connection = PgConnection::establish(&database_url)?;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|error| anyhow::anyhow!("database migration failed: {error}"))?;
    Ok(())
}
