use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::process::Stdio;
use tokio::process::Command;

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
          let flake_url = build_flake_url(&package, version.as_deref())?;
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
            run_nix_command(vec![
                "profile",
                "add",
                &format!("nixpkgs#{}", package),
                "--pin",
                &version,
            ])
            .await
        }
    }
}
fn build_flake_url(package: &str, version: Option<&str>) -> Result<String> {
    match version {
        Some(v) => {
            // Handle different version formats
            if v.matches('.').count() == 1 && v.chars().all(|c| c.is_ascii_digit()) {
                // Channel format like "23.11"
                Ok(format!("nixpkgs/{}#{}", v, package))
            } else if v.len() == 7 && v.chars().all(|c| c.is_ascii_hexdigit()) {
                // Commit hash format like "cb82756"
                Ok(format!("github:NixOS/nixpkgs/{}#{}", v, package))
            } else if v.contains('.') {
                // Semantic version like "14.1.0" - need to find specific package version
                // This is more complex and requires searching nixpkgs history
                Ok(format!("nixpkgs#{}", package)) // Fallback for now
            } else {
                // Unknown format, treat as channel
                Ok(format!("nixpkgs/{}#{}", v, package))
            }
        }
        None => Ok(format!("nixpkgs#{}", package)),
    }
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