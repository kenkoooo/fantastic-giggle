use sqlx::{PgPool, Result};

pub async fn connect_to_test_sql() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPool::connect(&database_url).await?;
    sqlx::query("DELETE FROM relationship")
        .execute(&pool)
        .await?;
    Ok(pool)
}
