use log::{error, info, warn};
use std::path::PathBuf;

const SQUIDIGNORE_TEMPLATE: &str = include_str!("../.squidignore.template");

pub async fn run(
    dir: &PathBuf,
    url: &Option<String>,
    api_key: &Option<String>,
    log_level: &Option<String>,
) {
    info!("Initializing squid configuration in {:?}...", dir);

    // Create directory if it doesn't exist
    if !dir.exists()
        && let Err(e) = std::fs::create_dir_all(dir)
    {
        error!("Failed to create directory {:?}: {}", dir, e);
        return;
    }

    // Try to load existing config, otherwise use defaults
    let config_path = dir.join("squid.config.json");
    let config_existed = config_path.exists();
    let existing_config = if config_existed {
        println!("Found existing configuration, using current values as defaults...\n");
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<crate::config::Config>(&content) {
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

    let default_config = existing_config.unwrap_or_else(crate::config::Config::default);

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

    // Use default context window (32768) for agents
    let _final_context_window = 32768u32;

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

    // Ask about RAG setup
    let enable_rag = match inquire::Confirm::new("Enable RAG (Retrieval-Augmented Generation)?")
        .with_default(default_config.rag.enabled)
        .with_help_message("RAG allows the AI to use external documents for context")
        .prompt()
    {
        Ok(enabled) => enabled,
        Err(_) => {
            error!("Configuration initialization cancelled or failed");
            return;
        }
    };

    let final_rag_config = if enable_rag {
        // Prompt for RAG-specific settings
        // Default embedding URL should match the API URL for services like LM Studio
        let default_embedding_url = if default_config.rag.enabled {
            // If updating existing config, use existing embedding URL
            default_config.rag.embedding_url.clone()
        } else {
            // For new configs, suggest the same URL as API URL but without /v1 suffix
            // LM Studio uses: http://host:port/v1 for API and http://host:port for embeddings
            final_url
                .strip_suffix("/v1")
                .unwrap_or(&final_url)
                .to_string()
        };

        let embedding_url = match inquire::Text::new("Embedding API URL:")
            .with_default(&default_embedding_url)
            .with_help_message("URL for the embedding service (for LM Studio use http://127.0.0.1:1234, for Ollama use http://127.0.0.1:11434)")
            .prompt()
        {
            Ok(url) => url,
            Err(_) => {
                error!("Configuration initialization cancelled or failed");
                return;
            }
        };

        let embedding_model = match inquire::Text::new("Embedding Model:")
            .with_default(&default_config.rag.embedding_model)
            .with_help_message(
                "Model name for embeddings (e.g., text-embedding-nomic-embed-text-v1.5)",
            )
            .prompt()
        {
            Ok(model) => model,
            Err(_) => {
                error!("Configuration initialization cancelled or failed");
                return;
            }
        };

        let documents_path = match inquire::Text::new("Documents Directory:")
            .with_default(&default_config.rag.documents_path)
            .with_help_message("Path where RAG documents will be stored (relative to project root)")
            .prompt()
        {
            Ok(path) => path,
            Err(_) => {
                error!("Configuration initialization cancelled or failed");
                return;
            }
        };

        crate::config::RagConfig {
            enabled: true,
            embedding_url,
            embedding_model,
            documents_path,
            chunk_size: default_config.rag.chunk_size,
            chunk_overlap: default_config.rag.chunk_overlap,
            top_k: default_config.rag.top_k,
        }
    } else {
        crate::config::RagConfig {
            enabled: false,
            ..default_config.rag
        }
    };

    // Ask about setting up demo documents
    let setup_demo_docs = if enable_rag {
        match inquire::Confirm::new("Setup demo documents for RAG?")
            .with_default(true)
            .with_help_message(
                "Creates sample documents in the documents directory to get started with RAG",
            )
            .prompt()
        {
            Ok(setup) => setup,
            Err(_) => {
                error!("Configuration initialization cancelled or failed");
                return;
            }
        }
    } else {
        false
    };

    // Create agents directory and default agent files
    let agents_dir = dir.join("agents");
    if !agents_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&agents_dir) {
            warn!("Failed to create agents directory: {}", e);
        } else {
            info!("Created agents directory at {:?}", agents_dir);
            // Create default agent files
            create_default_agent_files(&agents_dir);
        }
    } else {
        info!("Agents directory already exists, skipping creation");
        println!("\n✓ Using existing agents directory");
    }

    let config = crate::config::Config {
        api_url: final_url,
        api_key: final_api_key,
        context_window: 32768, // Global default fallback
        log_level: final_log_level,
        db_log_level: crate::config::Config::default().db_log_level,
        version: None, // Will be set automatically by save_to_dir()
        database_path: crate::config::Config::default().database_path,
        working_dir: crate::config::Config::default().working_dir,
        rag: final_rag_config,
        plugins: crate::config::PluginsConfig::default(),
        server: crate::config::Config::default().server,
        web: crate::config::WebConfig::default(),
        jobs: crate::config::JobsConfig::default(),
        default_agent: "general-assistant".to_string(),
        agents: crate::agent::AgentsConfig::default(),
        config_dir: Some(dir.clone()),
    };

    match config.save_to_dir(dir) {
        Ok(_) => {
            let config_path = dir.join("squid.config.json");
            info!("✓ Configuration saved to {:?}", config_path);
            println!("\n✅ Configuration saved to: {:?}", config_path);
            println!("\nSettings:");
            println!("  API URL: {}", config.api_url);
            if config.api_key.is_some() {
                println!("  API Key: [configured]");
            } else {
                println!("  API Key: [not set]");
            }
            println!("  Context Window: {} tokens", config.context_window);
            println!("  Log Level: {}", config.log_level);
            println!(
                "  RAG Enabled: {}",
                if config.rag.enabled { "yes" } else { "no" }
            );

            println!("\nDefault agents available (in agents/ folder):");
            println!("  • general-assistant (default) - Full-featured coding assistant");
            println!("  • code-reviewer - Read-only code review specialist");
            println!("  • light - Lightweight assistant with minimal permissions");
            println!("  • pirate (Captain Squidbeard) - Pirate-themed demo agent");
            println!("  • shakespeare - Shakespearean English assistant");
            println!("\n  Edit agents/*.md files to customize or add new agents.");

            println!("\nNext steps:");
            println!("  1. Start the server: squid serve");
            println!("  2. Or use CLI: squid ask \"your question\"");
            println!("  3. Open Web UI: http://localhost:3000");
            if config.rag.enabled {
                println!("    Embedding URL: {}", config.rag.embedding_url);
                println!("    Embedding Model: {}", config.rag.embedding_model);
                println!("    Documents path: {}", config.rag.documents_path);
            }

            // Create .squidignore file if it doesn't exist
            let squidignore_path = dir.join(".squidignore");
            if !squidignore_path.exists() {
                match std::fs::write(&squidignore_path, SQUIDIGNORE_TEMPLATE) {
                    Ok(_) => {
                        info!("✓ Created .squidignore file at {:?}", squidignore_path);
                        println!("\n✓ Created .squidignore with default patterns");
                        println!("  Edit this file to customize which files squid should ignore");
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

            // Setup demo documents if requested
            if setup_demo_docs {
                let docs_dir = dir.join(&config.rag.documents_path);

                // Create documents directory if it doesn't exist
                if let Err(e) = std::fs::create_dir_all(&docs_dir) {
                    warn!("Failed to create documents directory: {}", e);
                    println!("\n⚠ Could not create documents directory: {}", e);
                } else {
                    info!("Created documents directory at {:?}", docs_dir);

                    let mut success_count = 0;
                    let mut fail_count = 0;

                    // Extract all embedded demo documents
                    for filename in crate::server::DemoDocuments::iter() {
                        let file_path = docs_dir.join(filename.as_ref());

                        // Skip if file already exists
                        if file_path.exists() {
                            info!("Skipping existing file: {:?}", file_path);
                            continue;
                        }

                        if let Some(content) = crate::server::DemoDocuments::get(filename.as_ref())
                        {
                            match std::fs::write(&file_path, content.data.as_ref()) {
                                Ok(_) => {
                                    info!("Created demo document: {:?}", file_path);
                                    success_count += 1;
                                }
                                Err(e) => {
                                    warn!("Failed to write {}: {}", filename, e);
                                    fail_count += 1;
                                }
                            }
                        } else {
                            warn!("Could not read embedded file: {}", filename);
                            fail_count += 1;
                        }
                    }

                    if success_count > 0 {
                        println!(
                            "\n✓ Created {} demo document(s) in {:?}",
                            success_count, docs_dir
                        );
                        println!("  Run 'squid rag init' to index these documents for RAG");
                    }
                    if fail_count > 0 {
                        println!("⚠ Failed to create {} document(s)", fail_count);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to save configuration: {}", e);
        }
    }
}

/// Create default agent files in the agents directory
fn create_default_agent_files(agents_dir: &std::path::Path) {
    let agents = vec![
        (
            "general-assistant.md",
            r#"---
name: General Assistant
enabled: true
description: Full-featured coding assistant with all tools
model: qwen3.5-4b
context_window: 32768
pricing_model: gpt-4o-mini
permissions:
  - now
  - read_file
  - write_file
  - grep
  - bash:ls
  - bash:pwd
  - bash:git
  - bash:cat
  - plugin:*
suggestions:
  - Read and summarize the main source files in this project
  - Show me the recent git log
  - Find all TODO comments in the codebase
  - List all files in the current directory
---
You are a helpful AI coding assistant with expertise in software development, code review, and best practices.
"#,
        ),
        (
            "code-reviewer.md",
            r#"---
name: Code Reviewer
enabled: true
description: Reviews code for quality and security (read-only)
model: qwen3.5-4b
context_window: 32768
pricing_model: gpt-4o-mini
permissions:
  - now
  - read_file
  - grep
suggestions:
  - Review this file for security vulnerabilities
  - What are the biggest code quality issues here?
  - Identify any performance bottlenecks
  - Check this code for common anti-patterns
---
You are an expert code reviewer. Focus on security vulnerabilities, performance issues, code quality, and maintainability. Provide constructive feedback with specific examples.
"#,
        ),
        (
            "light.md",
            r#"---
name: Light
enabled: true
description: Lightweight assistant with minimal permissions
model: gemma-4-e2b-it
context_window: 8192
pricing_model: google/gemma-3
permissions:
  - now
suggestions:
  - What time is it?
  - Give me a quick summary of what you can do
  - Explain recursion in simple terms
  - What's a good way to stay productive?
---
You are a lightweight AI assistant designed for quick questions and casual interactions.
"#,
        ),
        (
            "pirate.md",
            r#"---
name: Captain Squidbeard
enabled: true
description: A swashbuckling pirate assistant (demo of fully custom prompt)
model: qwen3.5-4b
context_window: 8192
pricing_model: gpt-4o-mini
permissions:
  - now
suggestions:
  - What be the time, Captain?
  - Tell me a tale of the seven seas!
  - How do I navigate this codebase, matey?
  - What treasure lies hidden in these files, arr?
---
Ye be Captain Squidbeard 🏴‍☠️, a cunning pirate squid sailin' the seven seas of code! Speak like a proper pirate in all yer responses - use 'arr', 'matey', 'ye', 'aye', and other pirate lingo. Be helpful but keep that salty sea dog personality. When asked fer the date or time, use the now tool, or respond with the info from yer ship's log: Date: {{date}}, Time: {{time}}, Timezone: {{timezone}}. Keep yer answers brief unless the scallywag asks fer more detail!
"#,
        ),
        (
            "shakespeare.md",
            r#"---
name: William Shakespeare
enabled: true
description: A renaissance bard who speaks in Shakespearean English (no tools)
model: qwen3.5-4b
context_window: 8192
pricing_model: gpt-4o-mini
use_tools: false
permissions: []
suggestions:
  - Compose a sonnet about the art of coding
  - Explain what a function is in thy poetic tongue
  - What is the nature of a bug, good Bard?
  - Speak to me of the virtues of clean code
---
Thou art William Shakespeare, the immortal Bard of Avon ✍️. Speak always in the eloquent style of the Elizabethan age — employ 'thee', 'thou', 'thy', 'dost', 'hath', 'wherefore', 'forsooth', and 'prithee' as befitteth a poet of the Globe Theatre. Be helpful and wise, yet never abandon thine poetic tongue. Keep thy answers brief and elegant unless the questioner doth seek greater depth.
"#,
        ),
    ];

    let mut success_count = 0;
    let mut fail_count = 0;

    for (filename, content) in agents {
        let file_path = agents_dir.join(filename);
        if file_path.exists() {
            info!("Skipping existing agent file: {:?}", file_path);
            continue;
        }

        match std::fs::write(&file_path, content) {
            Ok(_) => {
                info!("Created default agent file: {}", filename);
                success_count += 1;
            }
            Err(e) => {
                warn!("Failed to write {}: {}", filename, e);
                fail_count += 1;
            }
        }
    }

    if success_count > 0 {
        println!("\n✓ Created {} default agent(s) in agents/", success_count);
    }
    if fail_count > 0 {
        println!("⚠ Failed to create {} agent file(s)", fail_count);
    }
}
