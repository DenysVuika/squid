use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use log::{debug, error, info};

mod logger;

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
    dotenv().ok();
    logger::init_logger();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    println!("Database URL: {}", db_url);
    println!("API Key: {}", api_key);

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            info!("Initializing project...");
            debug!("This is a debug message during initialization.");
            // Placeholder implementation
        }
        Commands::Run { command } => {
            // println!("Running command: {}", command);
            info!("Running command: {}", command);
            debug!("This is a debug message while running the command.");
            if command == "fail" {
                error!("An error occurred while executing the command.");
            }
            // Placeholder implementation
        }
    }
}
