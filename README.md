# nixbrew

A Homebrew-like CLI for Nix's imperative package management that combines the convenience of Homebrew with Nix's declarative reproducibility.

## Features

- **Install**: Install packages from nixpkgs with a simple command
- **Uninstall**: Remove packages from your Nix profile
- **Search**: Search for packages in nixpkgs
- **List**: View all installed packages
- **Update**: Update the nixpkgs flake
- **Upgrade**: Upgrade specific packages to their latest versions
- **Pin**: Pin packages to specific versions with proper flake references
- **Create Flake**: Generate flake.nix files for packages
- **History**: View package installation history and available versions
- **Rollback**: Rollback packages to previous versions

## Installation

```bash
cargo install --path .
```

## Usage

### Install a package
```bash
nixbrew install ripgrep
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

### Pin a package to a specific version
```bash
nixbrew pin ripgrep 14.0.3
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

- **Semantic Version Resolution**: Automatically finds the correct nixpkgs channel for specific package versions
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

- Nix with flakes enabled
- Rust toolchain for building