# RustyTag

A semantic version management tool built on Git tags.

## Features

- Semantic version management using Git tags
- Semantic versioning support
- Automatic version bumping (patch/minor/major)
- Local tag synchronization with remote
- Tag comparison with previous version
- Lightweight and fast
- Cross-platform support

## Installation

### Using Cargo

```sh
cargo install rustytag
```

### From Source

1. Clone the repository:

```sh
git clone https://github.com/yourusername/rustytag.git
```

2. Build the project:

```sh
cd rustytag
cargo build --release
```

## Usage

### Basic Commands

```sh
# Initialize semantic versioning

rustytag init

# Bump patch version (e.g., 1.0.0 -> 1.0.1)

rustytag patch

# Bump minor version (e.g., 1.0.0 -> 1.1.0)

rustytag minor

# Bump major version (e.g., 1.0.0 -> 2.0.0)

rustytag major

# Reset local tags to match remote

rustytag reset
```

### Advanced Usage

```sh
# Show project information
rustytag show

# List all release add release link
rustytag list

# Show tag details
rustytag show v1.0.0

# Create a release with changelog
rustytag release

# Set config
rustytag config --set XXXX=xxx
```

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Git's native tag functionality
- Built with Rust for performance and safety
