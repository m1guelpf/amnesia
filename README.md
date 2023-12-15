# Amnesia

> An expressive Rust library for interacting with a Cache.

[![crates.io](https://img.shields.io/crates/v/amnesia.svg)](https://crates.io/crates/amnesia)
[![download count badge](https://img.shields.io/crates/d/amnesia.svg)](https://crates.io/crates/amnesia)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/amnesia)

## Features

- **Driver-Based Architecture**: Easily switch between different caching strategies by using drivers.
- **Asynchronous API**: Built with async/await for non-blocking I/O operations.
- **Serialization**: Leverage Serde for serializing and deserializing cache values.
- **Time-to-Live (TTL)**: Set expiration times for cache entries to ensure stale data is not served.
- **Extensible**: Implement your own cache drivers to extend functionality.

## Usage

```rust
let mut cache = Cache::<RedisDriver>::new(RedisConfig { // or DynamoDBDriver, DatabaseDriver, MemoryDriver, etc.
    redis_url: "..."
}).await?;

let my_value = cache.remember("test-value", Duration::from_secs(10), my_value).await?;

cache.forget("test-value").await?;
```

Please refer to the [documentation on docs.rs](https://docs.rs/amnesia) for detailed usage instructions.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
