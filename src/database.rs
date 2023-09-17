use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{migrate, Pool, Sqlite};

pub async fn setup() -> Result<Pool<Sqlite>> {
    let pool = Pool::<Sqlite>::connect("sqlite::memory:").await?;

    migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct TestRow {
    pub id: String,
    pub url: String,
    pub method: String,
    pub content_type: String,
    pub status: String,
    pub body: Option<String>,
    pub created_at: String,
    pub finished_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct TestResultsRow {
    pub id: String,
    pub test_id: String,
    pub second: i64,
    pub requests: i64,
    pub error_codes: String,
}
