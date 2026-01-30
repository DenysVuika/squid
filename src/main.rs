use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "squid")]
#[command(about = "A basic CLI application", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init,
    /// Run a specific command
    Run {
        /// The command to run
        command: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            println!("Initializing project...");
            // Placeholder implementation
        }
        Commands::Run { command } => {
            println!("Running command: {}", command);
            // Placeholder implementation
        }
    }
}
