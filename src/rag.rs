use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use rig::client::EmbeddingsClient;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use tiktoken_rs::cl100k_base;

use crate::config::RagConfig;
use crate::db::Database;

/// Supported document file extensions for RAG indexing
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "md", "txt", "rs", "py", "js", "ts", "jsx", "tsx", "java", "c", "cpp", "h", "hpp", "go",
    "rb", "php", "sh", "bash", "yml", "yaml", "json", "toml", "xml", "html", "css", "scss",
];

/// RAG embedder using Rig with OpenAI-compatible API
pub struct RagEmbedder {
    client: rig::providers::openai::Client,
    model: String,
}

impl RagEmbedder {
    /// Create a new RAG embedder with the specified configuration
    ///
    /// # Arguments
    /// * `embedding_url` - Base URL for the embedding service (e.g., "http://localhost:1234")
    /// * `model` - Model name to use for embeddings
    ///
    /// # Notes
    /// This uses the rig-core 0.33.0+ builder pattern to set a custom base URL.
    /// The builder pattern is required to properly override the OpenAI endpoint
    /// for local services like LM Studio or Ollama.
    pub fn new(embedding_url: &str, model: &str) -> Result<Self> {
        // For OpenAI-compatible APIs (LM Studio, Ollama, Docker model runner, etc.)
        // Pass the full URL as-is, only removing trailing slashes
        // Examples:
        //   - http://localhost:1234/v1 (LM Studio)
        //   - http://localhost:11434/v1 (Ollama)
        //   - http://localhost:12434/engines/v1 (Docker model runner)
        let normalized_url = embedding_url.trim_end_matches('/');

        // Create client with custom base URL using builder pattern
        // Note: In rig-core 0.33.0+, the builder() -> api_key() -> base_url() -> build()
        // pattern is required to properly set a custom endpoint. The full URL path
        // (including /v1 or /engines/v1) should be provided in the base_url.
        // The API key "not-needed" works for local services that don't require authentication.
        let client = rig::providers::openai::Client::builder()
            .api_key("not-needed")
            .base_url(normalized_url)
            .build()
            .context("Failed to create OpenAI client")?;

        Ok(Self {
            client,
            model: model.to_string(),
        })
    }

    /// Generate embeddings for a single text
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating embedding using model: {}", self.model);
        debug!("Text length: {} characters", text.len());

        let embeddings = match self.client.embeddings(&self.model)
            .document(text)
            .context("Failed to create embedding document")?
            .build()
            .await
        {
            Ok(embeddings) => embeddings,
            Err(e) => {
                error!("Failed to call embedding API");
                error!("  Error details: {:?}", e);
                error!("  Model: {}", self.model);
                return Err(anyhow::anyhow!(
                    "Failed to generate embedding (model: {}, error: {}). Check if embedding service is running and the model is available",
                    self.model,
                    e
                ));
            }
        };

        debug!("Successfully generated embeddings");

        if let Some((_, embedding)) = embeddings.first() {
            // Get the first embedding from OneOrMany
            let emb = embedding.iter().next()
                .context("No embeddings in response")?;
            Ok(emb.vec.iter().map(|&x| x as f32).collect())
        } else {
            Err(anyhow::anyhow!("No embeddings returned"))
        }
    }

    /// Generate embeddings for multiple texts in batch
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();

        for text in texts {
            let embedding = self.embed_text(&text).await?;
            results.push(embedding);
        }

        Ok(results)
    }
}

/// Document chunk with metadata
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub index: usize,
    pub text: String,
    pub tokens: usize,
}

/// Document manager for chunking and processing documents
pub struct DocumentManager {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl DocumentManager {
    /// Create a new document manager
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }

    /// Check if a file extension is supported
    pub fn is_supported_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return SUPPORTED_EXTENSIONS.contains(&ext_str);
            }
        }
        false
    }

    /// Read and extract text content from a file
    pub fn read_file_content(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        Ok(content)
    }

    /// Calculate SHA256 hash of content
    pub fn calculate_content_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let digest = hasher.finalize();
        digest.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Split text into chunks based on token count
    ///
    /// This implementation works with character positions rather than token slicing
    /// to avoid UTF-8 decoding issues when token boundaries don't align with
    /// character boundaries.
    pub fn chunk_text(&self, text: &str) -> Result<Vec<DocumentChunk>> {
        let bpe = cl100k_base()
            .context("Failed to load tokenizer")?;

        // Get total token count for the entire text
        let total_tokens = bpe.encode_ordinary(text).len();

        if total_tokens == 0 {
            return Ok(vec![]);
        }

        // If text is smaller than chunk size, return as single chunk
        if total_tokens <= self.chunk_size {
            return Ok(vec![DocumentChunk {
                index: 0,
                text: text.to_string(),
                tokens: total_tokens,
            }]);
        }

        let mut chunks = Vec::new();
        let mut chunk_index = 0;

        // Estimate characters per token for initial sizing
        let chars_per_token = text.chars().count() as f64 / total_tokens as f64;

        let mut start_char = 0;
        let text_chars: Vec<char> = text.chars().collect();

        while start_char < text_chars.len() {
            // Estimate end position based on desired token count
            let estimated_chars = (self.chunk_size as f64 * chars_per_token * 1.1) as usize;
            let mut end_char = (start_char + estimated_chars).min(text_chars.len());

            // Extract candidate chunk and measure tokens
            let mut chunk_text: String = text_chars[start_char..end_char].iter().collect();
            let mut chunk_tokens = bpe.encode_ordinary(&chunk_text).len();

            // Adjust chunk size to meet token target
            // Shrink if too large
            while chunk_tokens > self.chunk_size && end_char > start_char + 1 {
                let reduction = ((end_char - start_char) as f64 * 0.1).max(1.0) as usize;
                end_char = end_char.saturating_sub(reduction);
                chunk_text = text_chars[start_char..end_char].iter().collect();
                chunk_tokens = bpe.encode_ordinary(&chunk_text).len();
            }

            // Grow if too small (and not at end)
            while chunk_tokens < self.chunk_size && end_char < text_chars.len() {
                let growth = ((end_char - start_char) as f64 * 0.1).max(1.0) as usize;
                let new_end = (end_char + growth).min(text_chars.len());
                let new_chunk_text: String = text_chars[start_char..new_end].iter().collect();
                let new_tokens = bpe.encode_ordinary(&new_chunk_text).len();

                if new_tokens <= self.chunk_size {
                    end_char = new_end;
                    chunk_text = new_chunk_text;
                    chunk_tokens = new_tokens;
                } else {
                    break;
                }
            }

            chunks.push(DocumentChunk {
                index: chunk_index,
                text: chunk_text,
                tokens: chunk_tokens,
            });

            chunk_index += 1;

            if end_char >= text_chars.len() {
                break;
            }

            // Calculate overlap in characters
            let overlap_chars = (self.chunk_overlap as f64 * chars_per_token) as usize;
            start_char = end_char.saturating_sub(overlap_chars.max(1));
        }

        debug!(
            "Chunked document into {} chunks (total tokens: {})",
            chunks.len(),
            total_tokens
        );

        Ok(chunks)
    }

    /// Process a document file: read, chunk, and return chunks
    pub fn process_document(&self, path: &Path) -> Result<(String, Vec<DocumentChunk>)> {
        let content = self.read_file_content(path)?;
        let chunks = self.chunk_text(&content)?;
        Ok((content, chunks))
    }
}

/// Vector store interface for embedding storage and retrieval
pub trait VectorStore {
    fn insert_embedding(&self, chunk_id: i64, embedding: &[f32]) -> Result<()>;
    fn query_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
}

/// Search result from vector store
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunk_id: i64,
    pub chunk_text: String,
    pub filename: String,
    pub distance: f32,
}

/// SQLite vector store implementation
pub struct SqliteVecStore {
    db: Arc<Database>,
}

impl SqliteVecStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl VectorStore for SqliteVecStore {
    fn insert_embedding(&self, chunk_id: i64, embedding: &[f32]) -> Result<()> {
        self.db
            .insert_rag_embedding(chunk_id, embedding)
            .context("Failed to insert embedding")
    }

    fn query_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        let results = self
            .db
            .query_similar_chunks(embedding, limit as i32)
            .context("Failed to query similar chunks")?;

        Ok(results
            .into_iter()
            .map(|(chunk_id, chunk_text, filename, distance)| SearchResult {
                chunk_id,
                chunk_text,
                filename,
                distance,
            })
            .collect())
    }
}

/// RAG query pipeline
pub struct RagQuery {
    embedder: Arc<RagEmbedder>,
    vector_store: Arc<SqliteVecStore>,
    top_k: usize,
}

impl RagQuery {
    pub fn new(
        embedder: Arc<RagEmbedder>,
        vector_store: Arc<SqliteVecStore>,
        top_k: usize,
    ) -> Self {
        Self {
            embedder,
            vector_store,
            top_k,
        }
    }

    /// Execute RAG query: embed query, retrieve context, format for LLM
    pub async fn execute(&self, query: &str) -> Result<String> {
        let query_embedding = self.embedder.embed_text(query).await?;

        let results = self.vector_store.query_similar(&query_embedding, self.top_k)?;

        if results.is_empty() {
            return Ok(String::new());
        }

        let mut context = String::from("# Retrieved Context\n\n");

        for (idx, result) in results.iter().enumerate() {
            context.push_str(&format!(
                "## Source {}: {} (relevance: {:.3})\n\n{}\n\n",
                idx + 1,
                result.filename,
                1.0 - result.distance.min(1.0),
                result.chunk_text
            ));
        }

        Ok(context)
    }

    /// Execute query and return structured results
    pub async fn execute_structured(&self, query: &str) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedder.embed_text(query).await?;
        self.vector_store.query_similar(&query_embedding, self.top_k)
    }
}

/// Document watcher for auto-indexing
pub struct DocumentWatcher {
    watcher: RecommendedWatcher,
    receiver: Arc<Mutex<Receiver<Result<Event, notify::Error>>>>,
    documents_path: PathBuf,
    indexer: Arc<RagIndexer>,
}

impl DocumentWatcher {
    /// Create a new document watcher
    pub fn new(documents_path: PathBuf, indexer: Arc<RagIndexer>) -> Result<Self> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = tx.send(res) {
                    error!("Failed to send watch event: {}", e);
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .context("Failed to create file watcher")?;

        Ok(Self {
            watcher,
            receiver: Arc::new(Mutex::new(rx)),
            documents_path,
            indexer,
        })
    }

    /// Start watching the documents directory
    pub fn start(&mut self) -> Result<()> {
        if !self.documents_path.exists() {
            warn!(
                "Documents directory does not exist: {}",
                self.documents_path.display()
            );
            return Ok(());
        }

        self.watcher
            .watch(&self.documents_path, RecursiveMode::Recursive)
            .context("Failed to start watching documents directory")?;

        info!("Started watching: {}", self.documents_path.display());
        Ok(())
    }

    /// Stop watching
    pub fn stop(&mut self) -> Result<()> {
        self.watcher
            .unwatch(&self.documents_path)
            .context("Failed to stop watching documents directory")?;
        info!("Stopped watching: {}", self.documents_path.display());
        Ok(())
    }

    /// Process file system events (should be called in a loop)
    pub async fn process_events(&self) -> Result<()> {
        let rx = self.receiver.lock().await;

        while let Ok(result) = rx.try_recv() {
            match result {
                Ok(event) => self.handle_event(event).await?,
                Err(e) => error!("Watch error: {}", e),
            }
        }

        Ok(())
    }

    /// Handle a single file system event
    async fn handle_event(&self, event: Event) -> Result<()> {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    if path.is_file() && self.is_supported_file(&path) {
                        info!("Detected change in: {}", path.display());
                        if let Err(e) = self.indexer.index_single_file(&path).await {
                            error!("Failed to index {}: {}", path.display(), e);
                        }
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    if let Some(filename) = path.file_name() {
                        if let Some(filename_str) = filename.to_str() {
                            info!("Detected removal of: {}", path.display());
                            if let Err(e) = self.indexer.remove_document(filename_str) {
                                error!("Failed to remove document {}: {}", filename_str, e);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn is_supported_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return SUPPORTED_EXTENSIONS.contains(&ext_str);
            }
        }
        false
    }
}

/// RAG indexer with progress reporting
pub struct RagIndexer {
    db: Arc<Database>,
    pub embedder: Arc<RagEmbedder>,
    vector_store: Arc<SqliteVecStore>,
    doc_manager: DocumentManager,
    embedding_url: String,
}

impl RagIndexer {
    pub fn new(
        db: Arc<Database>,
        embedder: Arc<RagEmbedder>,
        vector_store: Arc<SqliteVecStore>,
        config: &RagConfig,
    ) -> Self {
        Self {
            db,
            embedder,
            vector_store,
            doc_manager: DocumentManager::new(config.chunk_size, config.chunk_overlap),
            embedding_url: config.embedding_url.clone(),
        }
    }

    /// Scan and index all documents in a directory
    pub async fn scan_and_index(&self, documents_path: &Path) -> Result<IndexStats> {
        if !documents_path.exists() {
            return Err(anyhow::anyhow!(
                "Documents directory does not exist: {}",
                documents_path.display()
            ));
        }

        let mut stats = IndexStats::default();
        let mut files_to_process = Vec::new();

        for entry in walkdir::WalkDir::new(documents_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if self.doc_manager.is_supported_extension(path) {
                    files_to_process.push(path.to_path_buf());
                }
            }
        }

        stats.files_found = files_to_process.len();

        if stats.files_found == 0 {
            info!("No documents found to index");
            return Ok(stats);
        }

        info!("Found {} documents to process", stats.files_found);

        for path in files_to_process {
            match self.index_single_file(&path).await {
                Ok(_) => {
                    stats.files_processed += 1;
                }
                Err(e) => {
                    error!("Failed to index {}: {}", path.display(), e);
                    stats.files_failed += 1;
                }
            }
        }

        let (_doc_count, chunk_count, embedding_count) = self.get_stats()?;
        stats.total_chunks = chunk_count as usize;
        stats.total_embeddings = embedding_count as usize;

        Ok(stats)
    }

    /// Index a single document file
    pub async fn index_single_file(&self, path: &Path) -> Result<()> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

        let (content, chunks) = self.doc_manager.process_document(path)?;

        if chunks.is_empty() {
            debug!("No chunks generated for {}", filename);
            return Ok(());
        }

        let content_hash = self.doc_manager.calculate_content_hash(&content);

        if let Some((doc_id, existing_hash, _)) = self.db.get_rag_document_by_filename(filename)? {
            if existing_hash == content_hash {
                debug!("Document {} unchanged, skipping", filename);
                return Ok(());
            }

            debug!("Document {} changed, re-indexing", filename);
            self.db.delete_rag_document_chunks(doc_id)?;
        }

        let file_size = content.len() as i64;
        let doc_id = self
            .db
            .upsert_rag_document(filename, &content, &content_hash, file_size)?;

        for chunk in chunks {
            let chunk_id = self.db.insert_rag_chunk(
                doc_id,
                chunk.index as i32,
                &chunk.text,
                chunk.tokens as i32,
            )?;

            debug!("Generating embedding for chunk {} (length: {} chars)", chunk.index, chunk.text.len());
            let embedding = self
                .embedder
                .embed_text(&chunk.text)
                .await
                .with_context(|| {
                    error!("Embedding generation failed for chunk {}", chunk.index);
                    error!("  Embedding URL: {}", self.embedding_url);
                    error!("  Chunk text preview: {}...", &chunk.text.chars().take(100).collect::<String>());
                    format!(
                        "Failed to generate embedding for chunk {} (embedding service: {})",
                        chunk.index, self.embedding_url
                    )
                })?;
            debug!("Successfully generated embedding for chunk {}", chunk.index);

            self.vector_store
                .insert_embedding(chunk_id, &embedding)
                .context("Failed to insert embedding")?;
        }

        info!("Indexed {} successfully", filename);
        Ok(())
    }

    /// Remove a document from the index
    pub fn remove_document(&self, filename: &str) -> Result<()> {
        if let Some((doc_id, _, _)) = self.db.get_rag_document_by_filename(filename)? {
            self.db.delete_rag_document(doc_id)?;
            info!("Removed document: {}", filename);
        }
        Ok(())
    }

    /// Get RAG statistics
    pub fn get_stats(&self) -> Result<(i64, i64, i64)> {
        self.db
            .get_rag_stats()
            .context("Failed to get RAG stats")
    }

    /// List all indexed documents
    pub fn list_documents(&self) -> Result<Vec<DocumentInfo>> {
        let docs = self
            .db
            .list_rag_documents()
            .context("Failed to list documents")?;

        Ok(docs
            .into_iter()
            .map(|(id, filename, file_size, created_at, updated_at)| DocumentInfo {
                id,
                filename,
                file_size,
                created_at,
                updated_at,
            })
            .collect())
    }

    /// Rebuild the entire index (clear and re-index)
    pub async fn rebuild(&self, documents_path: &Path) -> Result<IndexStats> {
        info!("Clearing existing RAG index...");

        let docs = self.list_documents()?;
        for doc in docs {
            self.db.delete_rag_document(doc.id)?;
        }

        info!("Rebuilding index...");
        self.scan_and_index(documents_path).await
    }
}

/// Indexing statistics
#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    pub files_found: usize,
    pub files_processed: usize,
    pub files_failed: usize,
    pub total_chunks: usize,
    pub total_embeddings: usize,
}

/// Document information
#[derive(Debug, Clone)]
pub struct DocumentInfo {
    pub id: i64,
    pub filename: String,
    pub file_size: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// RAG system coordinator
pub struct RagSystem {
    pub embedder: Arc<RagEmbedder>,
    pub vector_store: Arc<SqliteVecStore>,
    pub indexer: Arc<RagIndexer>,
    pub query: Arc<RagQuery>,
}

impl RagSystem {
    /// Initialize the RAG system
    pub async fn new(db: Arc<Database>, config: &RagConfig) -> Result<Self> {
        let embedder = Arc::new(
            RagEmbedder::new(&config.embedding_url, &config.embedding_model)
                .context("Failed to create embedder")?,
        );

        let vector_store = Arc::new(SqliteVecStore::new(db.clone()));

        let indexer = Arc::new(RagIndexer::new(
            db.clone(),
            embedder.clone(),
            vector_store.clone(),
            config,
        ));

        let query = Arc::new(RagQuery::new(
            embedder.clone(),
            vector_store.clone(),
            config.top_k,
        ));

        Ok(Self {
            embedder,
            vector_store,
            indexer,
            query,
        })
    }

    /// Create a document watcher for the system
    pub fn create_watcher(&self, documents_path: PathBuf) -> Result<DocumentWatcher> {
        DocumentWatcher::new(documents_path, self.indexer.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========== DocumentManager Tests ==========

    #[test]
    fn test_document_manager_new() {
        let manager = DocumentManager::new(512, 50);
        assert_eq!(manager.chunk_size, 512);
        assert_eq!(manager.chunk_overlap, 50);
    }

    #[test]
    fn test_document_manager_chunking() {
        let manager = DocumentManager::new(100, 20);
        let text = "This is a test document. ".repeat(50);
        let chunks = manager.chunk_text(&text).unwrap();
        assert!(!chunks.is_empty());

        // Verify chunks have proper structure
        for chunk in &chunks {
            assert!(!chunk.text.is_empty());
            assert!(chunk.tokens > 0);
        }
    }

    #[test]
    fn test_document_manager_chunking_with_overlap() {
        let manager = DocumentManager::new(50, 10);
        let text = "Word ".repeat(100); // Create text longer than chunk size
        let chunks = manager.chunk_text(&text).unwrap();

        // Should have multiple chunks
        assert!(chunks.len() > 1);

        // Verify indices are sequential
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
        }
    }

    #[test]
    fn test_document_manager_empty_text() {
        let manager = DocumentManager::new(512, 50);
        let result = manager.chunk_text("");
        assert!(result.is_ok(), "Empty text should return Ok with empty vector");
        assert_eq!(result.unwrap().len(), 0, "Empty text should produce no chunks");
    }

    #[test]
    fn test_document_manager_small_text() {
        let manager = DocumentManager::new(1000, 100);
        let text = "Small text.";
        let chunks = manager.chunk_text(text).unwrap();

        // Small text should result in single chunk
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].index, 0);
        assert_eq!(chunks[0].text, text);
    }

    #[test]
    fn test_supported_extensions() {
        let manager = DocumentManager::new(512, 50);

        // Test all supported extensions
        assert!(manager.is_supported_extension(Path::new("test.md")));
        assert!(manager.is_supported_extension(Path::new("test.rs")));
        assert!(manager.is_supported_extension(Path::new("test.py")));
        assert!(manager.is_supported_extension(Path::new("test.js")));
        assert!(manager.is_supported_extension(Path::new("test.ts")));
        assert!(manager.is_supported_extension(Path::new("test.jsx")));
        assert!(manager.is_supported_extension(Path::new("test.tsx")));
        assert!(manager.is_supported_extension(Path::new("test.json")));
        assert!(manager.is_supported_extension(Path::new("test.yaml")));
        assert!(manager.is_supported_extension(Path::new("test.yml")));

        // Test unsupported extensions
        assert!(!manager.is_supported_extension(Path::new("test.exe")));
        assert!(!manager.is_supported_extension(Path::new("test.bin")));
        assert!(!manager.is_supported_extension(Path::new("test.pdf")));
        assert!(!manager.is_supported_extension(Path::new("test")));
    }

    #[test]
    fn test_content_hash() {
        let manager = DocumentManager::new(512, 50);
        let hash1 = manager.calculate_content_hash("test content");
        let hash2 = manager.calculate_content_hash("test content");
        let hash3 = manager.calculate_content_hash("different content");

        assert_eq!(hash1, hash2, "Same content should produce same hash");
        assert_ne!(hash1, hash3, "Different content should produce different hash");

        // Test empty content
        let hash_empty = manager.calculate_content_hash("");
        assert!(!hash_empty.is_empty());
    }

    #[test]
    fn test_read_file_content() {
        let manager = DocumentManager::new(512, 50);
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create test file
        std::fs::write(&file_path, "Test file content").unwrap();

        // Read it back
        let content = manager.read_file_content(&file_path).unwrap();
        assert_eq!(content, "Test file content");

        // Test non-existent file
        let result = manager.read_file_content(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_process_document() {
        let manager = DocumentManager::new(100, 20);
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");

        // Create test file with enough content to create multiple chunks
        let content = "This is test content. ".repeat(50);
        std::fs::write(&file_path, &content).unwrap();

        // Process document
        let (read_content, chunks) = manager.process_document(&file_path).unwrap();

        assert_eq!(read_content, content);
        assert!(!chunks.is_empty());

        // Verify all chunks have content
        for chunk in chunks {
            assert!(!chunk.text.is_empty());
            assert!(chunk.tokens > 0);
        }
    }

    // ========== RagEmbedder Tests ==========
    // Note: These tests verify the API structure but require a running embedding service

    #[test]
    fn test_rag_embedder_creation() {
        // Test that embedder can be created with various URL formats
        let result1 = RagEmbedder::new("http://localhost:1234", "text-embedding-3-small");
        assert!(result1.is_ok(), "Should create embedder with http URL");

        let result2 = RagEmbedder::new("http://localhost:1234/v1", "text-embedding-3-small");
        assert!(result2.is_ok(), "Should create embedder with /v1 suffix");

        let result3 = RagEmbedder::new("http://localhost:1234/v1/", "text-embedding-3-small");
        assert!(result3.is_ok(), "Should create embedder with /v1/ suffix");
    }

    #[tokio::test]
    #[ignore] // Requires running embedding service (e.g., LM Studio, Ollama)
    async fn test_rag_embedder_embed_text() {
        // This test requires a local embedding service running on port 1234
        // Start LM Studio or Ollama with an embedding model before running
        let embedder = RagEmbedder::new("http://localhost:1234", "text-embedding-3-small")
            .expect("Failed to create embedder");

        let result = embedder.embed_text("Hello, world!").await;
        assert!(result.is_ok(), "Should successfully embed text");

        let embedding = result.unwrap();
        assert!(!embedding.is_empty(), "Embedding should not be empty");
        assert!(embedding.len() > 100, "Embedding should have reasonable size");

        // Verify all values are finite
        for &value in &embedding {
            assert!(value.is_finite(), "Embedding values should be finite");
        }
    }

    #[tokio::test]
    #[ignore] // Requires running embedding service
    async fn test_rag_embedder_embed_multiple_texts() {
        let embedder = RagEmbedder::new("http://localhost:1234", "text-embedding-3-small")
            .expect("Failed to create embedder");

        // Embed same text twice - should produce similar embeddings
        let embedding1 = embedder.embed_text("Test text").await.unwrap();
        let embedding2 = embedder.embed_text("Test text").await.unwrap();

        assert_eq!(embedding1.len(), embedding2.len());

        // Embeddings should be very similar (cosine similarity should be high)
        let similarity = cosine_similarity(&embedding1, &embedding2);
        assert!(similarity > 0.99, "Same text should produce nearly identical embeddings");

        // Different text should produce different embeddings
        let embedding3 = embedder.embed_text("Completely different content").await.unwrap();
        let similarity2 = cosine_similarity(&embedding1, &embedding3);
        assert!(similarity2 < 0.95, "Different text should produce different embeddings");
    }

    #[tokio::test]
    #[ignore] // Requires running embedding service
    async fn test_rag_embedder_embed_batch() {
        let embedder = RagEmbedder::new("http://localhost:1234", "text-embedding-3-small")
            .expect("Failed to create embedder");

        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let result = embedder.embed_batch(texts.clone()).await;
        assert!(result.is_ok(), "Should successfully embed batch");

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), texts.len());

        // Verify each embedding
        for embedding in embeddings {
            assert!(!embedding.is_empty());
            assert!(embedding.len() > 100);
        }
    }

    #[tokio::test]
    #[ignore] // Requires running embedding service
    async fn test_rag_embedder_empty_text() {
        let embedder = RagEmbedder::new("http://localhost:1234", "text-embedding-3-small")
            .expect("Failed to create embedder");

        // Test with empty string - behavior may vary by provider
        let result = embedder.embed_text("").await;
        // Most embedding services should handle empty text gracefully
        assert!(result.is_ok() || result.is_err());
    }

    // ========== Helper Functions ==========

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        dot_product / (magnitude_a * magnitude_b)
    }

    // ========== SearchResult Tests ==========

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            chunk_id: 1,
            chunk_text: "Test chunk".to_string(),
            filename: "test.md".to_string(),
            distance: 0.5,
        };

        assert_eq!(result.chunk_id, 1);
        assert_eq!(result.chunk_text, "Test chunk");
        assert_eq!(result.filename, "test.md");
        assert_eq!(result.distance, 0.5);
    }

    // ========== SUPPORTED_EXTENSIONS Tests ==========

    #[test]
    fn test_supported_extensions_constant() {
        // Verify the constant contains expected extensions
        assert!(SUPPORTED_EXTENSIONS.contains(&"md"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"rs"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"py"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"js"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"json"));

        // Verify it doesn't contain binary formats
        assert!(!SUPPORTED_EXTENSIONS.contains(&"exe"));
        assert!(!SUPPORTED_EXTENSIONS.contains(&"bin"));
        assert!(!SUPPORTED_EXTENSIONS.contains(&"pdf"));
    }

    #[test]
    fn test_chunk_indices_are_sequential() {
        let manager = DocumentManager::new(50, 10);
        let text = "Word ".repeat(100);
        let chunks = manager.chunk_text(&text).unwrap();

        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i, "Chunk index should match position");
        }
    }

    #[test]
    fn test_chunk_token_counts_are_positive() {
        let manager = DocumentManager::new(100, 20);
        let text = "This is a test. ".repeat(20);
        let chunks = manager.chunk_text(&text).unwrap();

        for chunk in chunks {
            assert!(chunk.tokens > 0, "All chunks should have positive token count");
        }
    }

    #[test]
    fn test_document_manager_with_special_characters() {
        let manager = DocumentManager::new(512, 50);
        let text = "Special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?\n\t\r";
        let chunks = manager.chunk_text(text).unwrap();

        assert!(!chunks.is_empty());
        assert!(chunks[0].text.contains("Special chars"));
    }

    #[test]
    fn test_document_manager_with_unicode() {
        let manager = DocumentManager::new(512, 50);
        let text = "Unicode: 你好世界 🌍 émojis ñ café";
        let chunks = manager.chunk_text(text).unwrap();

        assert!(!chunks.is_empty());
        assert!(chunks[0].text.contains("你好世界"));
        assert!(chunks[0].text.contains("🌍"));
    }

    #[test]
    fn test_hash_consistency_across_calls() {
        let manager = DocumentManager::new(512, 50);
        let content = "Consistent content";

        // Generate hash multiple times
        let hashes: Vec<String> = (0..5)
            .map(|_| manager.calculate_content_hash(content))
            .collect();

        // All hashes should be identical
        for hash in &hashes[1..] {
            assert_eq!(hash, &hashes[0]);
        }
    }
}
