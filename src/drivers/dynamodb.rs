use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aws_sdk_dynamodb::{primitives::Blob, types::AttributeValue};
use serde::{de::DeserializeOwned, Serialize};

use super::Driver;

#[derive(Debug, Clone)]
pub struct Config {
	pub table: String,
	pub prefix: String,
	pub key_attribute: String,
	pub value_attribute: String,
	pub expiration_attribute: String,
	pub aws_config: aws_types::SdkConfig,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			prefix: String::new(),
			table: "cache".to_string(),
			key_attribute: String::from("key"),
			value_attribute: String::from("value"),
			expiration_attribute: String::from("expires_at"),
			aws_config: aws_types::SdkConfig::builder().build(),
		}
	}
}

#[allow(clippy::module_name_repetitions)]
/// A driver that uses DynamoDB as a backend.
pub struct DynamoDBDriver {
	table: String,
	prefix: String,
	key_attribute: String,
	value_attribute: String,
	expiration_attribute: String,
	client: aws_sdk_dynamodb::Client,
}

impl DynamoDBDriver {
	async fn get_item(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
		let response = self
			.client
			.get_item()
			.table_name(&self.table)
			.key(
				self.key_attribute.clone(),
				AttributeValue::S(format!("{}{key}", self.prefix)),
			)
			.send()
			.await?;

		let Some(item) = response.item else {
			return Ok(None);
		};

		if let Some(expires_at) = item
			.get(&self.expiration_attribute)
			.map(|value| value.as_n())
		{
			let expires_at: u64 = expires_at
				.map_err(|_| Error::InvalidDataFormat)?
				.parse()
				.map_err(|_| Error::InvalidDataFormat)?;

			if UNIX_EPOCH + Duration::from_secs(expires_at) < SystemTime::now() {
				return Ok(None);
			}
		}

		let data = if let Some(data) = item.get(&self.value_attribute).map(|value| value.as_b()) {
			data.map_err(|_| Error::InvalidDataFormat)?
		} else {
			return Err(Error::InvalidDataFormat);
		};

		Ok(Some(data.as_ref().to_vec()))
	}
}

impl Driver for DynamoDBDriver {
	type Error = Error;
	type Config = Config;

	async fn new(config: Self::Config) -> Result<Self, Self::Error> {
		Ok(Self {
			table: config.table,
			prefix: config.prefix,
			key_attribute: config.key_attribute,
			value_attribute: config.value_attribute,
			expiration_attribute: config.expiration_attribute,
			client: aws_sdk_dynamodb::Client::new(&config.aws_config),
		})
	}

	async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error> {
		let item = self.get_item(key).await?;

		Ok(item
			.map(|item| bitcode::deserialize(item.as_ref()))
			.transpose()?)
	}

	async fn has(&self, key: &str) -> Result<bool, Self::Error> {
		let item = self.get_item(key).await?;

		Ok(item.is_some())
	}

	async fn put<T: Serialize + Sync>(
		&mut self,
		key: &str,
		value: &T,
		expiry: Option<Duration>,
	) -> Result<(), Self::Error> {
		let expires_at = expiry.map(|expiry| SystemTime::now() + expiry);

		self.client
			.put_item()
			.table_name(&self.table)
			.item(
				self.key_attribute.clone(),
				AttributeValue::S(format!("{}{key}", self.prefix)),
			)
			.item(
				self.value_attribute.clone(),
				AttributeValue::B(Blob::new(bitcode::serialize(value)?)),
			)
			.item(
				self.expiration_attribute.clone(),
				expires_at.map_or(AttributeValue::Null(true), |expires_at| {
					AttributeValue::N(
						expires_at
							.duration_since(SystemTime::UNIX_EPOCH)
							.unwrap()
							.as_secs()
							.to_string(),
					)
				}),
			)
			.send()
			.await?;

		Ok(())
	}

	async fn forget(&mut self, key: &str) -> Result<(), Self::Error> {
		self.client
			.delete_item()
			.table_name(&self.table)
			.key(&self.key_attribute, AttributeValue::S(key.to_string()))
			.send()
			.await?;

		Ok(())
	}

	async fn flush(&mut self) -> Result<(), Self::Error> {
		Err(Error::FlushNotSupported)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("DynamoDB does not support flushing the cache.")]
	FlushNotSupported,
	#[error("the stored data was on an unexpected format.")]
	InvalidDataFormat,
	#[error(transparent)]
	GetItem(
		#[from]
		aws_smithy_runtime_api::client::result::SdkError<
			aws_sdk_dynamodb::operation::get_item::GetItemError,
			aws_smithy_runtime_api::client::orchestrator::HttpResponse,
		>,
	),
	#[error(transparent)]
	PutItem(
		#[from]
		aws_smithy_runtime_api::client::result::SdkError<
			aws_sdk_dynamodb::operation::put_item::PutItemError,
			aws_smithy_runtime_api::client::orchestrator::HttpResponse,
		>,
	),
	#[error(transparent)]
	DeleteItem(
		#[from]
		aws_smithy_runtime_api::client::result::SdkError<
			aws_sdk_dynamodb::operation::delete_item::DeleteItemError,
			aws_smithy_runtime_api::client::orchestrator::HttpResponse,
		>,
	),
	#[error(transparent)]
	Serialization(#[from] bitcode::Error),
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Cache;

	#[tokio::test]
	async fn test_dynamodb_driver() {
		let mut cache = Cache::<DynamoDBDriver>::new(Config::default())
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
