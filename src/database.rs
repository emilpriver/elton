use anyhow::Result;
use sqlx::SqlitePool;

pub async fn setup() -> Result<SqlitePool> {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
