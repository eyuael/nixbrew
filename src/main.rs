use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PackageInfo {
    name: String,
    version: String,
    flake_url: String,
    install_date: String,
    flake_lock: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageRegistry {
    packages: HashMap<String, Vec<PackageInfo>>,
    version_cache: HashMap<String, HashMap<String, String>>, // package -> version -> flake_url
}

impl PackageRegistry {
    fn new() -> Self {
        Self {
            packages: HashMap::new(),
            version_cache: HashMap::new(),
        }
    }

    fn load() -> Result<Self> {
        let path = get_registry_path()?;
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::new())
        }
    }

    fn save(&self) -> Result<()> {
        let path = get_registry_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn add_package(&mut self, package: PackageInfo) {
        self.packages.entry(package.name.clone()).or_insert_with(Vec::new).push(package);
    }

    fn get_package_history(&self, package: &str) -> Option<&Vec<PackageInfo>> {
        self.packages.get(package)
    }

    fn cache_version(&mut self, package: &str, version: &str, flake_url: &str) {
        self.version_cache
            .entry(package.to_string())
            .or_insert_with(HashMap::new)
            .insert(version.to_string(), flake_url.to_string());
    }

    fn get_cached_version(&self, package: &str, version: &str) -> Option<&String> {
        self.version_cache
            .get(package)
            .and_then(|versions| versions.get(version))
    }
}

fn get_registry_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home_dir.join(".nixbrew").join("registry.json"))
}

async fn create_package_flake(package: &str, version: Option<&str>) -> Result<()> {
    let _flake_url = build_flake_url(package, version).await?;
    let flake_content = format!(
        r#"{{
  description = "Nix flake for {0} package";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  }};

  outputs = {{ self, nixpkgs }}:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${{system}};
    in {{
      packages.${{system}}.default = pkgs.{0};
      defaultPackage.${{system}} = pkgs.{0};
    }};
}}
"#,
        package
    );

    let flake_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?
        .join(".nixbrew")
        .join("flakes")
        .join(package);

    fs::create_dir_all(&flake_dir)?;
    
    let flake_path = flake_dir.join("flake.nix");
    fs::write(&flake_path, flake_content)?;
    println!("Created flake at: {}", flake_path.display());
    
    // Generate flake.lock
    run_nix_command(vec!["flake", "update", "--flake", &flake_dir.to_string_lossy()]).await?;
    
    Ok(())
}

async fn show_package_history(package: &str) -> Result<()> {
    let registry = PackageRegistry::load()?;
    
    if let Some(history) = registry.get_package_history(package) {
        println!("History for {}:", package);
        for (i, pkg_info) in history.iter().enumerate() {
            println!("  {}. Version: {} ({})", i + 1, pkg_info.version, pkg_info.install_date);
            println!("     Flake URL: {}", pkg_info.flake_url);
        }
    } else {
        println!("No history found for package: {}", package);
        
        // Try to get available versions from nixpkgs
        println!("Searching for available versions...");
        let channels = ["nixpkgs/nixos-unstable", "nixpkgs/nixos-23.11", "nixpkgs/nixos-23.05"];
        for channel in &channels {
            let output = Command::new("nix")
                .args([
                    "--extra-experimental-features", "nix-command",
                    "--extra-experimental-features", "flakes",
                    "eval",
                    &format!("{}#{}.version", channel, package),
                    "--json",
                ])
                .output()
                .await?;
                
            if output.status.success() {
                let version_str = String::from_utf8(output.stdout)?;
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&version_str) {
                    if let Some(version) = parsed.as_str() {
                        println!("  {}: {}", channel, version);
                    }
                }
            }
        }
    }
    
    Ok(())
}

async fn rollback_package(package: &str, version: &str) -> Result<()> {
    println!("Rolling back {} to version {}...", package, version);
    
    // First uninstall current version
    let list_output = Command::new("nix")
        .args(["profile", "list"])
        .output()
        .await?;

    if list_output.status.success() {
        let list_str = String::from_utf8(list_output.stdout)?;
        for line in list_str.lines() {
            if line.contains(&format!("nixpkgs#{}", package)) {
                if let Some(index) = line.split_whitespace().next() {
                    run_nix_command(vec!["profile", "remove", index]).await?;
                    break;
                }
            }
        }
    }

    // Install the specific version
    let flake_url = build_flake_url(package, Some(version)).await?;
    run_nix_command(vec!["profile", "add", &flake_url]).await?;
    
    // Update registry
    let mut registry = PackageRegistry::load()?;
    let package_info = PackageInfo {
        name: package.to_string(),
        version: version.to_string(),
        flake_url,
        install_date: chrono::Utc::now().to_rfc3339(),
        flake_lock: None,
    };
    registry.add_package(package_info);
    registry.save()?;
    
    println!("Successfully rolled back {} to version {}", package, version);
    Ok(())
}

// Define the structure of our command-line interface using Clap's derive macros.
#[derive(Parser)]
#[command(name = "nixbrew")]
#[command(about = "A Homebrew-like CLI for Nix's imperative package management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Define the subcommands: install, uninstall, etc.
#[derive(Subcommand)]
enum Commands {
    /// Install a package from nixpkgs
    Install {
        /// The name of the package to install (e.g., ripgrep)
        package: String,
        /// Optional version or channel to install (e.g., "1.2.3" or "nixpkgs-23.05"), or commit hash
        version: Option<String>,
    },
    /// Uninstall a package
    Uninstall {
        /// The name of the package to uninstall
        package: String,
    },
    /// Search for a package in nixpkgs
    Search {
        /// The search query
        query: String,
    },
    /// List installed packages
    List,
    /// Update the nixpkgs flake (like 'brew update')
    Update,
    /// Upgrade a specific package
    Upgrade {
        /// The name of the package to upgrade
        package: String,
    },
    ///List all available versions of a package
    Versions {
        /// The name of the package to list versions for
        package: String,
    },
    /// Pin a package to a specific version
    Pin {
        /// The name of the package to pin
        package: String,
        /// The version to pin the package to (e.g., "1.2.3")
        version: String,
    },
    /// Create a flake.nix for a package
    CreateFlake {
        /// The name of the package
        package: String,
        /// Optional version specification
        version: Option<String>,
    },
    /// Show package history and available versions
    History {
        /// The name of the package
        package: String,
    },
    /// Rollback to a previous version
    Rollback {
        /// The name of the package
        package: String,
        /// The version to rollback to
        version: String,
    },
}

// Helper function to run a `nix` command and pipe its output to the console.
async fn run_nix_command(args: Vec<&str>) -> Result<()> {
    let mut full_args = vec![
        "--extra-experimental-features", "nix-command",
        "--extra-experimental-features", "flakes"
    ];
    full_args.extend(args);
    
    let status = Command::new("nix")
        .args(full_args)
        .stdout(Stdio::inherit()) // Forward nix's stdout to our stdout
        .stderr(Stdio::inherit()) // Forward nix's stderr to our stderr
        .status()
        .await?;

    if !status.success() {
        // If nix failed, return an error with its exit code
        return Err(anyhow!(
            "Nix command failed with exit code {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

// The main logic for each command
async fn handle_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Install { package, version } => {
          println!("Installing {}{}...", package, version.as_ref().map(|v| format!(" version {}", v)).unwrap_or_default());
          let flake_url = build_flake_url(&package, version.as_deref()).await?;
          run_nix_command(vec!["profile", "add", &flake_url]).await
        }
        Commands::Uninstall { package } => {
            // Find the package's index in the profile.
            println!("Finding package '{}' to uninstall...", package);
            let list_output = Command::new("nix")
                .args(["profile", "list"])
                .output()
                .await?;

            if !list_output.status.success() {
                return Err(anyhow!("Failed to run 'nix profile list'"));
            }

            let list_str = String::from_utf8(list_output.stdout)?;
            let mut pkg_index: Option<String> = None;

            for line in list_str.lines() {
                // The output looks like: "3  nixpkgs#cowsay-3.04"
                if line.contains(&format!("nixpkgs#{}", package)) {
                    pkg_index = line.split_whitespace().next().map(String::from);
                    break;
                }
            }

            match pkg_index {
                Some(index) => {
                    println!("Uninstalling {} (index: {})...", package, index);
                    run_nix_command(vec!["profile", "remove", &index]).await
                }
                None => Err(anyhow!("Package '{}' not found in profile.", package)),
            }
        }
        Commands::Search { query } => {
            run_nix_command(vec!["search", "nixpkgs", &query]).await
        }
        Commands::List => {
            run_nix_command(vec!["profile", "list"]).await
        }
        Commands::Update => {
            println!("Updating nixpkgs flake...");
            run_nix_command(vec!["flake", "update", "nixpkgs"]).await
        }
        Commands::Upgrade { package } => {
            println!("Upgrading {}...", package);
            run_nix_command(vec![
                "profile",
                "add",
                &format!("nixpkgs#{}", package),
                "--reinstall",
            ])
            .await
        }
        Commands::Versions { package } => {
            println!("Listing versions of {}...", package);
            let channels = ["nixpkgs", "nixpkgs/23.11", "nixpkgs/23.05"];
            for channel in &channels {
              println!("\nChecking {}:", channel);
              run_nix_command(vec![
                "flake",
                "show",
                &format!("{}#{}", channel, package),
              ])
              .await?;
            }Ok(())
            
        }
        Commands::Pin { package, version } => {
            println!("Pinning {} to version {}...", package, version);
            let flake_url = build_flake_url(&package, Some(&version)).await?;
            run_nix_command(vec![
                "profile",
                "add",
                &flake_url,
            ])
            .await?;
            
            // Add to registry
            let mut registry = PackageRegistry::load()?;
            let package_info = PackageInfo {
                name: package.clone(),
                version: version.clone(),
                flake_url,
                install_date: chrono::Utc::now().to_rfc3339(),
                flake_lock: None,
            };
            registry.add_package(package_info);
            registry.save()?;
            Ok(())
        }
        Commands::CreateFlake { package, version } => {
            create_package_flake(&package, version.as_deref()).await
        }
        Commands::History { package } => {
            show_package_history(&package).await
        }
        Commands::Rollback { package, version } => {
            rollback_package(&package, &version).await
        }
    }
}
async fn build_flake_url(package: &str, version: Option<&str>) -> Result<String> {
    match version {
        Some(v) => {
            // Handle different version formats
            if v.matches('.').count() == 1 && v.chars().all(|c| c.is_ascii_digit()) {
                // Channel format like "23.11"
                Ok(format!("nixpkgs/{}#{}", v, package))
            } else if v.len() >= 7 && v.chars().all(|c| c.is_ascii_hexdigit()) {
                // Commit hash format like "cb82756"
                Ok(format!("github:NixOS/nixpkgs/{}#{}", v, package))
            } else if v.contains('.') {
                // Semantic version like "14.1.0" - try to find specific package version
                resolve_semantic_version(package, v).await
            } else {
                // Unknown format, treat as channel
                Ok(format!("nixpkgs/{}#{}", v, package))
            }
        }
        None => Ok(format!("nixpkgs#{}", package)),
    }
}

async fn resolve_semantic_version(package: &str, version: &str) -> Result<String> {
    // Check cache first
    let registry = PackageRegistry::load()?;
    if let Some(cached_url) = registry.get_cached_version(package, version) {
        return Ok(cached_url.clone());
    }

    // Try to find the package in different nixpkgs channels
    let channels = ["nixpkgs/nixos-unstable", "nixpkgs/nixos-23.11", "nixpkgs/nixos-23.05"];
    
    for channel in &channels {
        let output = Command::new("nix")
            .args([
                "--extra-experimental-features", "nix-command",
                "--extra-experimental-features", "flakes",
                "eval",
                &format!("{}#{}.version", channel, package),
                "--json",
            ])
            .output()
            .await?;
            
        if output.status.success() {
            let version_str = String::from_utf8(output.stdout)?;
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&version_str) {
                if let Some(pkg_version) = parsed.as_str() {
                    if pkg_version.starts_with(version) {
                        let flake_url = format!("{}#{}", channel, package);
                        
                        // Cache the result
                        let mut registry = PackageRegistry::load()?;
                        registry.cache_version(package, version, &flake_url);
                        registry.save()?;
                        
                        return Ok(flake_url);
                    }
                }
            }
        }
    }
    
    // Fallback to latest if specific version not found
    let fallback_url = format!("nixpkgs#{}", package);
    
    // Cache the fallback
    let mut registry = PackageRegistry::load()?;
    registry.cache_version(package, version, &fallback_url);
    registry.save()?;
    
    Ok(fallback_url)
}



#[tokio::main]
async fn main() {
    // Parse the CLI arguments
    let cli = Cli::parse();

    // Run the command and handle any errors gracefully
    if let Err(e) = handle_command(cli.command).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}