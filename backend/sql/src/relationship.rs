use sqlx::{types::time::OffsetDateTime, Executor, Postgres, Result};

pub struct Relationship {
    pub source_id: i64,
    pub target_id: i64,
    pub following: bool,
    pub is_followed_by: bool,
    pub updated_at: OffsetDateTime,
}

impl Relationship {
    pub async fn save_friends<'a, E>(conn: E, source_id: i64, target_ids: &[i64]) -> Result<()>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query(
            r"
        INSERT INTO relationship
        (
            source_id,
            target_id,
            following
        )
        SELECT $1, UNNEST($2), TRUE
        ON CONFLICT (source_id, target_id)
        DO UPDATE
            SET following=TRUE, updated_at=CURRENT_TIMESTAMP
        ",
        )
        .bind(source_id)
        .bind(target_ids)
        .execute(conn)
        .await?;
        Ok(())
    }

    pub async fn save_followers<'a, E>(conn: E, source_id: i64, target_ids: &[i64]) -> Result<()>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query(
            r"
        INSERT INTO relationship
        (
            source_id,
            target_id,
            is_followed_by
        )
        SELECT $1, UNNEST($2), TRUE
        ON CONFLICT (source_id, target_id)
        DO UPDATE
            SET is_followed_by=TRUE, updated_at=CURRENT_TIMESTAMP
        ",
        )
        .bind(source_id)
        .bind(target_ids)
        .execute(conn)
        .await?;
        Ok(())
    }

    pub async fn find_by_source_id<'a, E>(conn: E, source_id: i64) -> Result<Vec<Relationship>>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let relationships = sqlx::query_as!(
            Relationship,
            "SELECT * FROM relationship WHERE source_id=$1",
            source_id
        )
        .fetch_all(conn)
        .await?;
        Ok(relationships)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_relationship() -> Result<()> {
        let pool = fantastic_giggle_test::connect_to_test_sql().await?;

        Relationship::save_followers(&pool, 1, &[2]).await?;
        let relationships = Relationship::find_by_source_id(&pool, 1).await?;
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].source_id, 1);
        assert_eq!(relationships[0].target_id, 2);
        assert!(!relationships[0].following);
        assert!(relationships[0].is_followed_by);

        Relationship::save_friends(&pool, 1, &[2]).await?;
        let relationships = Relationship::find_by_source_id(&pool, 1).await?;
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].source_id, 1);
        assert_eq!(relationships[0].target_id, 2);
        assert!(relationships[0].following);
        assert!(relationships[0].is_followed_by);

        Ok(())
    }
}
