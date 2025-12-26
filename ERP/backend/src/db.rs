use sqlx::{Pool, Sqlite, SqlitePool};
use std::time::Duration;

pub type DbPool = Pool<Sqlite>;

/// Criar connection pool para SQLite
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    tracing::info!("Creating SQLite database connection pool...");

    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await
}

/// Executar migrations
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    tracing::info!("Running database migrations...");

    // Criar tabela tenants
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS tenants (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            domain TEXT UNIQUE NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            plan TEXT NOT NULL DEFAULT 'startup',
            max_users INTEGER NOT NULL DEFAULT 10,
            storage_limit_gb INTEGER NOT NULL DEFAULT 100,
            settings TEXT DEFAULT '{}',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )
    "#)
    .execute(pool)
    .await?;

    // Criar tabela users
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            email TEXT NOT NULL,
            password_hash TEXT,
            name TEXT NOT NULL,
            avatar_url TEXT,
            phone TEXT,
            language TEXT DEFAULT 'pt-BR',
            timezone TEXT DEFAULT 'America/Sao_Paulo',
            status TEXT NOT NULL DEFAULT 'active',
            last_login_at TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
            UNIQUE(tenant_id, email)
        )
    "#)
    .execute(pool)
    .await?;

    tracing::info!("Migrations completed successfully");
    Ok(())
}

/// Health check do banco de dados
pub async fn health_check(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;
    Ok(())
}

/// Estatísticas do connection pool
pub fn pool_stats(pool: &DbPool) -> PoolStats {
    PoolStats {
        size: pool.size(),
        idle: pool.num_idle(),
        active: pool.size() - (pool.num_idle() as u32),
    }
}

#[derive(Debug, serde::Serialize)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
    pub active: u32,
}
