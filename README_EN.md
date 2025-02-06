<div align="center">

# RustyTag

### A Semantic Version Management Tool Built on Git Tags

[简体中文](README.md) | English

</div>

## Features

- Semantic version management using Git tags
- Complete semantic versioning support
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

# Version bumping commands
rustytag patch  # Bump patch version (e.g., 1.0.0 -> 1.0.1)
rustytag minor  # Bump minor version (e.g., 1.0.0 -> 1.1.0)
rustytag major  # Bump major version (e.g., 1.0.0 -> 2.0.0)

# Tag synchronization commands
rustytag sync   # Sync local tags with remote
rustytag reset  # Reset local tags to match remote

# Information commands
rustytag show   # Show current version information
rustytag show v1.0.0  # Show specific tag details

# Release management
rustytag release  # Create a release
rustytag release -l  # List all releases
rustytag release --list  # List all releases
rustytag release -t v1.0.0  # Create a release for specific version
rustytag release --tag v1.0.0  # Create a release for specific version

# Configuration
rustytag config  # Show current configuration
rustytag config --set KEY=VALUE  # Set configuration
```

### Command Details

- `init`: Initialize a new Git repository with semantic versioning
- `patch/minor/major`: Bump version according to semver specification
- `sync`: Synchronize local tags with remote repository
- `reset`: Reset local tags to match remote repository
- `show`: Display version information
- `release`: Manage releases
- `config`: Configure RustyTag settings, shows current configuration when used without parameters

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Git's native tag functionality
- Built with Rust for performance and safety
