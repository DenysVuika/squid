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
use std::io::{self, Write};
use std::path::PathBuf;

mod logger;

const CODE_REVIEW_PROMPT: &str = include_str!("./assets/code-review.md");
const CODE_REVIEW_RUST_PROMPT: &str = include_str!("./assets/review-rust.md");
const CODE_REVIEW_TYPESCRIPT_PROMPT: &str = include_str!("./assets/review-typescript.md");
const CODE_REVIEW_HTML_PROMPT: &str = include_str!("./assets/review-html.md");
const CODE_REVIEW_CSS_PROMPT: &str = include_str!("./assets/review-css.md");

fn get_review_prompt_for_file(file_path: &PathBuf) -> &'static str {
    if let Some(extension) = file_path.extension() {
        match extension.to_str() {
            Some("rs") => CODE_REVIEW_RUST_PROMPT,
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => {
                CODE_REVIEW_TYPESCRIPT_PROMPT
            }
            Some("html") | Some("htm") => CODE_REVIEW_HTML_PROMPT,
            Some("css") | Some("scss") | Some("sass") | Some("less") => CODE_REVIEW_CSS_PROMPT,
            _ => CODE_REVIEW_PROMPT,
        }
    } else {
        CODE_REVIEW_PROMPT
    }
}

async fn ask_llm_streaming(
    question: &str,
    file_content: Option<&str>,
    system_prompt: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    // Build the user message, including file content if provided
    let user_message = if let Some(content) = file_content {
        format!(
            "Here is the content of the file:\n\n```\n{}\n```\n\nQuestion: {}",
            content, question
        )
    } else {
        question.to_string()
    };

    // Use custom system prompt or default
    let system_message = system_prompt.unwrap_or(
        "You are a helpful assistant. When provided with file content, analyze it carefully and answer questions based on that content."
    );

    // Create the chat completion request
    let request = CreateChatCompletionRequestArgs::default()
        .model(api_model)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_message)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_message)
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

async fn ask_llm(
    question: &str,
    file_content: Option<&str>,
    system_prompt: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
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

    // Build the user message, including file content if provided
    let user_message = if let Some(content) = file_content {
        format!(
            "Here is the content of the file:\n\n```\n{}\n```\n\nQuestion: {}",
            content, question
        )
    } else {
        question.to_string()
    };

    // Use custom system prompt or default
    let system_message = system_prompt.unwrap_or(
        "You are a helpful assistant. When provided with file content, analyze it carefully and answer questions based on that content."
    );

    // Create the chat completion request
    let request = CreateChatCompletionRequestArgs::default()
        .model(api_model)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_message)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_message)
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
#[command(about = "squid 🦑: An AI-powered command-line tool for code reviews and suggestions.", long_about = None)]
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
        /// Stream the response
        #[arg(short, long)]
        stream: bool,
        /// Optional file to provide context
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Review code from a file
    Review {
        /// Path to the file to review
        file: PathBuf,
        /// Optional additional message or specific question about the code
        #[arg(short, long)]
        message: Option<String>,
        /// Stream the response
        #[arg(short, long)]
        stream: bool,
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
        Commands::Ask {
            question,
            stream,
            file,
        } => {
            info!("Asking question: {}", question);

            // Read file content if provided
            let file_content = if let Some(file_path) = file {
                match std::fs::read_to_string(file_path) {
                    Ok(content) => {
                        info!("Read file content ({} bytes)", content.len());
                        Some(content)
                    }
                    Err(e) => {
                        error!("Failed to read file: {}", e);
                        return;
                    }
                }
            } else {
                None
            };

            if *stream {
                if let Err(e) = ask_llm_streaming(question, file_content.as_deref(), None).await {
                    error!("Failed to get response: {}", e);
                }
            } else {
                match ask_llm(question, file_content.as_deref(), None).await {
                    Ok(response) => {
                        println!("\n{}", response);
                    }
                    Err(e) => {
                        error!("Failed to get response: {}", e);
                    }
                }
            }
        }
        Commands::Review {
            file,
            message,
            stream,
        } => {
            info!("Reviewing file: {:?}", file);

            // Read file content
            let file_content = match std::fs::read_to_string(file) {
                Ok(content) => {
                    info!("Read file content ({} bytes)", content.len());
                    content
                }
                Err(e) => {
                    error!("Failed to read file: {}", e);
                    return;
                }
            };

            // Get the appropriate review prompt based on file type
            let review_prompt = get_review_prompt_for_file(file);
            debug!("Using review prompt for file type");

            // Build the review question
            let question = if let Some(msg) = message {
                format!("Please review this code. {}", msg)
            } else {
                "Please review this code.".to_string()
            };

            if *stream {
                if let Err(e) =
                    ask_llm_streaming(&question, Some(&file_content), Some(review_prompt)).await
                {
                    error!("Failed to get review: {}", e);
                }
            } else {
                match ask_llm(&question, Some(&file_content), Some(review_prompt)).await {
                    Ok(response) => {
                        println!("\n{}", response);
                    }
                    Err(e) => {
                        error!("Failed to get review: {}", e);
                    }
                }
            }
        }
    }
}
