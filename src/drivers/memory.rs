use super::Driver;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

#[allow(clippy::module_name_repetitions)]
/// A driver that stores values in memory.
pub struct MemoryDriver {
    cache: HashMap<String, (Vec<u8>, Option<SystemTime>)>,
}

impl Driver for MemoryDriver {
    type Error = Error;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Self {
            cache: HashMap::new(),
        })
    }

    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
        let Some((data, expires_at)) = self.cache.get(key) else {
            return Ok(None);
        };

        if let Some(expires_at) = expires_at {
            if expires_at < &SystemTime::now() {
                // We would ideally clean up expired values here, but that would require a mutable reference to self,
                // which provides a worse developer experience than just letting the cache grow.
                return Ok(None);
            }
        }

        Ok(Some(bitcode::deserialize(data)?))
    }

    async fn has(&self, key: &str) -> Result<bool, Self::Error> {
        Ok(self.cache.contains_key(key))
    }

    async fn put<T: Serialize + Sync>(
        &mut self,
        key: &str,
        value: &T,
        duration: Option<Duration>,
    ) -> Result<(), Self::Error> {
        let data = bitcode::serialize(value)?;
        let expires_at = duration.map(|duration| SystemTime::now() + duration);

        self.cache.insert(key.to_owned(), (data, expires_at));

        Ok(())
    }

    async fn forget(&mut self, key: &str) -> Result<(), Self::Error> {
        self.cache.remove(key);

        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.cache.clear();

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to deserialize data")]
    DeserializationError(#[from] bitcode::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cache;

    #[tokio::test]
    async fn test_memory_driver() {
        let mut cache = Cache::<MemoryDriver>::new().await.unwrap();

        assert_eq!(cache.get::<String>("foo").await.unwrap(), None);
        assert!(!cache.has("foo").await.unwrap());

        cache
            .put("foo", &"bar".to_string(), Duration::from_secs(10))
            .await
            .unwrap();

        assert_eq!(cache.get("foo").await.unwrap(), Some("bar".to_string()));
        assert!(cache.has("foo").await.unwrap());

        cache.forget("foo").await.unwrap();

        assert_eq!(cache.get::<String>("foo").await.unwrap(), None);
        assert!(!cache.has("foo").await.unwrap());
    }
}
