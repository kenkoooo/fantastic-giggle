use std::{
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap},
    time::{Duration, Instant},
};

use anyhow::Result;
use egg_mode::{
    user::{follow, relation_lookup, Connection},
    KeyPair,
};
use fantastic_giggle_sql::{PgPool, Relationship, User};
use rand::{prelude::SliceRandom, thread_rng};
use tokio::time::sleep;

use crate::Sortable;

const RELATION_LOOKUP_LIMIT: usize = 100;
pub struct FollowBackWorker {
    pool: PgPool,
    consumer: KeyPair,
}

impl FollowBackWorker {
    pub fn new(pool: PgPool, consumer: KeyPair) -> Self {
        Self { pool, consumer }
    }
    pub async fn run(&self) {
        loop {
            log::info!("Start following back ...");
            let users = match User::find_all(&self.pool).await {
                Ok(users) => users,
                Err(e) => {
                    log::error!("database error: {:?}", e);
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };

            let mut heap = BinaryHeap::new();
            for user in users {
                let follow_back_user_ids = match fetch_follow_back_user_ids(
                    &user,
                    &self.pool,
                    self.consumer.clone(),
                )
                .await
                {
                    Ok(user_ids) => user_ids,
                    Err(e) => {
                        log::error!("{:?}", e);
                        continue;
                    }
                };
                let consumer = self.consumer.clone();
                let access = KeyPair::new(user.access_key, user.access_secret);
                let token = egg_mode::Token::Access { consumer, access };
                heap.push(Sortable {
                    key: Reverse(Instant::now()),
                    data: ((token, follow_back_user_ids)),
                });
            }

            while let Some(Sortable { key, data }) = heap.pop() {
                if key.0 > Instant::now() {
                    sleep(Duration::from_secs(1)).await;
                    heap.push(Sortable { key, data });
                    continue;
                }

                let (token, mut user_ids) = data;
                let id = match user_ids.pop() {
                    Some(id) => id,
                    None => continue,
                };

                match follow(id, false, &token).await {
                    Ok(_) => {
                        log::info!("followed {}", id);
                        heap.push(Sortable {
                            key: Reverse(Instant::now() + Duration::from_secs(60)),
                            data: (token, user_ids),
                        });
                    }
                    Err(e) => {
                        log::error!("failed to follow: {:?}", e);
                    }
                }
            }

            log::info!("finished following back. sleeping 5 minutes");
            sleep(Duration::from_secs(5 * 60)).await;
        }
    }
}
async fn fetch_follow_back_user_ids(
    user: &User,
    pool: &PgPool,
    consumer: KeyPair,
) -> Result<Vec<u64>> {
    let followers = Relationship::find_followers_by_source_id(pool, user.id).await?;
    let friends = Relationship::find_friends_by_source_id(pool, user.id).await?;

    let mut follower_ids = followers
        .into_iter()
        .map(|r| r.target_id)
        .collect::<BTreeSet<_>>();
    for friend in friends {
        follower_ids.remove(&friend.target_id);
    }

    let mut following_ids = follower_ids
        .into_iter()
        .map(|id| id as u64)
        .collect::<Vec<_>>();
    following_ids.shuffle(&mut thread_rng());
    following_ids.truncate(RELATION_LOOKUP_LIMIT);

    let access = KeyPair::new(user.access_key.clone(), user.access_secret.clone());
    let token = egg_mode::Token::Access { consumer, access };
    let relationships = relation_lookup(following_ids, &token).await?;

    let mut following_user_ids = vec![];
    for relationship in relationships {
        let mut is_followed_by = false;
        let mut following = false;
        for connection in &relationship.connections {
            match connection {
                Connection::FollowingReceived | Connection::FollowedBy => {
                    is_followed_by = true;
                }
                Connection::Following | Connection::FollowingRequested => {
                    following = true;
                }
                _ => {}
            }
        }

        if is_followed_by && !following {
            following_user_ids.push(relationship.id);
        }
    }
    Ok(following_user_ids)
}
