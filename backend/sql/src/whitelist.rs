use sqlx::{Executor, Postgres, Result};
pub struct WhiteList {
    pub source_id: i64,
    pub target_id: i64,
}

impl WhiteList {
    pub async fn save<'a, E: Executor<'a, Database = Postgres>>(
        conn: E,
        whitelist: WhiteList,
    ) -> Result<()> {
        sqlx::query(
            r#"
        INSERT INTO "whitelist"
        (
            source_id,
            target_id
        )
        VALUES ($1, $2)
        ON CONFLICT (source_id, target_id)
        DO NOTHING
        "#,
        )
        .bind(whitelist.source_id)
        .bind(whitelist.target_id)
        .execute(conn)
        .await?;
        Ok(())
    }
    pub async fn find_by_source_id<'a, E: Executor<'a, Database = Postgres>>(
        conn: E,
        source_id: i64,
    ) -> Result<Vec<WhiteList>> {
        sqlx::query_as!(
            WhiteList,
            r#"SELECT * FROM "whitelist" WHERE source_id=$1"#,
            source_id
        )
        .fetch_all(conn)
        .await
    }
}
