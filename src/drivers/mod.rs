use serde::{de::DeserializeOwned, Serialize};
use std::{future::Future, time::Duration};

#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "memory")]
pub mod memory;
pub mod null;

#[cfg(feature = "database")]
pub use database::DatabaseDriver;
#[cfg(feature = "memory")]
pub use memory::MemoryDriver;
pub use null::NullDriver;

/// Cache driver.
pub trait Driver: Sized + Send + Sync {
	type Error: Send;

	fn new() -> impl Future<Output = Result<Self, Self::Error>> + Send;

	/// Get a value from the cache.
	fn get<T: DeserializeOwned>(
		&self,
		key: &str,
	) -> impl Future<Output = Result<Option<T>, Self::Error>> + Send;

	/// Check if a value exists in the cache.
	fn has(&self, key: &str) -> impl Future<Output = Result<bool, Self::Error>> + Send;

	/// Put a value into the cache.
	fn put<T: Serialize + Sync>(
		&mut self,
		key: &str,
		data: &T,
		expiry: Option<Duration>,
	) -> impl Future<Output = Result<(), Self::Error>> + Send;

	/// Remove a value from the cache.
	fn forget(&mut self, key: &str) -> impl Future<Output = Result<(), Self::Error>> + Send;

	/// Remove all values from the cache.
	fn flush(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
