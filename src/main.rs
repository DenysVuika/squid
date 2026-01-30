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
    // Configure the client to use LM Studio's endpoint
    let config = OpenAIConfig::new()
        .with_api_base("http://127.0.0.1:1234/v1")
        .with_api_key("not-needed"); // LM Studio doesn't require a real API key

    let client = Client::with_config(config);

    // Create the chat completion request
    let request = CreateChatCompletionRequestArgs::default()
        .model("local-model") // LM Studio uses whatever model is loaded
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

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    println!("Database URL: {}", db_url);
    println!("API Key: {}", api_key);

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
