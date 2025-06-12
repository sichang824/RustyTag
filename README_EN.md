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
rustytag patch                    # Bump patch version (e.g., 1.0.0 -> 1.0.1)
rustytag patch -V 1.2.3          # Set to specific version 1.2.3
rustytag minor                    # Bump minor version (e.g., 1.0.0 -> 1.1.0)
rustytag minor --version 2.0.0   # Set to specific version 2.0.0
rustytag major                    # Bump major version (e.g., 1.0.0 -> 2.0.0)
rustytag major -V 3.0.0          # Set to specific version 3.0.0

# Tag synchronization commands
rustytag sync   # Sync local tags with remote
rustytag reset  # Reset local tags to match remote

# Information commands
rustytag show   # Show current version information

# Release management
rustytag release                  # Create a release
rustytag release -l               # List all releases
rustytag release --list           # List all releases
rustytag release -t v1.0.0        # Create a release for specific version
rustytag release --tag v1.0.0     # Create a release for specific version

# Configuration
rustytag config                   # Show current configuration
rustytag config --set KEY=VALUE  # Set configuration
```

### Command Details

#### Version Management Commands

- `init`: Initialize a new Git repository with semantic versioning
- `patch/minor/major`: Bump version according to semver specification
  - Without parameters: Automatically increment version
  - With `-V` or `--version` parameter: Set to specific version

#### Tag Synchronization Commands

- `sync`: Synchronize local tags with remote repository
- `reset`: Reset local tags to match remote repository

#### Information Commands

- `show`: Display detailed project and tool information

#### Release Management Commands

- `release`: Manage GitHub releases
  - Without parameters: Create release for current version
  - `-l` or `--list`: List all releases
  - `-t` or `--tag`: Create release for specific version

#### Configuration Commands

- `config`: Configure RustyTag settings
  - Without parameters: Show current configuration
  - `--set KEY=VALUE`: Set configuration value
  - `--global`: Set global configuration
  - `--local`: Set local configuration

## Usage Examples

### Basic Workflow

```sh
# 1. Initialize project
rustytag init

# 2. After development, bump version
rustytag patch              # Bug fixes, bump patch version
rustytag minor              # New features, bump minor version  
rustytag major              # Breaking changes, bump major version

# 3. Push to remote repository
git push --follow-tags origin main

# 4. Create GitHub release
rustytag release
```

### Advanced Usage

```sh
# Set specific version directly
rustytag patch -V 1.2.3
rustytag minor --version 2.0.0

# View project information
rustytag show

# Sync remote tags
rustytag sync

# Configure GitHub Token (for release management)
rustytag config --set GITHUB_TOKEN=your_token_here
```

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Git's native tag functionality
- Built with Rust for performance and safety
