use super::Driver;
use serde::{de::DeserializeOwned, Serialize};
use std::{convert::Infallible, time::Duration};

#[allow(clippy::module_name_repetitions)]
/// A driver that does nothing.
pub struct NullDriver;

impl Driver for NullDriver {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Self)
    }

    async fn get<T: DeserializeOwned>(&self, _key: &str) -> Result<Option<T>, Self::Error> {
        Ok(None)
    }

    async fn has(&self, _key: &str) -> Result<bool, Self::Error> {
        Ok(false)
    }

    async fn put<T: Serialize + Sync>(
        &mut self,
        _: &str,
        _: &T,
        _: Option<Duration>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn forget(&mut self, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cache;

    #[tokio::test]
    async fn test_null_driver() {
        let mut cache = Cache::<NullDriver>::new().await.unwrap();

        assert_eq!(cache.get::<String>("foo").await.unwrap(), None);
        assert!(!cache.has("foo").await.unwrap());

        cache
            .put("foo", &"bar", Duration::from_secs(1))
            .await
            .unwrap();

        assert_eq!(cache.get::<String>("foo").await.unwrap(), None);
        assert!(!cache.has("foo").await.unwrap());

        cache.forget("foo").await.unwrap();

        assert_eq!(cache.get::<String>("foo").await.unwrap(), None);
        assert!(!cache.has("foo").await.unwrap());
    }
}
