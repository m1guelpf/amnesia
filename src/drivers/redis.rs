use super::Driver;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

pub struct Config {
	pub prefix: String,
	pub redis_url: String,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			prefix: String::new(),
			redis_url: "redis://localhost".to_string(),
		}
	}
}

#[allow(clippy::module_name_repetitions)]
/// A driver that uses Redis.
pub struct RedisDriver {
	prefix: String,
	client: redis::Client,
}

impl Driver for RedisDriver {
	type Error = Error;
	type Config = Config;

	async fn new(config: Self::Config) -> Result<Self, Self::Error> {
		Ok(Self {
			prefix: config.prefix,
			client: redis::Client::open(config.redis_url)?,
		})
	}

	async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
		let mut conn = self.client.get_async_connection().await?;

		let Some(data) = conn
			.get::<_, Option<Vec<u8>>>(format!("{}{key}", self.prefix))
			.await?
		else {
			return Ok(None);
		};

		Ok(Some(bitcode::deserialize(&data)?))
	}

	async fn has(&self, key: &str) -> Result<bool, Self::Error> {
		let mut conn = self.client.get_async_connection().await?;

		Ok(conn.exists(format!("{}{key}", self.prefix)).await?)
	}

	async fn put<T: Serialize + Sync>(
		&mut self,
		key: &str,
		value: &T,
		expiry: Option<Duration>,
	) -> Result<(), Self::Error> {
		let mut conn = self.client.get_async_connection().await?;
		let data = bitcode::serialize(value)?;

		if let Some(expiry) = expiry {
			conn.set_ex(format!("{}{key}", self.prefix), data, expiry.as_secs())
				.await?;
		} else {
			conn.set(format!("{}{key}", self.prefix), data).await?;
		}

		Ok(())
	}

	async fn forget(&mut self, key: &str) -> Result<(), Self::Error> {
		let mut conn = self.client.get_async_connection().await?;
		conn.del(format!("{}{key}", self.prefix)).await?;

		Ok(())
	}

	async fn flush(&mut self) -> Result<(), Self::Error> {
		let mut conn = self.client.get_async_connection().await?;
		redis::cmd("FLUSHDB").query_async(&mut conn).await?;

		Ok(())
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Redis(#[from] redis::RedisError),
	#[error(transparent)]
	Serialization(#[from] bitcode::Error),
}

#[cfg(test)]
mod tests {
	use std::env;

	use super::*;
	use crate::Cache;

	#[tokio::test]
	async fn test_redis_driver() {
		let mut cache = Cache::<RedisDriver>::new(Config {
			redis_url: env::var("REDIS_URL").expect("REDIS_URL not set"),
			..Default::default()
		})
		.await
		.unwrap();

		assert_eq!(cache.get::<String>("foo").await.unwrap(), None);
		assert!(!cache.has("foo").await.unwrap());

		cache
			.put("foo", &"bar", Duration::from_secs(1))
			.await
			.unwrap();

		assert_eq!(
			cache.get::<String>("foo").await.unwrap(),
			Some("bar".to_string())
		);
		assert!(cache.has("foo").await.unwrap());

		cache.forget("foo").await.unwrap();

		assert_eq!(cache.get::<String>("foo").await.unwrap(), None);
		assert!(!cache.has("foo").await.unwrap());
	}
}
