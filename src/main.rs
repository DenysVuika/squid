use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use log::error;
use std::path::PathBuf;
use std::sync::Arc;

mod agent;
mod api;
mod config;
mod db;
mod doctor;
mod envinfo;
mod init;
mod llm;
mod logger;
mod plugins;
mod rag;
mod server;
mod session;
mod template;
mod tokens;
mod tools;
mod validate;
mod workspace;

#[derive(Parser)]
#[command(name = "squid")]
#[command(about = "squid 🦑: An AI-powered command-line tool for code reviews and suggestions.", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        /// Directory to initialize (defaults to current directory)
        #[arg(default_value = ".")]
        dir: PathBuf,
        /// API URL (skips interactive prompt if provided)
        #[arg(long)]
        url: Option<String>,
        /// API Key (skips interactive prompt if provided)
        #[arg(long)]
        key: Option<String>,
        /// Log Level (skips interactive prompt if provided)
        #[arg(long)]
        log_level: Option<String>,
    },
    /// Ask a question to the LLM
    Ask {
        /// The question to ask
        question: String,
        /// Optional additional context or instructions
        #[arg(short, long)]
        message: Option<String>,
        /// Disable streaming (return complete response at once)
        #[arg(long)]
        no_stream: bool,
        /// Optional file to provide context
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Optional custom system prompt file
        #[arg(short, long)]
        prompt: Option<PathBuf>,
        /// Agent to use (defaults to default_agent from config)
        #[arg(long)]
        agent: Option<String>,
        /// Enable RAG (overrides config setting)
        #[arg(long)]
        rag: bool,
        /// Disable RAG (overrides config setting)
        #[arg(long, conflicts_with = "rag")]
        no_rag: bool,
    },
    /// Review code from a file
    Review {
        /// Path to the file to review
        file: PathBuf,
        /// Optional additional message or specific question about the code
        #[arg(short, long)]
        message: Option<String>,
        /// Disable streaming (return complete response at once)
        #[arg(long)]
        no_stream: bool,
        /// Agent to use (defaults to default_agent from config)
        #[arg(long)]
        agent: Option<String>,
        /// Enable RAG (overrides config setting)
        #[arg(long)]
        rag: bool,
        /// Disable RAG (overrides config setting)
        #[arg(long, conflicts_with = "rag")]
        no_rag: bool,
    },
    /// Start a web server for the Squid Web UI
    Serve {
        /// Port to run the server on
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Custom database path
        #[arg(long)]
        db: Option<PathBuf>,
        /// Custom working directory for the server
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// View application logs from the database
    Logs {
        #[command(subcommand)]
        command: LogCommands,
    },
    /// RAG (Retrieval-Augmented Generation) operations
    Rag {
        #[command(subcommand)]
        command: RagCommands,
    },
    /// Run diagnostic checks to verify configuration and setup
    Doctor,
}

#[derive(Subcommand)]
enum LogCommands {
    /// Display logs from the database
    Show {
        /// Number of log entries to display
        #[arg(short, long, default_value = "100")]
        limit: usize,
        /// Filter by log level (trace, debug, info, warn, error)
        #[arg(short = 'L', long)]
        level: Option<String>,
        /// Filter by session ID
        #[arg(short, long)]
        session_id: Option<String>,
    },
    /// Clear all logs from the database
    Reset,
    /// Remove logs older than a specified number of days
    Cleanup {
        /// Maximum age of logs to keep (in days)
        #[arg(short, long, default_value = "30")]
        max_age_days: u32,
    },
}

#[derive(Subcommand)]
enum RagCommands {
    /// Initialize RAG index by scanning and embedding documents
    Init {
        /// Custom documents directory (defaults to ./documents)
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// List indexed documents
    List,
    /// Rebuild the entire RAG index
    Rebuild {
        /// Custom documents directory (defaults to ./documents)
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Show RAG statistics
    Stats,
}

/// Check if configuration file exists and suggest running init
fn check_config_or_suggest_init() -> bool {
    if !config::Config::config_file_exists() {
        // Check if essential environment variables are set
        if std::env::var("API_URL").is_ok() {
            eprintln!("ℹ️  Using configuration from environment variables (.env file)");
            return true;
        }

        eprintln!("⚠️  No squid.config.json found and no API_URL environment variable set.\n");
        eprintln!("To get started, you can either:");
        eprintln!("  1. Run 'squid init' to create squid.config.json");
        eprintln!("  2. Set API_URL in your .env file\n");
        eprintln!("For squid init, this will guide you through setting up:");
        eprintln!("  • API endpoint configuration");
        eprintln!("  • Default agents (general-assistant, code-reviewer)");
        eprintln!("  • Context window settings");
        eprintln!("  • Optional RAG (document search) features\n");
        return false;
    }
    true
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();

    // Load config early to initialize logger with correct log level
    // For init command, we'll use defaults since config doesn't exist yet
    let mut app_config = if matches!(cli.command, Commands::Init { .. }) {
        config::Config::default()
    } else {
        config::Config::load()
    };

    // Initialize logger with database support only for serve command
    // Other commands use stdout-only logging
    if matches!(cli.command, Commands::Serve { .. }) {
        let db_path_buf = std::path::PathBuf::from(&app_config.database_path);

        // Parse database log level from config
        let db_level = match app_config.db_log_level.to_lowercase().as_str() {
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => {
                eprintln!(
                    "Invalid db_log_level '{}', defaulting to 'debug'",
                    app_config.db_log_level
                );
                log::LevelFilter::Debug
            }
        };

        logger::init_with_db(
            Some(&app_config.log_level),
            Some(db_path_buf),
            Some(db_level),
        );

        // Load agents from files after logger is initialized (so logs are visible)
        app_config.load_agents();
    } else {
        logger::init(Some(&app_config.log_level));

        // Load agents for non-serve commands as well
        app_config.load_agents();
    }

    match &cli.command {
        Commands::Init {
            dir,
            url,
            key: api_key,
            log_level,
        } => {
            init::run(dir, url, api_key, log_level).await;
        }
        Commands::Ask {
            question,
            message,
            no_stream,
            file,
            prompt,
            agent,
            rag,
            no_rag,
        } => {
            if !check_config_or_suggest_init() {
                return;
            }
            llm::run_ask_command(
                question,
                llm::AskCommandOptions {
                    message: message.as_deref(),
                    no_stream: *no_stream,
                    file: file.as_deref(),
                    prompt: prompt.as_deref(),
                    agent: agent.as_deref(),
                    rag_flag: *rag,
                    no_rag_flag: *no_rag,
                },
                &app_config,
            )
            .await;
        }
        Commands::Review {
            file,
            message,
            no_stream,
            agent,
            rag,
            no_rag,
        } => {
            if !check_config_or_suggest_init() {
                return;
            }
            llm::run_review_command(
                file,
                message.as_deref(),
                *no_stream,
                agent.as_deref(),
                *rag,
                *no_rag,
                &app_config,
            )
            .await;
        }
        Commands::Serve { port, db, dir } => {
            if !check_config_or_suggest_init() {
                return;
            }

            server::start_server(*port, db.clone(), dir.clone(), app_config.clone()).await;
        }
        Commands::Logs { command } => {
            let db_path = &app_config.database_path;

            match command {
                LogCommands::Show {
                    limit,
                    level,
                    session_id,
                } => {
                    println!("🦑: Fetching logs from database: {}", db_path);

                    match logger::query_logs(
                        db_path,
                        Some(*limit),
                        level.as_deref(),
                        session_id.as_deref(),
                    ) {
                        Ok(logs) => {
                            if logs.is_empty() {
                                println!("No logs found.");
                            } else {
                                println!("\n{} log entries:\n", logs.len());
                                for log in logs {
                                    let timestamp =
                                        chrono::DateTime::from_timestamp(log.timestamp, 0)
                                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                            .unwrap_or_else(|| "unknown".to_string());

                                    println!(
                                        "[{}] {} {}: {}",
                                        timestamp,
                                        log.level.to_uppercase(),
                                        log.target,
                                        log.message
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to query logs: {}", e);
                            println!("🦑: Failed to read logs from database - {}", e);
                            println!("    Database path: {}", db_path);
                            println!("    Make sure the database exists and is not corrupted.");
                        }
                    }
                }
                LogCommands::Reset => {
                    println!("🦑: Clearing all logs from database: {}", db_path);

                    match logger::reset_logs(db_path) {
                        Ok(count) => {
                            println!("✓ Successfully cleared {} log entries.", count);
                        }
                        Err(e) => {
                            error!("Failed to reset logs: {}", e);
                            println!("🦑: Failed to clear logs from database - {}", e);
                            println!("    Database path: {}", db_path);
                        }
                    }
                }
                LogCommands::Cleanup { max_age_days } => {
                    let max_age_seconds = (*max_age_days as i64) * 24 * 60 * 60;
                    println!(
                        "🦑: Removing logs older than {} day(s) from database: {}",
                        max_age_days, db_path
                    );

                    match logger::cleanup_old_logs(db_path, max_age_seconds) {
                        Ok(count) => {
                            println!("✓ Successfully removed {} old log entries.", count);
                        }
                        Err(e) => {
                            error!("Failed to clean up logs: {}", e);
                            println!("🦑: Failed to remove old logs from database - {}", e);
                            println!("    Database path: {}", db_path);
                        }
                    }
                }
            }
        }
        Commands::Rag { command } => {
            let db_path = &app_config.database_path;
            let db = match db::Database::new(db_path) {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    error!("Failed to open database: {}", e);
                    println!("🦑: Failed to open database - {}", e);
                    return;
                }
            };

            let rag_config = &app_config.rag;

            if !rag_config.enabled {
                println!("🦑: RAG is disabled in configuration");
                println!(
                    "    Set 'rag.enabled = true' in squid.config.json to enable RAG features"
                );
                return;
            }

            let rag_system = match rag::RagSystem::new(db.clone(), rag_config).await {
                Ok(system) => system,
                Err(e) => {
                    error!("Failed to initialize RAG system: {}", e);
                    println!("🦑: Failed to initialize RAG system - {}", e);
                    return;
                }
            };

            match command {
                RagCommands::Init { dir } => {
                    let documents_path = dir
                        .clone()
                        .unwrap_or_else(|| PathBuf::from(&rag_config.documents_path));

                    if !documents_path.exists() {
                        println!(
                            "🦑: Documents directory not found: {}",
                            documents_path.display()
                        );
                        println!("    Create the directory and add documents to index");
                        return;
                    }

                    // Test embedding service connection with a simple request
                    println!("🦑: Testing embedding service connection...");
                    match rag_system.indexer.embedder.embed_text("test").await {
                        Ok(_) => {
                            println!("✓ Embedding service is accessible");
                        }
                        Err(e) => {
                            println!("✗ Embedding service connection failed:");
                            println!("    {}", e);
                            println!("\nTroubleshooting:");
                            println!(
                                "  1. Check if embedding service is running at: {}",
                                rag_config.embedding_url
                            );
                            println!(
                                "  2. Verify the embedding model '{}' is loaded",
                                rag_config.embedding_model
                            );
                            println!("  3. For Ollama: run 'ollama pull nomic-embed-text'");
                            println!("  4. For LM Studio: ensure an embedding model is loaded");
                            println!("\nUpdate config with: squid init");
                            return;
                        }
                    }

                    println!(
                        "🦑: Scanning documents directory: {}",
                        documents_path.display()
                    );

                    let pb = indicatif::ProgressBar::new_spinner();
                    pb.set_message("Indexing documents...");
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));

                    match rag_system.indexer.scan_and_index(&documents_path).await {
                        Ok(stats) => {
                            pb.finish_and_clear();
                            println!("🦑: Indexing complete!");
                            println!("    Files found: {}", stats.files_found);
                            println!("    Files processed: {}", stats.files_processed);
                            if stats.files_failed > 0 {
                                println!("    Files failed: {}", stats.files_failed);
                            }
                            println!("    Total chunks: {}", stats.total_chunks);
                            println!("    Total embeddings: {}", stats.total_embeddings);
                        }
                        Err(e) => {
                            pb.finish_and_clear();
                            error!("Failed to index documents: {}", e);
                            println!("🦑: Failed to index documents - {}", e);
                        }
                    }
                }
                RagCommands::List => match rag_system.indexer.list_documents() {
                    Ok(docs) => {
                        if docs.is_empty() {
                            println!("🦑: No documents indexed");
                            println!("    Run 'squid rag init' to index documents");
                        } else {
                            println!("🦑: Indexed documents:\n");
                            for doc in &docs {
                                let updated = chrono::DateTime::from_timestamp(doc.updated_at, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "unknown".to_string());
                                println!(
                                    "  {} ({} bytes, updated: {})",
                                    doc.filename, doc.file_size, updated
                                );
                            }
                            println!("\nTotal: {} documents", docs.len());
                        }
                    }
                    Err(e) => {
                        error!("Failed to list documents: {}", e);
                        println!("🦑: Failed to list documents - {}", e);
                    }
                },
                RagCommands::Rebuild { dir } => {
                    let documents_path = dir
                        .clone()
                        .unwrap_or_else(|| PathBuf::from(&rag_config.documents_path));

                    if !documents_path.exists() {
                        println!(
                            "🦑: Documents directory not found: {}",
                            documents_path.display()
                        );
                        return;
                    }

                    println!("🦑: Rebuilding RAG index...");

                    let pb = indicatif::ProgressBar::new_spinner();
                    pb.set_message("Rebuilding index...");
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));

                    match rag_system.indexer.rebuild(&documents_path).await {
                        Ok(stats) => {
                            pb.finish_and_clear();
                            println!("🦑: Rebuild complete!");
                            println!("    Files found: {}", stats.files_found);
                            println!("    Files processed: {}", stats.files_processed);
                            if stats.files_failed > 0 {
                                println!("    Files failed: {}", stats.files_failed);
                            }
                            println!("    Total chunks: {}", stats.total_chunks);
                            println!("    Total embeddings: {}", stats.total_embeddings);
                        }
                        Err(e) => {
                            pb.finish_and_clear();
                            error!("Failed to rebuild index: {}", e);
                            println!("🦑: Failed to rebuild index - {}", e);
                        }
                    }
                }
                RagCommands::Stats => match rag_system.indexer.get_stats() {
                    Ok((doc_count, chunk_count, embedding_count)) => {
                        println!("🦑: RAG Statistics:\n");
                        println!("  Documents: {}", doc_count);
                        println!("  Chunks: {}", chunk_count);
                        println!("  Embeddings: {}", embedding_count);
                        if doc_count > 0 {
                            let avg_chunks = chunk_count as f64 / doc_count as f64;
                            println!("  Average chunks per document: {:.1}", avg_chunks);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get stats: {}", e);
                        println!("🦑: Failed to get statistics - {}", e);
                    }
                },
            }
        }
        Commands::Doctor => {
            if !check_config_or_suggest_init() {
                return;
            }

            let doctor = doctor::Doctor::new();
            let all_passed = doctor.run(&app_config).await;

            if !all_passed {
                std::process::exit(1);
            }
        }
    }
}
