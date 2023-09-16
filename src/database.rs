use anyhow::Result;
use sqlx::{Pool, Sqlite};

pub async fn setup() -> Result<Pool<Sqlite>> {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
