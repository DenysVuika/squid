use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
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
    pub fn new(embedding_url: &str, model: &str) -> Result<Self> {
        // For local services (LM Studio, Ollama), use a dummy API key
        // The from_url method accepts: (api_key, base_url)
        let client = rig::providers::openai::Client::from_url("not-needed", embedding_url);
        Ok(Self {
            client,
            model: model.to_string(),
        })
    }

    /// Generate embeddings for a single text
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        use rig::embeddings::EmbeddingsBuilder;
        
        let model = self.client.embedding_model(&self.model);
        
        let embeddings = EmbeddingsBuilder::new(model)
            .simple_document("doc", text)
            .build()
            .await
            .context("Failed to generate embedding")?;

        if let Some(doc_embedding) = embeddings.first() {
            if let Some(embedding) = doc_embedding.embeddings.first() {
                Ok(embedding.vec.iter().map(|&x| x as f32).collect())
            } else {
                Err(anyhow::anyhow!("No embeddings in document"))
            }
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
        format!("{:x}", hasher.finalize())
    }

    /// Split text into chunks based on token count
    pub fn chunk_text(&self, text: &str) -> Result<Vec<DocumentChunk>> {
        let bpe = cl100k_base()
            .context("Failed to load tokenizer")?;

        let tokens = bpe.encode_with_special_tokens(text);
        let total_tokens = tokens.len();

        if total_tokens == 0 {
            return Ok(vec![]);
        }

        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;

        while start < total_tokens {
            let end = (start + self.chunk_size).min(total_tokens);
            let chunk_tokens = &tokens[start..end];

            let chunk_text = bpe
                .decode(chunk_tokens.to_vec())
                .context("Failed to decode tokens")?;

            chunks.push(DocumentChunk {
                index: chunk_index,
                text: chunk_text,
                tokens: chunk_tokens.len(),
            });

            chunk_index += 1;

            if end == total_tokens {
                break;
            }

            start = end.saturating_sub(self.chunk_overlap);
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
        let rx = self.receiver.lock().unwrap();

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
    embedder: Arc<RagEmbedder>,
    vector_store: Arc<SqliteVecStore>,
    doc_manager: DocumentManager,
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

            let embedding = self
                .embedder
                .embed_text(&chunk.text)
                .await
                .context("Failed to generate embedding")?;

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

    #[test]
    fn test_document_manager_chunking() {
        let manager = DocumentManager::new(100, 20);
        let text = "This is a test document. ".repeat(50);
        let chunks = manager.chunk_text(&text).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_supported_extensions() {
        let manager = DocumentManager::new(512, 50);
        assert!(manager.is_supported_extension(Path::new("test.md")));
        assert!(manager.is_supported_extension(Path::new("test.rs")));
        assert!(!manager.is_supported_extension(Path::new("test.exe")));
    }

    #[test]
    fn test_content_hash() {
        let manager = DocumentManager::new(512, 50);
        let hash1 = manager.calculate_content_hash("test content");
        let hash2 = manager.calculate_content_hash("test content");
        let hash3 = manager.calculate_content_hash("different content");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
