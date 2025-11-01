# nixbrew

A Homebrew-like CLI for Nix's imperative package management that combines the convenience of Homebrew with Nix's declarative reproducibility.

## Features

- **Install**: Install packages from nixpkgs with a simple command (supports version specification)
- **Uninstall**: Remove packages from your Nix profile
- **Search**: Search for packages in nixpkgs
- **List**: View all installed packages
- **Update**: Update the nixpkgs flake
- **Upgrade**: Upgrade specific packages to their latest versions
- **Versions**: List all available versions of a package across different channels
- **Pin**: Pin packages to specific versions with proper flake references
- **Create Flake**: Generate flake.nix files for packages
- **History**: View package installation history and available versions
- **Rollback**: Rollback packages to previous versions

## Installation

### Via Nix Flake (Recommended for Nix users)
```bash
nix profile install github:eyuael/nixbrew
```

### Via Cargo (crates.io)
```bash
cargo install nixbrew
```

### Via GitHub Releases (Pre-built binaries)
Download the latest release from [GitHub Releases](https://github.com/eyuael/nixbrew/releases) for your platform:
- **macOS**: `nixbrew-macos-x86_64` or `nixbrew-macos-aarch64`
- **Linux**: `nixbrew-linux-x86_64`
- **Windows**: `nixbrew-windows-x86_64.exe`

After downloading, make it executable and add to your PATH:
```bash
chmod +x nixbrew
mv nixbrew /usr/local/bin/  # or another directory in your PATH
```

### Build from Source
```bash
git clone https://github.com/eyuael/nixbrew.git
cd nixbrew
cargo build --release
sudo cp target/release/nixbrew /usr/local/bin/
```

## Usage

### Install a package
```bash
nixbrew install ripgrep
nixbrew install ripgrep 14.1.0        # Install specific version
nixbrew install ripgrep 23.11         # Install from specific channel
nixbrew install ripgrep cb82756       # Install from commit hash
```

### Uninstall a package
```bash
nixbrew uninstall ripgrep
```

### Search for packages
```bash
nixbrew search "text editor"
```

### List installed packages
```bash
nixbrew list
```

### Update nixpkgs
```bash
nixbrew update
```

### Upgrade a package
```bash
nixbrew upgrade ripgrep
```

### List available versions
```bash
nixbrew versions ripgrep
```

### Pin a package to a specific version
```bash
nixbrew pin ripgrep 14.0.3
```

### Change package version
You can change a package to a different version using any of these methods:
```bash
# Install a different version (will replace current version)
nixbrew install ripgrep 14.1.0

# Pin to a specific version
nixbrew pin ripgrep 14.0.3

# Rollback to a previously installed version
nixbrew rollback ripgrep 14.0.3
```

### Create a flake for a package
```bash
nixbrew create-flake ripgrep
nixbrew create-flake ripgrep --version 14.0.3
```

### View package history
```bash
nixbrew history ripgrep
```

### Rollback to a previous version
```bash
nixbrew rollback ripgrep 14.0.3
```

## How it works

nixbrew is a wrapper around Nix profiles that provides Homebrew-style commands. It uses `nix profile` commands under the hood with experimental features enabled automatically.

### Version Management

- **Version Specification**: Install packages with specific versions using semantic versions (e.g., "14.1.0"), channel names (e.g., "23.11"), or commit hashes
- **Semantic Version Resolution**: Automatically finds the correct nixpkgs channel for specific package versions
- **Version Listing**: View available versions across different nixpkgs channels
- **Version Changing**: Easily switch between package versions using `install`, `pin`, or `rollback` commands
- **Local Registry**: Tracks installed packages and their versions in `~/.nixbrew/registry.json`
- **Version Caching**: Caches version lookups to reduce network calls
- **Flake Generation**: Creates reproducible flake.nix files for packages
- **Rollback Support**: Maintains history of package installations for easy rollbacks

### Directory Structure

```
~/.nixbrew/
├── registry.json      # Package registry with history
└── flakes/            # Generated flake files
    ├── ripgrep.flake.nix
    └── ...
```

## Requirements

- **Nix** with flakes enabled (required for using nixbrew)
- **Rust toolchain** (only required if building from source or installing via cargo)