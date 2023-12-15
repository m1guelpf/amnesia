use super::Driver;
use ensemble::{types::DateTime, Model};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

#[derive(Debug, Model)]
#[ensemble(table = "cache")]
struct CacheEntry {
	#[model(primary)]
	pub key: String,
	pub value: String,
	pub expiration: Option<DateTime>,
}

#[allow(clippy::module_name_repetitions)]
/// A driver that stores cache entries in a database.
pub struct DatabaseDriver;

impl Driver for DatabaseDriver {
	type Config = ();
	type Error = Error;

	async fn new((): Self::Config) -> Result<Self, Self::Error> {
		Ok(Self)
	}

	async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
		let Some(entry) = CacheEntry::query()
			.r#where("key", '=', key)
			.where_group(|query| {
				query
					.where_null("expiration")
					.or_where("expiration", '>', DateTime::now())
			})
			.first::<CacheEntry>()
			.await?
		else {
			return Ok(None);
		};

		Ok(Some(serde_json::from_str::<T>(&entry.value)?))
	}

	async fn has(&self, key: &str) -> Result<bool, Self::Error> {
		let count = CacheEntry::query()
			.r#where("key", '=', key)
			.where_null("expiration")
			.or_where("expiration", '>', DateTime::now())
			.count()
			.await?;

		Ok(count != 0)
	}

	async fn put<T: Serialize + Sync>(
		&mut self,
		key: &str,
		value: &T,
		duration: Option<Duration>,
	) -> Result<(), Self::Error> {
		let expiration = duration.map(|duration| DateTime::now() + duration);

		// TODO: This should be a single query.
		CacheEntry::query()
			.r#where("key", '=', key)
			.delete()
			.await?;

		CacheEntry::create(CacheEntry {
			expiration,
			key: key.to_string(),
			value: serde_json::to_string(value)?,
		})
		.await?;

		Ok(())
	}

	async fn forget(&mut self, key: &str) -> Result<(), Self::Error> {
		CacheEntry::query()
			.r#where("key", '=', key)
			.delete()
			.await?;

		Ok(())
	}

	async fn flush(&mut self) -> Result<(), Self::Error> {
		CacheEntry::query().delete().await?;

		Ok(())
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Database(#[from] ensemble::Error),
	#[error(transparent)]
	Serialize(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
	use std::env;

	use super::*;
	use crate::Cache;

	#[tokio::test]
	async fn test_database_driver() {
		ensemble::setup(&env::var("DATABASE_URL").expect("DATABASE_URL not set")).unwrap();

		let mut cache = Cache::<DatabaseDriver>::new(()).await.unwrap();

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

		cache
			.put("foo", &"bar".to_string(), Duration::from_secs(1))
			.await
			.unwrap();
	}
}
