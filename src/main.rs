mod model;
mod parsers;

use clap::{Parser, Subcommand};
use parsers::package_json::parse_package_json;
use std::{fs, process};

#[derive(Parser)]
#[command(
    name = "driftpin",
    version,
    about = "Zero-config runtime contract auditor",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check runtime contracts in the current repository
    Check,

    /// Explain a DriftPin diagnostic code
    Explain {
        /// Diagnostic code, for example DP004
        code: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => {
            if let Err(message) = run_check() {
                eprintln!("{message}");
                process::exit(2);
            }
        }
        Commands::Explain { code } => {
            println!("Diagnostic: {code}");
        }
    }
}

fn run_check() -> Result<(), String> {
    let path = "package.json";

    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            println!("No package.json found.");
            return Ok(());
        }
        Err(error) => {
            return Err(format!("Failed to read {path}: {error}"));
        }
    };

    match parse_package_json(&contents, path)
        .map_err(|error| format!("Failed to parse {path}: {error}"))?
    {
        Some(declaration) => {
            println!("Found runtime declaration:");
            println!("  source: {}", declaration.source);
            println!("  runtime: {:?}", declaration.runtime);
            println!("  constraint: {}", declaration.constraint);
            println!("  role: {:?}", declaration.role);
        }
        None => {
            println!("No Node.js runtime declaration found in package.json.");
        }
    }

    Ok(())
}
