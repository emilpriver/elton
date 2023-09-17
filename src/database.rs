use anyhow::Result;
use sqlx::{migrate, Pool, Sqlite};

pub async fn setup() -> Result<Pool<Sqlite>> {
    let pool = Pool::<Sqlite>::connect("sqlite::memory:").await?;

    migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
