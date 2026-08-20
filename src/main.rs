mod model;
mod parsers;
mod rules;
mod scanner;

use crate::rules::development_and_shipped_conflict;
use crate::rules::development_and_test_conflict;
use clap::{Parser, Subcommand};
use model::{Role, Runtime};
use parsers::{
    dockerfile::parse_dockerfile, node_version::parse_node_version, nvmrc::parse_nvmrc,
    package_json::parse_package_json,
};
use rules::{development_declarations_conflict, development_outside_support};
use scanner::scan_github_actions_workflows;
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

    let node_version_path = ".node-version";

    match fs::read_to_string(node_version_path) {
        Ok(contents) => {
            if let Some(declaration) = parse_node_version(&contents, node_version_path) {
                declarations.push(declaration);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!("Failed to read {node_version_path}: {error}"));
        }
    }

    let dockerfile_path = "Dockerfile";

    match fs::read_to_string(dockerfile_path) {
        Ok(contents) => {
            declarations.extend(parse_dockerfile(&contents, dockerfile_path));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!("Failed to read {dockerfile_path}: {error}"));
        }
    }

    declarations.extend(scan_github_actions_workflows(".github/workflows")?);

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

    let mut has_issues = false;

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

        has_issues = true;
    }

    if development_declarations_conflict(&declarations)? {
        println!();
        println!("DP002 Conflicting development runtime declarations");

        for declaration in declarations.iter().filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Development
        }) {
            println!(
                "{} selects Node {}",
                declaration.source, declaration.constraint
            );
        }

        has_issues = true;
    }

    if development_and_test_conflict(&declarations)? {
        println!();
        println!("DP003 Development and CI runtimes conflict");

        for declaration in declarations.iter().filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Development
        }) {
            println!(
                "{} selects Node {}",
                declaration.source, declaration.constraint
            );
        }

        for declaration in declarations.iter().filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Test
        }) {
            println!(
                "{} tests Node {}",
                declaration.source, declaration.constraint
            );
        }

        has_issues = true;
    }

    if development_and_shipped_conflict(&declarations)? {
        println!();
        println!("DP004 Development and shipped runtimes conflict");

        for declaration in declarations.iter().filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Development
        }) {
            println!(
                "{} selects Node {}",
                declaration.source, declaration.constraint
            );
        }

        for declaration in declarations.iter().filter(|declaration| {
            declaration.runtime == Runtime::Node && declaration.role == Role::Shipped
        }) {
            println!(
                "{} ships Node {}",
                declaration.source, declaration.constraint
            );
        }

        has_issues = true;
    }

    if !has_issues {
        println!();
        println!("No runtime contract issues found.");
    }

    Ok(has_issues)
}
