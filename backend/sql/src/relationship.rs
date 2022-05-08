use sqlx::{types::time::OffsetDateTime, Executor, Postgres, Result};

pub struct Relationship {
    pub source_id: i64,
    pub target_id: i64,
    pub updated_at: OffsetDateTime,
}

impl Relationship {
    pub async fn save_followers<'a, E: Executor<'a, Database = Postgres>>(
        conn: E,
        source_id: i64,
        target_ids: &[i64],
    ) -> Result<()> {
        sqlx::query(
            r#"
    INSERT INTO follower
    (
        source_id,
        target_id
    )
    SELECT $1, UNNEST($2)
    ON CONFLICT (source_id, target_id)
    DO UPDATE
    SET updated_at=CURRENT_TIMESTAMP    
    "#,
        )
        .bind(source_id)
        .bind(target_ids)
        .execute(conn)
        .await?;
        Ok(())
    }

    pub async fn find_followers_by_source_id<'a, E: Executor<'a, Database = Postgres>>(
        conn: E,
        source_id: i64,
    ) -> Result<Vec<Relationship>> {
        let relationships = sqlx::query_as!(
            Relationship,
            "SELECT * FROM follower WHERE source_id=$1",
            source_id
        )
        .fetch_all(conn)
        .await?;
        Ok(relationships)
    }

    pub async fn save_friends<'a, E: Executor<'a, Database = Postgres>>(
        conn: E,
        source_id: i64,
        target_ids: &[i64],
    ) -> Result<()> {
        sqlx::query(
            r#"
    INSERT INTO friend
    (
        source_id,
        target_id
    )
    SELECT $1, UNNEST($2)
    ON CONFLICT (source_id, target_id)
    DO UPDATE
    SET updated_at=CURRENT_TIMESTAMP    
    "#,
        )
        .bind(source_id)
        .bind(target_ids)
        .execute(conn)
        .await?;
        Ok(())
    }

    pub async fn find_friends_by_source_id<'a, E: Executor<'a, Database = Postgres>>(
        conn: E,
        source_id: i64,
    ) -> Result<Vec<Relationship>> {
        let relationships = sqlx::query_as!(
            Relationship,
            "SELECT * FROM friend WHERE source_id=$1",
            source_id
        )
        .fetch_all(conn)
        .await?;
        Ok(relationships)
    }
}
