mod model;
mod parsers;
mod rules;

use clap::{Parser, Subcommand};
use model::{Role, Runtime};
use parsers::{nvmrc::parse_nvmrc, package_json::parse_package_json};
use rules::development_outside_support;
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
        /// Diagnostic code, for example DP001
        code: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => match run_check() {
            Ok(has_issues) => {
                if has_issues {
                    process::exit(1);
                }
            }
            Err(message) => {
                eprintln!("{message}");
                process::exit(2);
            }
        },
        Commands::Explain { code } => {
            println!("Diagnostic: {code}");
        }
    }
}

fn run_check() -> Result<bool, String> {
    let mut declarations = Vec::new();

    let package_path = "package.json";

    match fs::read_to_string(package_path) {
        Ok(contents) => {
            if let Some(declaration) = parse_package_json(&contents, package_path)
                .map_err(|error| format!("Failed to parse {package_path}: {error}"))?
            {
                declarations.push(declaration);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!("Failed to read {package_path}: {error}"));
        }
    }

    let nvmrc_path = ".nvmrc";

    match fs::read_to_string(nvmrc_path) {
        Ok(contents) => {
            if let Some(declaration) = parse_nvmrc(&contents, nvmrc_path) {
                declarations.push(declaration);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!("Failed to read {nvmrc_path}: {error}"));
        }
    }

    if declarations.is_empty() {
        println!("No runtime declarations found.");
        return Ok(false);
    }

    println!("Found runtime declarations:");

    for declaration in &declarations {
        println!();
        println!("  source: {}", declaration.source);
        println!("  runtime: {:?}", declaration.runtime);
        println!("  constraint: {}", declaration.constraint);
        println!("  role: {:?}", declaration.role);
    }

    if development_outside_support(&declarations)? {
        let support = declarations.iter().find(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Support
        });

        let development = declarations.iter().find(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Development
        });

        println!();
        println!("DP001 Development runtime is outside supported range");

        if let (Some(support), Some(development)) = (support, development) {
            println!();
            println!("{} declares Node {}", support.source, support.constraint);
            println!(
                "{} selects Node {}",
                development.source, development.constraint
            );
        }

        return Ok(true);
    }

    println!();
    println!("No runtime contract issues found.");

    Ok(false)
}
