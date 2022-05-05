use sqlx::{Executor, Postgres, Result};
pub struct AccessToken {
    pub id: i64,
    pub access_key: String,
    pub access_secret: String,
}

impl AccessToken {
    pub async fn save<'a, E>(conn: E, token: AccessToken) -> Result<()>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query(
            r"
        INSERT INTO access_token
        (
            id,
            access_key,
            access_secret
        )
        VALUES ($1, $2, $3)
        ON CONFLICT (id)
        DO UPDATE
            SET access_key=EXCLUDED.access_key, access_secret=EXCLUDED.access_secret
        ",
        )
        .bind(token.id)
        .bind(token.access_key)
        .bind(token.access_secret)
        .execute(conn)
        .await?;
        Ok(())
    }
    pub async fn find_all<'a, E>(conn: E) -> Result<Vec<AccessToken>>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(AccessToken, "SELECT * FROM access_token")
            .fetch_all(conn)
            .await
    }
    pub async fn find_by_id<'a, E>(conn: E, id: i64) -> Result<Option<AccessToken>>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(AccessToken, "SELECT * FROM access_token WHERE id=$1", id)
            .fetch_optional(conn)
            .await
    }
}
