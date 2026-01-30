use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use log::{debug, error, info};

mod logger;

async fn ask_llm(question: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Load configuration from environment variables
    let api_url =
        std::env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:1234/v1".to_string());
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "not-needed".to_string());
    let api_model = std::env::var("API_MODEL").unwrap_or_else(|_| "local-model".to_string());

    debug!("Using API URL: {}", api_url);
    debug!("Using API Model: {}", api_model);

    // Configure the client to use the specified endpoint
    let config = OpenAIConfig::new()
        .with_api_base(api_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    // Create the chat completion request
    let request = CreateChatCompletionRequestArgs::default()
        .model(api_model)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful assistant.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(question)
                .build()?
                .into(),
        ])
        .build()?;

    debug!("Sending request to LM Studio...");

    // Send the request and get the response
    let response = client.chat().create(request).await?;

    // Extract the response text
    let answer = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .ok_or("No response from LLM")?;

    Ok(answer.to_string())
}

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
    /// Ask a question to the LLM
    Ask {
        /// The question to ask
        question: String,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::init_logger();

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "not-set".to_string());
    debug!("Database URL: {}", db_url);

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            info!("Initializing project...");
            // Placeholder implementation
        }
        Commands::Run { command } => {
            info!("Running command: {}", command);
            debug!("This is a debug message while running the command.");
            if command == "fail" {
                error!("An error occurred while executing the command.");
            }
            // Placeholder implementation
        }
        Commands::Ask { question } => {
            info!("Asking question: {}", question);
            match ask_llm(question).await {
                Ok(response) => {
                    println!("\n{}", response);
                }
                Err(e) => {
                    error!("Failed to get response: {}", e);
                }
            }
        }
    }
}
