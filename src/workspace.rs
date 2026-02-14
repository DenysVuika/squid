use actix_web::{web, HttpResponse, Error};
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileNode>>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceFilesResponse {
    pub files: Vec<FileNode>,
}

/// Get workspace files structure
pub async fn get_workspace_files() -> Result<HttpResponse, Error> {
    debug!("Fetching workspace files");

    // Get current working directory
    let cwd = std::env::current_dir().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to get current directory: {}", e))
    })?;

    // Build file tree
    let files = build_file_tree(&cwd).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to build file tree: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(WorkspaceFilesResponse { files }))
}

/// Check if a file is a supported code/text file based on extension
fn is_supported_file(path: &std::path::Path) -> bool {
    // Extensions to include (code and documentation files)
    let code_extensions = vec![
        "rs", "toml", "lock", "json", "js", "jsx", "ts", "tsx", "css", "scss", "html",
        "md", "txt", "yaml", "yml", "sh", "py", "go", "java", "c", "cpp", "h", "hpp",
        "vue", "svelte", "rb", "php", "swift", "kt", "sql", "graphql", "proto",
    ];

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Check extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        code_extensions.contains(&ext)
    } else {
        // Files without extension - only include certain names
        let allowed_no_ext = vec!["Dockerfile", "Makefile", "README", "LICENSE"];
        allowed_no_ext.iter().any(|&name| file_name.starts_with(name))
    }
}

/// Get content of a single workspace file
pub async fn get_workspace_file(path: web::Path<String>) -> Result<HttpResponse, Error> {
    use std::path::Path;
    
    let file_path = path.into_inner();
    debug!("Fetching workspace file: {}", file_path);

    // Get current working directory
    let cwd = std::env::current_dir().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to get current directory: {}", e))
    })?;

    // Construct full path
    let full_path = cwd.join(&file_path);

    // Security check: ensure the resolved path is within the workspace
    let canonical_path = full_path.canonicalize().map_err(|e| {
        actix_web::error::ErrorNotFound(format!("File not found: {}", e))
    })?;

    if !canonical_path.starts_with(&cwd) {
        return Err(actix_web::error::ErrorForbidden("Access denied: Path is outside workspace"));
    }

    // Check if path is a file
    if !canonical_path.is_file() {
        return Err(actix_web::error::ErrorBadRequest("Path is not a file"));
    }

    // Check if file type is supported
    if !is_supported_file(&canonical_path) {
        return Err(actix_web::error::ErrorBadRequest("File type not supported for viewing"));
    }

    // Read file content
    let content = std::fs::read_to_string(&canonical_path).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to read file: {}", e))
    })?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(content))
}

/// Build a hierarchical file tree for a directory
fn build_file_tree(root_path: &std::path::Path) -> Result<Vec<FileNode>, Box<dyn std::error::Error>> {
    use walkdir::WalkDir;
    use std::collections::HashMap;

    // Directories to exclude
    let excluded_dirs = vec![
        "node_modules", "target", "dist", "build", ".git", ".next", ".nuxt",
        "vendor", "venv", ".venv", "env", ".env", "__pycache__", ".pytest_cache",
        "coverage", ".nyc_output", "tmp", "temp", ".cache",
    ];

    // Files to exclude
    let excluded_files = vec![
        ".DS_Store", "Thumbs.db", ".gitignore", ".gitattributes", "package-lock.json",
        "yarn.lock", "pnpm-lock.yaml", "Cargo.lock",
    ];

    // Store all entries with their metadata
    let mut entries: Vec<(std::path::PathBuf, bool)> = Vec::new();

    // Walk directory
    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            
            // Skip hidden files/directories (starting with .)
            if file_name.starts_with('.') {
                return false;
            }
            
            // Skip excluded directories
            if e.file_type().is_dir() {
                !excluded_dirs.iter().any(|&excluded| file_name == excluded)
            } else {
                true
            }
        })
    {
        let entry = entry?;
        let path = entry.path();
        
        // Skip root itself
        if path == root_path {
            continue;
        }

        let is_dir = entry.file_type().is_dir();
        
        // For files, check if they should be included
        if !is_dir {
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Skip excluded files
            if excluded_files.contains(&file_name) {
                continue;
            }

            // Check if file is supported
            if !is_supported_file(path) {
                continue;
            }
        }

        entries.push((path.to_path_buf(), is_dir));
    }

    // Build hierarchical structure
    let mut tree: HashMap<std::path::PathBuf, FileNode> = HashMap::new();
    let mut root_nodes: Vec<FileNode> = Vec::new();

    // Sort entries by depth (shallowest first) to ensure parents are processed before children
    entries.sort_by_key(|(path, _)| path.components().count());

    // First pass: create all nodes and identify root nodes
    for (path, is_dir) in &entries {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        let relative_path = path.strip_prefix(root_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let node = FileNode {
            name,
            path: relative_path.clone(),
            is_dir: *is_dir,
            children: if *is_dir { Some(Vec::new()) } else { None },
        };

        // Check if this is a direct child of root
        if path.parent() == Some(root_path) {
            root_nodes.push(node.clone());
        }

        tree.insert(path.clone(), node);
    }

    // Second pass: link children to parents
    for (path, _is_dir) in &entries {
        if let Some(parent_path) = path.parent() {
            if parent_path != root_path {
                if let Some(child_node) = tree.get(path).cloned() {
                    if let Some(parent_node) = tree.get_mut(parent_path) {
                        if let Some(ref mut children) = parent_node.children {
                            children.push(child_node);
                        }
                    }
                }
            }
        }
    }

    // Update root nodes with their populated children
    for root_node in &mut root_nodes {
        let full_path = root_path.join(&root_node.path);
        if let Some(updated_node) = tree.get(&full_path) {
            if let Some(ref children) = updated_node.children {
                root_node.children = Some(children.clone());
            }
        }
    }

    // Sort nodes: directories first, then alphabetically
    root_nodes.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    // Sort children recursively
    fn sort_children(node: &mut FileNode) {
        if let Some(ref mut children) = node.children {
            children.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });
            for child in children.iter_mut() {
                sort_children(child);
            }
        }
    }

    for node in &mut root_nodes {
        sort_children(node);
    }

    // Remove empty directories recursively
    fn remove_empty_dirs(nodes: &mut Vec<FileNode>) {
        nodes.retain_mut(|node| {
            if let Some(ref mut children) = node.children {
                remove_empty_dirs(children);
                // Keep the directory if it has children, or if it's a file
                !children.is_empty() || !node.is_dir
            } else {
                // Keep files (they don't have children)
                true
            }
        });
    }

    remove_empty_dirs(&mut root_nodes);

    Ok(root_nodes)
}
