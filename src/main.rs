use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use log::error;
use std::path::PathBuf;
use std::sync::Arc;
use tabled::{Table, Tabled};

mod agent;
mod api;
mod audio;
mod bundled;
mod config;
mod db;
mod doctor;
mod init;
mod jobs;
mod jobs_api;
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
    /// Manage background jobs (cron schedules and one-off tasks)
    Jobs {
        #[command(subcommand)]
        command: JobCommands,
    },
    /// Clean up bundled assets extracted from the binary
    Cleanup,
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

#[derive(Subcommand)]
enum JobCommands {
    /// List all background jobs
    List {
        /// Filter by status (pending, running, completed, failed, cancelled)
        #[arg(short, long)]
        status: Option<String>,
        /// Filter by schedule type (cron, once)
        #[arg(short = 't', long)]
        schedule_type: Option<String>,
    },
    /// Show details of a specific job
    Show {
        /// Job ID to display
        id: i64,
    },
    /// Create a new background job (interactive mode if no flags provided)
    Create {
        /// Job name (interactive prompt if not provided)
        #[arg(short, long)]
        name: Option<String>,
        /// Agent ID to use (interactive selection if not provided)
        #[arg(short, long)]
        agent: Option<String>,
        /// Message/prompt for the agent (interactive prompt if not provided)
        #[arg(short, long)]
        message: Option<String>,
        /// Schedule type: cron or once
        #[arg(short = 't', long)]
        schedule_type: Option<String>,
        /// Cron expression (required for cron jobs, e.g., "0 0 9 * * Mon-Fri")
        #[arg(short, long)]
        cron: Option<String>,
        /// Optional file path to provide as context
        #[arg(short, long)]
        file: Option<String>,
        /// Job priority (0-10, higher = runs first)
        #[arg(short, long)]
        priority: Option<i32>,
        /// Maximum CPU percent threshold
        #[arg(long)]
        max_cpu: Option<i32>,
        /// Timeout in seconds (0 = no timeout)
        #[arg(long)]
        timeout: Option<i64>,
    },
    /// Delete a background job
    Delete {
        /// Job ID to delete
        id: i64,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Pause a cron job (stops it from running on schedule)
    Pause {
        /// Job ID to pause
        id: i64,
    },
    /// Resume a paused cron job
    Resume {
        /// Job ID to resume
        id: i64,
    },
    /// Manually trigger a cron job (runs immediately, outside normal schedule)
    Trigger {
        /// Job ID to trigger
        id: i64,
    },
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
        Commands::Jobs { command } => {
            if !check_config_or_suggest_init() {
                return;
            }

            // Check if jobs are enabled
            if !app_config.jobs.enabled {
                println!("🦑: Background jobs are disabled in configuration");
                println!("    Set 'jobs.enabled = true' in squid.config.json to enable jobs");
                return;
            }

            let db_path = &app_config.database_path;
            let db = match db::Database::new(db_path) {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    error!("Failed to open database: {}", e);
                    println!("🦑: Failed to open database - {}", e);
                    return;
                }
            };

            match command {
                JobCommands::List {
                    status,
                    schedule_type,
                } => {
                    println!("🦑: Fetching jobs from database...\n");

                    match db.get_all_jobs() {
                        Ok(jobs) => {
                            if jobs.is_empty() {
                                println!("No jobs found.");
                                println!("\nCreate a job with: squid jobs create");
                                return;
                            }

                            // Filter jobs
                            let filtered_jobs: Vec<_> = jobs
                                .into_iter()
                                .filter(|job| {
                                    let status_match = status
                                        .as_ref()
                                        .is_none_or(|s| job.status.eq_ignore_ascii_case(s));
                                    let type_match = schedule_type
                                        .as_ref()
                                        .is_none_or(|t| job.schedule_type.eq_ignore_ascii_case(t));
                                    status_match && type_match
                                })
                                .collect();

                            if filtered_jobs.is_empty() {
                                println!("No jobs match the specified filters.");
                                return;
                            }

                            // Create table rows
                            #[derive(Tabled)]
                            struct JobRow {
                                #[tabled(rename = "")]
                                active: String,
                                #[tabled(rename = "ID")]
                                id: String,
                                #[tabled(rename = "Name")]
                                name: String,
                                #[tabled(rename = "Type")]
                                job_type: String,
                                #[tabled(rename = "Status")]
                                status: String,
                                #[tabled(rename = "Schedule")]
                                schedule: String,
                                #[tabled(rename = "Priority")]
                                priority: String,
                                #[tabled(rename = "Retries")]
                                retries: String,
                            }

                            let rows: Vec<JobRow> = filtered_jobs
                                .iter()
                                .map(|job| {
                                    let id = job.id.unwrap_or(0);
                                    let active = if job.is_active { "●" } else { "○" }.to_string();
                                    let schedule = job
                                        .cron_expression
                                        .as_ref()
                                        .map(|s| {
                                            // Truncate long cron expressions
                                            if s.len() > 20 {
                                                format!("{}...", &s[..17])
                                            } else {
                                                s.clone()
                                            }
                                        })
                                        .unwrap_or_else(|| "-".to_string());

                                    JobRow {
                                        active,
                                        id: id.to_string(),
                                        name: if job.name.len() > 30 {
                                            format!("{}...", &job.name[..27])
                                        } else {
                                            job.name.clone()
                                        },
                                        job_type: job.schedule_type.clone(),
                                        status: job.status.clone(),
                                        schedule,
                                        priority: job.priority.to_string(),
                                        retries: format!("{}/{}", job.retries, job.max_retries),
                                    }
                                })
                                .collect();

                            let table = Table::new(rows).to_string();
                            println!("{}", table);
                            println!("\n{} job(s) found", filtered_jobs.len());
                            println!("\nView details: squid jobs show <id>");
                        }
                        Err(e) => {
                            error!("Failed to list jobs: {}", e);
                            println!("🦑: Failed to list jobs - {}", e);
                        }
                    }
                }
                JobCommands::Show { id } => match db.get_job_by_id(*id) {
                    Ok(Some(job)) => {
                        println!("🦑: Job Details\n");
                        println!("  ID: {}", job.id.unwrap_or(0));
                        println!("  Name: {}", job.name);
                        println!("  Status: {}", job.status);
                        println!("  Type: {}", job.schedule_type);
                        println!(
                            "  Active: {}",
                            if job.is_active { "Yes" } else { "No (Paused)" }
                        );

                        if let Some(cron) = &job.cron_expression {
                            println!("  Cron expression: {}", cron);
                        }

                        println!("  Priority: {}", job.priority);
                        println!("  Max CPU: {}%", job.max_cpu_percent);
                        println!("  Timeout: {}s", job.timeout_seconds);
                        println!("  Retries: {}/{}", job.retries, job.max_retries);

                        if let Some(last_run) = &job.last_run {
                            println!("  Last run: {}", last_run);
                        }

                        if let Some(next_run) = &job.next_run {
                            println!("  Next run: {}", next_run);
                        }

                        println!("\n  Payload:");
                        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&job.payload)
                        {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&payload)
                                    .unwrap_or(job.payload.clone())
                            );
                        } else {
                            println!("{}", job.payload);
                        }

                        if let Some(result) = &job.result {
                            println!("\n  Result:");
                            if let Ok(result_json) =
                                serde_json::from_str::<serde_json::Value>(result)
                            {
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(&result_json)
                                        .unwrap_or(result.clone())
                                );
                            } else {
                                println!("{}", result);
                            }
                        }

                        if let Some(error) = &job.error_message {
                            println!("\n  Error: {}", error);
                        }
                    }
                    Ok(None) => {
                        println!("🦑: Job {} not found", id);
                    }
                    Err(e) => {
                        error!("Failed to get job: {}", e);
                        println!("🦑: Failed to get job - {}", e);
                    }
                },
                JobCommands::Create {
                    name,
                    agent,
                    message,
                    schedule_type,
                    cron,
                    file,
                    priority,
                    max_cpu,
                    timeout,
                } => {
                    // Get available agents
                    let available_agents: Vec<String> =
                        app_config.agents.agents.keys().cloned().collect();

                    if available_agents.is_empty() {
                        println!("🦑: No agents found in configuration");
                        println!("    Create agents in the agents/ directory");
                        return;
                    }

                    // Interactive mode: prompt for job name if not provided
                    let job_name = if let Some(n) = name {
                        n.clone()
                    } else {
                        match inquire::Text::new("Job name:")
                            .with_help_message("A descriptive name for this job")
                            .prompt()
                        {
                            Ok(name) => name,
                            Err(_) => {
                                println!("Job creation cancelled.");
                                return;
                            }
                        }
                    };

                    // Select or validate agent
                    let selected_agent = if let Some(agent_id) = agent {
                        // Validate provided agent exists
                        if !available_agents.contains(agent_id) {
                            println!("🦑: Agent '{}' not found", agent_id);
                            println!("\nAvailable agents:");
                            for id in &available_agents {
                                if let Some(agent_config) = app_config.get_agent(id) {
                                    println!("  - {} ({})", id, agent_config.name);
                                } else {
                                    println!("  - {}", id);
                                }
                            }
                            return;
                        }
                        agent_id.clone()
                    } else {
                        // Interactive agent selection
                        let agent_options: Vec<String> = available_agents
                            .iter()
                            .map(|id| {
                                if let Some(agent_config) = app_config.get_agent(id) {
                                    format!("{} - {}", id, agent_config.name)
                                } else {
                                    id.clone()
                                }
                            })
                            .collect();

                        let selected_idx =
                            match inquire::Select::new("Select agent:", agent_options.clone())
                                .with_help_message("Choose which agent will execute this job")
                                .prompt()
                            {
                                Ok(selection) => agent_options
                                    .iter()
                                    .position(|opt| opt == &selection)
                                    .unwrap(),
                                Err(_) => {
                                    println!("Job creation cancelled.");
                                    return;
                                }
                            };

                        available_agents[selected_idx].clone()
                    };

                    // Interactive mode: prompt for message if not provided
                    let job_message = if let Some(m) = message {
                        m.clone()
                    } else {
                        match inquire::Text::new("Message/prompt:")
                            .with_help_message("The prompt or question for the agent to process")
                            .prompt()
                        {
                            Ok(msg) => msg,
                            Err(_) => {
                                println!("Job creation cancelled.");
                                return;
                            }
                        }
                    };

                    // Interactive mode: select schedule type if not provided
                    let job_schedule_type = if let Some(st) = schedule_type {
                        st.clone()
                    } else {
                        let schedule_options = vec!["once", "cron"];
                        match inquire::Select::new("Schedule type:", schedule_options)
                            .with_help_message("'once' runs immediately, 'cron' runs on a schedule")
                            .prompt()
                        {
                            Ok(selection) => selection.to_string(),
                            Err(_) => {
                                println!("Job creation cancelled.");
                                return;
                            }
                        }
                    };

                    // Validate schedule type and get cron expression
                    let job_cron = if job_schedule_type == "cron" {
                        if let Some(c) = cron {
                            Some(c.clone())
                        } else {
                            match inquire::Text::new("Cron expression:")
                                .with_help_message("6-field format: sec min hour day month dayofweek (e.g., \"0 0 9 * * Mon-Fri\")")
                                .prompt()
                            {
                                Ok(expr) => Some(expr),
                                Err(_) => {
                                    println!("Job creation cancelled.");
                                    return;
                                }
                            }
                        }
                    } else {
                        cron.clone()
                    };

                    // Get optional parameters with defaults
                    let job_priority = priority.unwrap_or(5);
                    let job_max_cpu = max_cpu.unwrap_or(70);
                    let job_timeout = timeout.unwrap_or(3600);

                    // Create job payload
                    let payload = db::JobPayload {
                        agent_id: selected_agent.clone(),
                        message: job_message.clone(),
                        system_prompt: None,
                        file_path: file.clone(),
                        session_id: None,
                    };

                    let payload_json = serde_json::to_string(&payload).unwrap();

                    let job = db::BackgroundJob {
                        id: None,
                        name: job_name.clone(),
                        schedule_type: job_schedule_type.clone(),
                        cron_expression: job_cron.clone(),
                        priority: job_priority,
                        max_cpu_percent: job_max_cpu,
                        status: "pending".to_string(),
                        last_run: None,
                        next_run: None,
                        retries: 0,
                        max_retries: app_config.jobs.default_retries,
                        payload: payload_json,
                        result: None,
                        error_message: None,
                        is_active: true,
                        timeout_seconds: job_timeout,
                    };

                    match db.create_job(&job) {
                        Ok(job_id) => {
                            println!("✓ Job created successfully!");
                            println!("  Job ID: {}", job_id);
                            println!("  Name: {}", job_name);
                            println!("  Agent: {}", selected_agent);
                            println!("  Type: {}", job_schedule_type);
                            if let Some(cron_expr) = &job_cron {
                                println!("  Schedule: {}", cron_expr);
                            }
                            println!("\nView details: squid jobs show {}", job_id);

                            if job_schedule_type == "cron" {
                                println!(
                                    "\nNote: The job scheduler must be running for cron jobs to execute."
                                );
                                println!("      Start the server with: squid serve");
                            }
                        }
                        Err(e) => {
                            error!("Failed to create job: {}", e);
                            println!("🦑: Failed to create job - {}", e);
                        }
                    }
                }
                JobCommands::Delete { id, yes } => {
                    // Get job first to show what will be deleted
                    match db.get_job_by_id(*id) {
                        Ok(Some(job)) => {
                            if !yes {
                                println!("🦑: About to delete job:");
                                println!("  ID: {}", id);
                                println!("  Name: {}", job.name);
                                println!("  Type: {}", job.schedule_type);
                                println!("  Status: {}", job.status);
                                print!("\nAre you sure? (y/N): ");

                                use std::io::{self, Write};
                                io::stdout().flush().unwrap();

                                let mut response = String::new();
                                io::stdin().read_line(&mut response).unwrap();

                                if !response.trim().eq_ignore_ascii_case("y") {
                                    println!("Cancelled.");
                                    return;
                                }
                            }

                            match db.delete_job(*id) {
                                Ok(_) => {
                                    println!("✓ Job {} deleted successfully", id);
                                }
                                Err(e) => {
                                    error!("Failed to delete job: {}", e);
                                    println!("🦑: Failed to delete job - {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            println!("🦑: Job {} not found", id);
                        }
                        Err(e) => {
                            error!("Failed to get job: {}", e);
                            println!("🦑: Failed to get job - {}", e);
                        }
                    }
                }
                JobCommands::Pause { id } => match db.pause_job(*id) {
                    Ok(_) => {
                        println!("✓ Job {} paused", id);
                        println!("  The job will not run on its schedule until resumed.");
                        println!("  Resume with: squid jobs resume {}", id);
                    }
                    Err(e) => {
                        error!("Failed to pause job: {}", e);
                        println!("🦑: Failed to pause job - {}", e);
                        println!("    Note: Only cron jobs can be paused");
                    }
                },
                JobCommands::Resume { id } => match db.resume_job(*id) {
                    Ok(_) => {
                        println!("✓ Job {} resumed", id);
                        println!("  The job will run according to its schedule.");
                    }
                    Err(e) => {
                        error!("Failed to resume job: {}", e);
                        println!("🦑: Failed to resume job - {}", e);
                        println!("    Note: Only cron jobs can be resumed");
                    }
                },
                JobCommands::Trigger { id } => {
                    // First check if job exists and is a cron job
                    match db.get_job_by_id(*id) {
                        Ok(Some(job)) => {
                            if job.schedule_type != "cron" {
                                println!(
                                    "🦑: Job {} is not a cron job (type: {})",
                                    id, job.schedule_type
                                );
                                println!("    Only cron jobs can be manually triggered");
                                return;
                            }

                            println!("🦑: Triggering job {} manually...", id);
                            println!("    Name: {}", job.name);
                            println!(
                                "\n    Note: The job scheduler must be running to execute this job."
                            );
                            println!("          Start the server with: squid serve");
                            println!("\n    The job will be queued and executed by the scheduler.");
                            println!("    Check status with: squid jobs show {}", id);

                            // For now, just update the job to pending status
                            // The actual trigger would need the job scheduler running
                            match db.update_job_status(*id, "pending") {
                                Ok(_) => {
                                    println!("\n✓ Job queued for execution");
                                }
                                Err(e) => {
                                    error!("Failed to queue job: {}", e);
                                    println!("🦑: Failed to queue job - {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            println!("🦑: Job {} not found", id);
                        }
                        Err(e) => {
                            error!("Failed to get job: {}", e);
                            println!("🦑: Failed to get job - {}", e);
                        }
                    }
                }
            }
        }
        Commands::Cleanup => match bundled::cleanup_bundled_assets() {
            Ok(()) => {
                println!("✅ Bundled assets cleaned up successfully");
                println!("✅ Removed extracted plugins and agents from data directory");
            }
            Err(e) => {
                eprintln!("❌ Failed to clean bundled assets: {e}");
                std::process::exit(1);
            }
        },
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
