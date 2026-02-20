use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use log::{debug, error, info, warn};
use rust_embed::RustEmbed;
use std::path::PathBuf;
use std::sync::Arc;

mod api;
mod config;
mod db;
mod envinfo;
mod llm;
mod logger;
mod rag;
mod session;
mod tokens;
mod tools;
mod validate;
mod workspace;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

const SQUIDIGNORE_TEMPLATE: &str = include_str!("../.squidignore.example");

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
        /// API Model (skips interactive prompt if provided)
        #[arg(long)]
        model: Option<String>,
        /// API Key (skips interactive prompt if provided)
        #[arg(long)]
        key: Option<String>,
        /// Context window size in tokens (skips interactive prompt if provided)
        #[arg(long)]
        context_window: Option<u32>,
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
        /// Number of log entries to display
        #[arg(short, long, default_value = "50")]
        limit: usize,
        /// Filter by log level (trace, debug, info, warn, error)
        #[arg(short = 'L', long)]
        level: Option<String>,
        /// Filter by session ID
        #[arg(short, long)]
        session_id: Option<String>,
    },
    /// RAG (Retrieval-Augmented Generation) operations
    Rag {
        #[command(subcommand)]
        command: RagCommands,
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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();

    // Load config early to initialize logger with correct log level
    // For init command, we'll use defaults since config doesn't exist yet
    let app_config = if matches!(cli.command, Commands::Init { .. }) {
        config::Config::default()
    } else {
        config::Config::load()
    };

    // Initialize logger with database support only for serve command
    // Other commands use stdout-only logging
    if matches!(cli.command, Commands::Serve { .. }) {
        let db_path_buf = std::path::PathBuf::from(&app_config.database_path);
        logger::init_with_db(
            Some(&app_config.log_level),
            Some(db_path_buf),
            Some(log::LevelFilter::Info),
        );
    } else {
        logger::init(Some(&app_config.log_level));
    }

    match &cli.command {
        Commands::Init {
            dir,
            url,
            model,
            key: api_key,
            context_window,
            log_level,
        } => {
            info!("Initializing squid configuration in {:?}...", dir);

            // Create directory if it doesn't exist
            if !dir.exists() {
                if let Err(e) = std::fs::create_dir_all(dir) {
                    error!("Failed to create directory {:?}: {}", dir, e);
                    return;
                }
            }

            // Try to load existing config, otherwise use defaults
            let config_path = dir.join("squid.config.json");
            let config_existed = config_path.exists();
            let existing_config = if config_existed {
                println!("Found existing configuration, using current values as defaults...\n");
                match std::fs::read_to_string(&config_path) {
                    Ok(content) => match serde_json::from_str::<config::Config>(&content) {
                        Ok(cfg) => Some(cfg),
                        Err(e) => {
                            info!("Failed to parse existing config: {}", e);
                            None
                        }
                    },
                    Err(e) => {
                        info!("Failed to read existing config: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            let default_config = existing_config.unwrap_or_else(|| config::Config::default());

            // Use CLI args if provided, otherwise prompt interactively
            let final_url = if let Some(u) = url {
                u.clone()
            } else {
                match inquire::Text::new("API URL:")
                    .with_default(&default_config.api_url)
                    .with_help_message(
                        "The base URL for the API (e.g., http://127.0.0.1:1234/v1 for LM Studio)",
                    )
                    .prompt()
                {
                    Ok(u) => u,
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            let final_model = if let Some(m) = model {
                m.clone()
            } else {
                match inquire::Text::new("API Model:")
                    .with_default(&default_config.api_model)
                    .with_help_message("The model identifier to use")
                    .prompt()
                {
                    Ok(m) => m,
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            let final_context_window = if context_window.is_some() {
                context_window.unwrap()
            } else {
                match inquire::Text::new("Context Window (tokens):")
                    .with_default(&default_config.context_window.to_string())
                    .with_help_message(
                        "Max context window size for your model (e.g., 32768 for Qwen2.5-Coder, 128000 for GPT-4)",
                    )
                    .prompt()
                {
                    Ok(ctx_str) => match ctx_str.parse::<u32>() {
                        Ok(ctx) => ctx,
                        Err(_) => {
                            eprintln!("Invalid context window size, using default: {}", default_config.context_window);
                            default_config.context_window
                        }
                    },
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            let final_api_key = if api_key.is_some() {
                api_key.clone()
            } else {
                match inquire::Text::new("API Key (optional, press Enter to skip):")
                    .with_help_message("API key if required (leave empty for local models)")
                    .prompt_skippable()
                {
                    Ok(key) => key.filter(|k| !k.is_empty()),
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            let final_log_level = if let Some(level) = log_level {
                level.clone()
            } else {
                // Find the index of the current log level for the cursor position
                let levels = vec!["error", "warn", "info", "debug", "trace"];
                let cursor_pos = levels
                    .iter()
                    .position(|&l| l == default_config.log_level)
                    .unwrap_or(2);

                match inquire::Select::new("Log Level:", levels)
                    .with_help_message("Logging verbosity (info is recommended)")
                    .with_starting_cursor(cursor_pos)
                    .prompt()
                {
                    Ok(level) => level.to_string(),
                    Err(_) => {
                        error!("Configuration initialization cancelled or failed");
                        return;
                    }
                }
            };

            // Smart merge permissions: keep user's custom permissions + add new defaults
            let (merged_permissions, old_permissions) = if config_existed {
                let old_perms = default_config.permissions.clone();
                let allow_set: std::collections::HashSet<String> =
                    default_config.permissions.allow.iter().cloned().collect();
                let deny_set: std::collections::HashSet<String> =
                    default_config.permissions.deny.iter().cloned().collect();

                // Add new default permissions that user doesn't have
                let new_defaults = config::Permissions::default();
                let mut merged_allow = allow_set.clone();
                for default_perm in &new_defaults.allow {
                    if !allow_set.contains(default_perm) && !deny_set.contains(default_perm) {
                        merged_allow.insert(default_perm.clone());
                    }
                }

                (
                    config::Permissions {
                        allow: merged_allow.into_iter().collect(),
                        deny: deny_set.into_iter().collect(),
                    },
                    Some(old_perms),
                )
            } else {
                (default_config.permissions, None)
            };

            let config = config::Config {
                api_url: final_url,
                api_model: final_model,
                api_key: final_api_key,
                context_window: final_context_window,
                log_level: final_log_level,
                permissions: merged_permissions,
                version: None, // Will be set automatically by save_to_dir()
                database_path: config::Config::default().database_path,
                enable_env_context: config::Config::default().enable_env_context,
                rag: config::Config::default().rag,
            };

            match config.save_to_dir(dir) {
                Ok(_) => {
                    let config_path = dir.join("squid.config.json");
                    info!("✓ Configuration saved to {:?}", config_path);
                    println!("\nConfiguration saved to: {:?}", config_path);
                    println!("  API URL: {}", config.api_url);
                    println!("  API Model: {}", config.api_model);
                    if config.api_key.is_some() {
                        println!("  API Key: [configured]");
                    } else {
                        println!("  API Key: [not set]");
                    }
                    println!("  Context Window: {} tokens", config.context_window);
                    println!("  Log Level: {}", config.log_level);

                    // Show info about permissions
                    if let Some(old_perms) = old_permissions {
                        // Check if new permissions were added
                        let old_allow_set: std::collections::HashSet<String> =
                            old_perms.allow.iter().cloned().collect();
                        let new_allow_set: std::collections::HashSet<String> =
                            config.permissions.allow.iter().cloned().collect();
                        let added_perms: Vec<_> =
                            new_allow_set.difference(&old_allow_set).collect();

                        if !added_perms.is_empty() {
                            println!("\n✓ Added new default permissions: {:?}", added_perms);
                        }

                        if !config.permissions.allow.is_empty()
                            || !config.permissions.deny.is_empty()
                        {
                            println!("\n✓ Current tool permissions:");
                            if !config.permissions.allow.is_empty() {
                                println!("  Allowed: {:?}", config.permissions.allow);
                            }
                            if !config.permissions.deny.is_empty() {
                                println!("  Denied: {:?}", config.permissions.deny);
                            }
                        }
                    } else {
                        println!("\n✓ Default permissions configured");
                        if !config.permissions.allow.is_empty() {
                            println!("  Allowed: {:?}", config.permissions.allow);
                        }
                    }

                    // Create .squidignore file if it doesn't exist
                    let squidignore_path = dir.join(".squidignore");
                    if !squidignore_path.exists() {
                        match std::fs::write(&squidignore_path, SQUIDIGNORE_TEMPLATE) {
                            Ok(_) => {
                                info!("✓ Created .squidignore file at {:?}", squidignore_path);
                                println!("\n✓ Created .squidignore with default patterns");
                                println!(
                                    "  Edit this file to customize which files squid should ignore"
                                );
                            }
                            Err(e) => {
                                warn!("Failed to create .squidignore: {}", e);
                                println!("\n⚠ Could not create .squidignore: {}", e);
                            }
                        }
                    } else {
                        info!(".squidignore already exists, skipping creation");
                        println!("\n✓ Using existing .squidignore file");
                    }
                }
                Err(e) => {
                    error!("Failed to save configuration: {}", e);
                }
            }
        }
        Commands::Ask {
            question,
            message,
            no_stream,
            file,
            prompt,
        } => {
            let full_question = if let Some(m) = message {
                format!("{} {}", question, m)
            } else {
                question.clone()
            };

            info!("Q: {}", full_question);

            let file_content = if let Some(file_path) = file {
                // Validate path before reading
                let ignore_patterns = validate::PathValidator::load_ignore_patterns();
                let validator = validate::PathValidator::with_ignore_file(Some(ignore_patterns));

                match validator.validate(file_path) {
                    Ok(_) => match std::fs::read_to_string(file_path) {
                        Ok(content) => {
                            info!("Read file content ({} bytes)", content.len());
                            Some(content)
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::NotFound {
                                println!(
                                    "🦑: I can't find that file. Please check the path and try again."
                                );
                            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                                println!("🦑: I don't have permission to read that file.");
                            } else {
                                println!("🦑: I couldn't read that file - {}", e);
                            }
                            debug!("Failed to read file {}: {}", file_path.display(), e);
                            return;
                        }
                    },
                    Err(validate::PathValidationError::PathIgnored(_)) => {
                        println!("🦑: I can't access that file - it's in your .squidignore list.");
                        return;
                    }
                    Err(validate::PathValidationError::PathNotAllowed(_)) => {
                        println!(
                            "🦑: I can't access that file - it's outside the project directory or in a protected system location."
                        );
                        return;
                    }
                    Err(e) => {
                        debug!("Path validation failed: {}", e);
                        println!("🦑: I can't access that file - {}", e);
                        return;
                    }
                }
            } else {
                None
            };

            let custom_prompt = if let Some(prompt_path) = prompt {
                match std::fs::read_to_string(prompt_path) {
                    Ok(content) => {
                        info!("Using custom system prompt ({} bytes)", content.len());
                        Some(content)
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            println!(
                                "🦑: I can't find that custom prompt file. Please check the path and try again."
                            );
                        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                            println!("🦑: I don't have permission to read that prompt file.");
                        } else {
                            println!("🦑: I couldn't read that prompt file - {}", e);
                        }
                        debug!(
                            "Failed to read custom prompt file {}: {}",
                            prompt_path.display(),
                            e
                        );
                        return;
                    }
                }
            } else {
                None
            };

            if *no_stream {
                match llm::ask_llm(
                    &full_question,
                    file_content.as_deref(),
                    file.as_ref().and_then(|p| p.to_str()),
                    custom_prompt.as_deref(),
                    &app_config,
                )
                .await
                {
                    Ok(response) => {
                        println!("\n🦑: {}", response);
                    }
                    Err(e) => {
                        error!("Failed to get response: {}", e);
                    }
                }
            } else {
                if let Err(e) = llm::ask_llm_streaming(
                    &full_question,
                    file_content.as_deref(),
                    file.as_ref().and_then(|p| p.to_str()),
                    custom_prompt.as_deref(),
                    &app_config,
                )
                .await
                {
                    error!("Failed to get response: {}", e);
                }
            }
        }
        Commands::Review {
            file,
            message,
            no_stream,
        } => {
            info!("Reviewing file: {:?}", file);

            // Validate path before reading
            let ignore_patterns = validate::PathValidator::load_ignore_patterns();
            let validator = validate::PathValidator::with_ignore_file(Some(ignore_patterns));

            let file_content = match validator.validate(file) {
                Ok(_) => match std::fs::read_to_string(file) {
                    Ok(content) => {
                        info!("Read file content ({} bytes)", content.len());
                        content
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            println!(
                                "🦑: I can't find that file. Please check the path and try again."
                            );
                        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                            println!("🦑: I don't have permission to read that file.");
                        } else {
                            println!("🦑: I couldn't read that file - {}", e);
                        }
                        debug!("Failed to read file {}: {}", file.display(), e);
                        return;
                    }
                },
                Err(validate::PathValidationError::PathIgnored(_)) => {
                    println!("🦑: I can't access that file - it's in your .squidignore list.");
                    return;
                }
                Err(validate::PathValidationError::PathNotAllowed(_)) => {
                    println!(
                        "🦑: I can't access that file - it's outside the project directory or in a protected system location."
                    );
                    return;
                }
                Err(e) => {
                    debug!("Path validation failed: {}", e);
                    println!("🦑: I can't access that file - {}", e);
                    return;
                }
            };

            let review_prompt = llm::get_review_prompt_for_file(file);
            let combined_review_prompt = llm::combine_prompts(review_prompt);
            debug!("Using review prompt for file type");

            let question = if let Some(msg) = message {
                format!("Please review this code. {}", msg)
            } else {
                "Please review this code.".to_string()
            };

            if *no_stream {
                match llm::ask_llm(
                    &question,
                    Some(&file_content),
                    file.to_str(),
                    Some(&combined_review_prompt),
                    &app_config,
                )
                .await
                {
                    Ok(response) => {
                        println!("\n🦑: {}", response);
                    }
                    Err(e) => {
                        error!("Failed to get review: {}", e);
                    }
                }
            } else {
                if let Err(e) = llm::ask_llm_streaming(
                    &question,
                    Some(&file_content),
                    file.to_str(),
                    Some(&combined_review_prompt),
                    &app_config,
                )
                .await
                {
                    error!("Failed to get review: {}", e);
                }
            }
        }
        Commands::Serve { port, db, dir } => {
            info!("Starting Squid Web UI on port {}", port);

            // Change working directory if specified
            if let Some(work_dir) = dir {
                if let Err(e) = std::env::set_current_dir(work_dir) {
                    error!("Failed to change to directory {:?}: {}", work_dir, e);
                    println!("🦑: Failed to change to directory {:?} - {}", work_dir, e);
                    return;
                }
                info!("Changed working directory to: {:?}", work_dir);
                println!("🦑: Working directory set to: {:?}", work_dir);
            }

            let bind_address = format!("127.0.0.1:{}", port);
            let mut app_config = app_config.clone();
            
            // Override database path if specified via CLI
            if let Some(db_path) = db {
                let db_path_str = db_path.to_string_lossy().to_string();
                info!("Using custom database path: {}", db_path_str);
                app_config.database_path = db_path_str;
            }
            
            let app_config = Arc::new(app_config);

            // Initialize database
            let db_path = &app_config.database_path;
            info!("Initializing database at: {}", db_path);
            let database = match db::Database::new(db_path) {
                Ok(db) => {
                    info!("Database initialized successfully");
                    db
                }
                Err(e) => {
                    error!("Failed to initialize database: {}", e);
                    println!("🦑: Failed to initialize database - {}", e);
                    println!("    Database path: {}", db_path);
                    println!("    Make sure the directory is writable and the database file is not corrupted.");
                    return;
                }
            };

            let session_manager = Arc::new(session::SessionManager::new(database));
            
            // Initialize RAG system if enabled
            let rag_system = if app_config.rag.enabled {
                info!("Initializing RAG system...");
                match db::Database::new(db_path) {
                    Ok(db) => {
                        match rag::RagSystem::new(Arc::new(db), &app_config.rag).await {
                            Ok(system) => {
                                info!("RAG system initialized successfully");
                                Some(Arc::new(system))
                            }
                            Err(e) => {
                                warn!("Failed to initialize RAG system: {}", e);
                                println!("🦑: RAG initialization failed - {}", e);
                                println!("    RAG features will be disabled");
                                None
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to open database for RAG: {}", e);
                        println!("🦑: RAG initialization failed - {}", e);
                        None
                    }
                }
            } else {
                info!("RAG is disabled in configuration");
                None
            };
            
            // Create approval state map for tool approval workflow
            let approval_map: api::ApprovalStateMap = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

            // Spawn approval cleanup task to remove expired approvals
            let approval_map_cleanup = approval_map.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    let mut approvals = approval_map_cleanup.lock().await;
                    let now = std::time::Instant::now();
                    let initial_count = approvals.len();
                    approvals.retain(|_, state| {
                        now.duration_since(state.created_at).as_secs() < 300
                    });
                    let removed = initial_count - approvals.len();
                    if removed > 0 {
                        log::debug!("Cleaned up {} expired approval(s), {} remaining", removed, approvals.len());
                    }
                }
            });

            println!("🦑: Starting Squid Web UI...");
            println!("🌐 Server running at: http://{}", bind_address);
            println!("📡 API endpoint: http://{}/api/chat", bind_address);
            println!("Press Ctrl+C to stop the server\n");

            let server = HttpServer::new(move || {
                // Configure CORS to allow development mode (Vite dev server)
                let cors = Cors::default()
                    .allow_any_origin() // Allow all origins (for development)
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600);

                App::new()
                    .app_data(web::Data::new(app_config.clone()))
                    .app_data(web::Data::new(session_manager.clone()))
                    .app_data(web::Data::new(approval_map.clone()))
                    .app_data(web::Data::new(rag_system.clone()))
                    .wrap(cors)
                    .wrap(middleware::Logger::default())
                    .service(
                        web::scope("/api")
                            .route("/chat", web::post().to(api::chat_stream))
                            .route("/sessions", web::get().to(api::list_sessions))
                            .route("/sessions/{session_id}", web::get().to(api::get_session))
                            .route("/sessions/{session_id}", web::patch().to(api::update_session))
                            .route("/sessions/{session_id}", web::delete().to(api::delete_session))
                            .route("/logs", web::get().to(api::get_logs))
                            .route("/models", web::get().to(api::get_models))
                            .route("/config", web::get().to(api::get_config))
                            .route("/tool-approval", web::post().to(api::handle_tool_approval))
                            .route("/workspace/files", web::get().to(workspace::get_workspace_files))
                            .route("/workspace/files/{path:.*}", web::get().to(workspace::get_workspace_file))
                            .route("/rag/query", web::post().to(api::rag_query))
                            .route("/rag/documents", web::get().to(api::rag_list_documents))
                            .route("/rag/documents/{filename:.*}", web::delete().to(api::rag_delete_document))
                            .route("/rag/stats", web::get().to(api::rag_stats))
                            .route("/rag/upload", web::post().to(api::rag_upload_document))
                    )
                    .route("/", web::get().to(serve_index))
                    .route("/{filename:.*}", web::get().to(serve_static))
            })
            .bind(&bind_address);

            match server {
                Ok(server) => {
                    if let Err(e) = server.run().await {
                        error!("Server error: {}", e);
                        println!("🦑: Server error - {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to bind to {}: {}", bind_address, e);
                    println!("🦑: Failed to start server on {} - {}", bind_address, e);
                    println!(
                        "The port might already be in use. Try a different port with --port <PORT>"
                    );
                }
            }
        }
        Commands::Logs {
            limit,
            level,
            session_id,
        } => {
            let db_path = &app_config.database_path;

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
                            let timestamp = chrono::DateTime::from_timestamp(log.timestamp, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            let session_info = log
                                .session_id
                                .map(|sid| format!(" [session: {}]", &sid[..8]))
                                .unwrap_or_default();

                            println!(
                                "[{}] {} {}{}: {}",
                                timestamp, log.level.to_uppercase(), log.target, session_info, log.message
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
                println!("    Set 'rag.enabled = true' in squid.config.json to enable RAG features");
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
                        println!("🦑: Documents directory not found: {}", documents_path.display());
                        println!("    Create the directory and add documents to index");
                        return;
                    }

                    println!("🦑: Scanning documents directory: {}", documents_path.display());

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
                RagCommands::List => {
                    match rag_system.indexer.list_documents() {
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
                    }
                }
                RagCommands::Rebuild { dir } => {
                    let documents_path = dir
                        .clone()
                        .unwrap_or_else(|| PathBuf::from(&rag_config.documents_path));

                    if !documents_path.exists() {
                        println!("🦑: Documents directory not found: {}", documents_path.display());
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
                RagCommands::Stats => {
                    match rag_system.indexer.get_stats() {
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
                    }
                }
            }
        }
    }
}

async fn serve_index() -> HttpResponse {
    serve_static(web::Path::from("index.html".to_string())).await
}

async fn serve_static(path: web::Path<String>) -> HttpResponse {
    let path = path.into_inner();
    let path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path.as_str()
    };

    match Assets::get(path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(path).first_or_octet_stream();

            HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(content.data.into_owned())
        }
        None => HttpResponse::NotFound().body("404 - Not Found"),
    }
}
