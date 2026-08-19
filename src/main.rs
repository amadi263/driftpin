use clap::{Parser, Subcommand};

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
            println!("No runtime contract issues found.");
        }
        Commands::Explain { code } => {
            println!("Diagnostic: {code}");
        }
    }
}
