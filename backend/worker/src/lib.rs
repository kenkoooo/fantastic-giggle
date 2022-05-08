mod id_sync;
use std::time::{SystemTime, UNIX_EPOCH};

pub use id_sync::{FollowersDataConnector, FriendsDataConnector, IdSynchronizer};

mod follow_back;
pub use follow_back::FollowBackWorker;

pub(crate) struct Sortable<K, T> {
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

pub(crate) fn current_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
