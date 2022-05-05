use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use egg_mode::{
    auth::verify_tokens,
    cursor::{CursorIter, IDCursor},
    error::Error,
    user::{followers_ids, friends_ids},
    KeyPair, Token,
};
use fantastic_giggle_sql::{AccessToken, PgPool, Relationship};
use tokio::time::sleep;

pub struct IdSynchronizer<C> {
    consumer: KeyPair,
    pool: PgPool,
    connector: C,
}
impl<C> IdSynchronizer<C> {
    pub fn new(consumer: KeyPair, pool: PgPool, connector: C) -> Self {
        Self {
            consumer,
            pool,
            connector,
        }
    }
}

impl<C> IdSynchronizer<C>
where
    C: DataConnector,
{
    pub async fn run(&self) {
        loop {
            let tokens = match AccessToken::find_all(&self.pool).await {
                Ok(tokens) => tokens,
                Err(e) => {
                    log::error!("database error: {:?}", e);
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };
            if tokens.is_empty() {
                log::info!("No tokens");
                sleep(Duration::from_secs(10)).await;
                continue;
            }

            let mut heap = BinaryHeap::new();
            for token in tokens {
                let consumer = self.consumer.clone();
                let user_id = token.id;
                let access = KeyPair::new(token.access_key, token.access_secret);
                let token = Token::Access { consumer, access };
                heap.push(Sortable {
                    key: (Reverse(0)),
                    data: (user_id, token, -1),
                });
            }

            while let Some(Sortable { key, data }) = heap.pop() {
                let timestamp = key.0;
                let current_seconds = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if timestamp as u64 > current_seconds {
                    heap.push(Sortable { key, data });
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }

                let (user_id, token, next_cursor) = data;
                if let Err(e) = verify_tokens(&token).await {
                    log::error!("{:?}", e);
                    continue;
                }
                match self.connector.fetch_ids(user_id, &token, next_cursor).await {
                    Ok((ids, next_cursor)) => {
                        log::info!("successfully fetched {} ids", ids.len());
                        self.connector.save_ids(user_id, &ids).await;
                        if next_cursor != 0 {
                            heap.push(Sortable {
                                key: Reverse(timestamp),
                                data: (user_id, token, next_cursor),
                            });
                        }
                    }
                    Err(Error::RateLimit(timestamp)) => {
                        heap.push(Sortable {
                            key: Reverse(timestamp),
                            data: (user_id, token, next_cursor),
                        });
                    }
                    Err(e) => {
                        log::error!("twitter error: {:?}", e);
                    }
                }
            }
        }
    }
}

#[async_trait]
pub trait DataConnector {
    async fn fetch_ids(
        &self,
        user_id: i64,
        token: &Token,
        next_cursor: i64,
    ) -> Result<(Vec<i64>, i64), Error>;
    async fn save_ids(&self, user_id: i64, ids: &[i64]);
}

pub struct Followers {
    pub pool: PgPool,
}

#[async_trait]
impl DataConnector for Followers {
    async fn fetch_ids(
        &self,
        user_id: i64,
        token: &Token,
        next_cursor: i64,
    ) -> Result<(Vec<i64>, i64), Error> {
        fetch_ids(followers_ids, user_id, token, next_cursor).await
    }
    async fn save_ids(&self, user_id: i64, ids: &[i64]) {
        if let Err(e) = Relationship::save_followers(&self.pool, user_id, ids).await {
            log::error!("database error: {:?}", e);
            sleep(Duration::from_secs(5)).await;
        }
    }
}
pub struct Friends {
    pub pool: PgPool,
}

#[async_trait]
impl DataConnector for Friends {
    async fn fetch_ids(
        &self,
        user_id: i64,
        token: &Token,
        next_cursor: i64,
    ) -> Result<(Vec<i64>, i64), Error> {
        fetch_ids(friends_ids, user_id, token, next_cursor).await
    }
    async fn save_ids(&self, user_id: i64, ids: &[i64]) {
        if let Err(e) = Relationship::save_friends(&self.pool, user_id, ids).await {
            log::error!("database error: {:?}", e);
            sleep(Duration::from_secs(5)).await;
        }
    }
}

struct Sortable<K, T> {
    key: K,
    data: T,
}

impl<K: Ord, T> Ord for Sortable<K, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl<K: Eq, T> Eq for Sortable<K, T> {}
impl<K: PartialOrd, T> PartialOrd for Sortable<K, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K: PartialEq, T> PartialEq for Sortable<K, T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

async fn fetch_ids<F>(
    f: F,
    user_id: i64,
    token: &Token,
    next_cursor: i64,
) -> Result<(Vec<i64>, i64), Error>
where
    F: Fn(u64, &Token) -> CursorIter<IDCursor>,
{
    let result = {
        let mut cursor = f(user_id as u64, token);
        cursor.page_size = Some(5000);
        cursor.next_cursor = next_cursor;
        cursor.call()
    };

    match result.await {
        Ok(response) => {
            let next_cursor = response.next_cursor;
            let ids = response.ids.iter().map(|&id| id as i64).collect();
            Ok((ids, next_cursor))
        }
        Err(e) => Err(e),
    }
}
