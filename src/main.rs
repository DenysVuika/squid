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
use futures::StreamExt;
use log::{debug, error, info};
use mistralrs::{
    IsqType, PagedAttentionMetaBuilder, TextMessageRole, TextMessages, TextModelBuilder,
};
use std::io::{self, Write};

mod logger;

async fn ask_llm_streaming(question: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    debug!("Sending streaming request...");

    // Send the request and get the streaming response
    let mut stream = client.chat().create_stream(request).await?;

    // Lock stdout once before the loop to avoid locking on each iteration
    let mut lock = io::stdout().lock();
    writeln!(lock)?;

    // Process the stream
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                response.choices.iter().for_each(|chat_choice| {
                    if let Some(ref content) = chat_choice.delta.content {
                        write!(lock, "{}", content).unwrap();
                    }
                });
            }
            Err(err) => {
                writeln!(lock, "error: {err:?}").unwrap();
            }
        }
        lock.flush()?;
    }

    writeln!(lock)?;
    Ok(())
}

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

async fn ask_local_model(model_id: &str, question: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading local model: {}", model_id);
    println!("Loading model from HuggingFace: {}", model_id);
    println!("This may take a while on first run...\n");

    // Build the model with auto-detection
    let model = TextModelBuilder::new(model_id)
        .with_isq(IsqType::Q4K) // 4-bit quantization for efficiency
        .with_logging()
        .with_paged_attn(|| PagedAttentionMetaBuilder::default().build())?
        .build()
        .await?;

    debug!("Model loaded successfully");

    // Create the messages
    let messages = TextMessages::new().add_message(TextMessageRole::User, question);

    // Send the chat request
    let response = model.send_chat_request(messages).await?;

    // Print the response
    println!("\n{:?}\n", response.choices[0].message.content);

    Ok(())
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
    /// Ask a question to the LLM (via API)
    Ask {
        /// The question to ask
        question: String,
        /// Stream the response
        #[arg(short, long)]
        stream: bool,
    },
    /// Ask a question using a local model from HuggingFace
    AskLocal {
        /// The HuggingFace model ID (e.g., "LiquidAI/LFM2.5-1.2B-Instruct")
        #[arg(short, long)]
        model: String,
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
        Commands::Ask { question, stream } => {
            info!("Asking question: {}", question);
            if *stream {
                if let Err(e) = ask_llm_streaming(question).await {
                    error!("Failed to get response: {}", e);
                }
            } else {
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
        Commands::AskLocal { model, question } => {
            info!("Using local model: {}", model);
            if let Err(e) = ask_local_model(model, question).await {
                error!("Failed to get response from local model: {}", e);
                eprintln!("\nTroubleshooting:");
                eprintln!(
                    "- Check if the model architecture is supported: https://ericlbuehler.github.io/mistral.rs/supported_models.html"
                );
                eprintln!("- Try a tested model like: Qwen/Qwen2.5-0.5B-Instruct");
                eprintln!("- Ensure you have internet connection for first-time model download");
            }
        }
    }
}
