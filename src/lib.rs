#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
//! An expressive interface for interacting with a Cache.
//! Inspired by [Laravel's Cache](https://laravel.com/docs/cache) facade.

use drivers::Driver;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

pub mod drivers;

pub struct Cache<D: Driver> {
    driver: D,
}

impl<D: Driver> Cache<D> {
    /// Create a new instance of the cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to initialize.
    pub async fn new() -> Result<Self, D::Error> {
        Ok(Self {
            driver: D::new().await?,
        })
    }

    /// Retrieve an item from the cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to retrieve the item.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, D::Error> {
        self.driver.get(key).await
    }

    /// Check if an item exists in the cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to check if the item exists.
    pub async fn has(&self, key: &str) -> Result<bool, D::Error> {
        self.driver.has(key).await
    }

    /// Retrieve an item from the cache, or store it for some time if it doesn't exist yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to retrieve or store the item.
    pub async fn remember<T: Serialize + DeserializeOwned + Send + Sync>(
        &mut self,
        key: &str,
        duration: Duration,
        value: T,
    ) -> Result<T, D::Error> {
        let value = if let Some(value) = self.driver.get::<T>(key).await? {
            value
        } else {
            self.put(key, &value, duration).await?;

            value
        };

        Ok(value)
    }

    /// Retrieve an item from the cache, or store it forever if it doesn't exist yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to retrieve or store the item.
    pub async fn remember_forever<T: Serialize + DeserializeOwned + Send + Sync>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<T, D::Error> {
        let value = if let Some(value) = self.driver.get::<T>(key).await? {
            value
        } else {
            self.forever(key, &value).await?;

            value
        };

        Ok(value)
    }

    /// Remove an item from the cache and return it.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to retrieve or remove the item.
    pub async fn pull<T: DeserializeOwned + Send>(
        &mut self,
        key: &str,
    ) -> Result<Option<T>, D::Error> {
        let Some(item) = self.get(key).await? else {
            return Ok(None);
        };

        self.forget(key).await?;

        Ok(Some(item))
    }

    /// Store an item in the cache for a given duration.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to store the item.
    pub async fn put<T: Serialize + Sync>(
        &mut self,
        key: &str,
        value: &T,
        expiry: Duration,
    ) -> Result<(), D::Error> {
        self.driver.put(key, value, Some(expiry)).await
    }

    /// Store an item in the cache if it doesn't exist yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to store the item.
    pub async fn add<T: Serialize + Send + Sync>(
        &mut self,
        key: &str,
        value: T,
        expiry: Duration,
    ) -> Result<bool, D::Error> {
        if self.has(key).await? {
            return Ok(false);
        }

        self.put(key, &value, expiry).await?;

        Ok(true)
    }

    /// Store an item in the cache indefinitely.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to store the item.
    pub async fn forever<T: Serialize + Send + Sync>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<(), D::Error> {
        self.driver.put(key, &value, None).await
    }

    /// Remove an item from the cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to remove the item.
    pub async fn forget(&mut self, key: &str) -> Result<(), D::Error> {
        self.driver.forget(key).await
    }

    /// Remove all items from the cache.
    ///
    /// # Errors
    ///
    /// Returns an error if the driver fails to flush the cache.
    pub async fn flush(&mut self) -> Result<(), D::Error> {
        self.driver.flush().await
    }
}
