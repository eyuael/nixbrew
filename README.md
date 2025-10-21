# nixbrew

A Homebrew-like CLI for Nix's imperative package management that combines the convenience of Homebrew with Nix's declarative reproducibility.

## Features

- **Install**: Install packages from nixpkgs with a simple command
- **Uninstall**: Remove packages from your Nix profile
- **Search**: Search for packages in nixpkgs
- **List**: View all installed packages
- **Update**: Update the nixpkgs flake
- **Upgrade**: Upgrade specific packages to their latest versions

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

## How it works

nixbrew is a wrapper around Nix profiles that provides Homebrew-style commands. It uses `nix profile` commands under the hood with experimental features enabled automatically.

## Requirements

- Nix with flakes enabled
- Rust toolchain for building