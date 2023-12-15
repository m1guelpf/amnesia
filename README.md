# An expressive interface for interacting with a Cache.

## Features

- **Extensible**: Implement your own cache drivers to extend functionality.
- **Asynchronous API**: Built with async/await for non-blocking I/O operations.
- **Serialization**: Leverage Serde for serializing and deserializing cache values.
- **Time-to-Live (TTL)**: Set expiration times for cache entries to ensure stale data is not served.
- **Driver-Based Architecture**: Easily switch between different caching strategies by using drivers.

## Usage

Please refer to the [documentation on docs.rs](https://docs.rs/amnesia) for detailed usage instructions.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
