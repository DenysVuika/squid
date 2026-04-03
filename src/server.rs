use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use log::{error, info, warn};
use rust_embed::RustEmbed;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{api, config, db, rag, session, workspace};

#[derive(RustEmbed)]
#[folder = "static/"]
pub struct Assets;

#[derive(RustEmbed)]
#[folder = "documents/"]
pub struct DemoDocuments;

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

pub async fn start_server(
    port: u16,
    db: Option<PathBuf>,
    dir: Option<PathBuf>,
    mut app_config: config::Config,
) {
    info!("Starting Squid Web UI on port {}", port);

    // CLI --dir parameter overrides config working_dir
    if let Some(work_dir) = dir {
        let work_dir_str = work_dir.to_string_lossy().to_string();
        info!("CLI --dir parameter overrides config working_dir: {}", work_dir_str);
        app_config.working_dir = work_dir_str;
    }

    // Ensure working directory exists and change to it
    let working_dir_path = PathBuf::from(&app_config.working_dir);

    // Create working directory if it doesn't exist
    if !working_dir_path.exists() {
        info!("Creating working directory: {:?}", working_dir_path);
        if let Err(e) = std::fs::create_dir_all(&working_dir_path) {
            error!("Failed to create working directory {:?}: {}", working_dir_path, e);
            println!("🦑: Failed to create working directory {:?} - {}", working_dir_path, e);
            return;
        }
        println!("🦑: Created working directory: {:?}", working_dir_path);
    }

    // Change to working directory
    if working_dir_path != Path::new(".") {
        if let Err(e) = std::env::set_current_dir(&working_dir_path) {
            error!("Failed to change to working directory {:?}: {}", working_dir_path, e);
            println!("🦑: Failed to change to working directory {:?} - {}", working_dir_path, e);
            return;
        }
        info!("Changed working directory to: {:?}", working_dir_path);
        println!("🦑: Working directory set to: {:?}", working_dir_path);
    }

    let bind_address = if app_config.server.allow_network {
        format!("0.0.0.0:{}", port)
    } else {
        format!("127.0.0.1:{}", port)
    };

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
    
    // Initialize plugin system
    info!("Initializing plugin system...");
    match crate::plugins::initialize(Arc::clone(&app_config)) {
        Ok(()) => {
            let plugin_count = crate::plugins::plugin_count();
            info!("Plugin system initialized with {} plugin(s)", plugin_count);
            if plugin_count > 0 {
                println!("🦑: Loaded {} plugin(s)", plugin_count);
            }
        }
        Err(e) => {
            warn!("Failed to initialize plugin system: {}", e);
            println!("🦑: Warning - Plugin system initialization failed: {}", e);
        }
    }

    // Initialize RAG system if enabled
    let rag_system = if app_config.rag.enabled {
        info!("Initializing RAG system...");
        info!("RAG Configuration:");
        info!("  Embedding URL: {}", app_config.rag.embedding_url);
        info!("  Embedding Model: {}", app_config.rag.embedding_model);
        info!("  Documents Path: {}", app_config.rag.documents_path);
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

    // Start document watcher if RAG is enabled
    if let Some(ref rag) = rag_system {
        let documents_path = std::path::PathBuf::from(&app_config.rag.documents_path);
        match rag.create_watcher(documents_path.clone()) {
            Ok(mut watcher) => {
                match watcher.start() {
                    Ok(_) => {
                        info!("Document watcher started for: {}", documents_path.display());
                        println!("🦑: Document watcher active - monitoring {} for changes", documents_path.display());

                        // Spawn background task to process file system events
                        tokio::spawn(async move {
                            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
                            loop {
                                interval.tick().await;
                                if let Err(e) = watcher.process_events().await {
                                    log::error!("Error processing document watcher events: {}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Failed to start document watcher: {}", e);
                        println!("🦑: Document watcher could not be started - {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create document watcher: {}", e);
            }
        }
    }

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
    if app_config.server.allow_network {
        println!("🌐 Server running at: http://{} (accessible from local network)", bind_address);
    } else {
        println!("🌐 Server running at: http://{} (localhost only)", bind_address);
    }
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
                    .route("/agents", web::get().to(api::get_agents))
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
