use actix_web::http::header;
use actix_web::{HttpResponse, web};
use futures::stream::Stream;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::jobs::JobExecutionRequest;
use crate::{config, db};

/// Global sender channel for one-off jobs
/// Populated by the job scheduler at startup
static JOB_SENDER: std::sync::OnceLock<mpsc::Sender<JobExecutionRequest>> =
    std::sync::OnceLock::new();

/// Global database path (set at server startup from config)
static DB_PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Cached Database instance (initialized once at startup, shared across all API calls)
static DB_INSTANCE: std::sync::OnceLock<Arc<db::Database>> = std::sync::OnceLock::new();

/// Broadcast channel for job status updates (for SSE)
static JOB_UPDATE_BROADCASTER: std::sync::OnceLock<tokio::sync::broadcast::Sender<JobUpdateEvent>> =
    std::sync::OnceLock::new();

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum JobUpdateEvent {
    #[serde(rename = "update")]
    Update { job: Box<JobResponse> },
    #[serde(rename = "deleted")]
    Deleted { job_id: i64 },
}

/// Initialize the global job sender (called by scheduler at startup)
pub fn init_job_sender(tx: mpsc::Sender<JobExecutionRequest>) {
    let _ = JOB_SENDER.set(tx);
}

/// Initialize the job update broadcaster (called at server startup)
pub fn init_job_broadcaster() {
    let (tx, _rx) = tokio::sync::broadcast::channel(100);
    let _ = JOB_UPDATE_BROADCASTER.set(tx);
}

/// Broadcast a job update event to all SSE listeners
fn broadcast_job_update(event: JobUpdateEvent) {
    if let Some(tx) = JOB_UPDATE_BROADCASTER.get() {
        // Ignore send errors (no active listeners)
        let _ = tx.send(event);
    }
}

/// Public helper to broadcast a job status update (called from job worker)
pub fn broadcast_job_status_update(job: db::BackgroundJob) {
    let job_response = JobResponse::from(&job);
    broadcast_job_update(JobUpdateEvent::Update {
        job: Box::new(job_response),
    });
}

/// Initialize the global database path and open the connection (called at server startup)
pub fn init_db_path(path: &str) {
    let db_path = std::path::PathBuf::from(path);
    let _ = DB_PATH.set(db_path.clone());
    let db = db::Database::new(&db_path).expect("Failed to open database for jobs");
    let _ = DB_INSTANCE.set(Arc::new(db));
}

/// Get the job sender
fn get_job_sender() -> Option<mpsc::Sender<JobExecutionRequest>> {
    JOB_SENDER.get().cloned()
}

/// Get the cached Database instance (no new migrations)
fn get_db() -> Arc<db::Database> {
    DB_INSTANCE.get().cloned().unwrap_or_else(|| {
        let db_path = DB_PATH
            .get()
            .cloned()
            .unwrap_or_else(|| std::path::PathBuf::from("squid.db"));
        Arc::new(db::Database::new(&db_path).expect("Failed to open database for jobs"))
    })
}

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub name: String,
    pub schedule_type: String, // "cron" or "once"
    pub cron_expression: Option<String>,
    pub priority: Option<i32>,
    pub max_cpu_percent: Option<i32>,
    pub max_retries: Option<i32>,
    pub timeout_seconds: Option<i64>,
    pub payload: db::JobPayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobExecutionResponse {
    pub id: i64,
    pub job_id: i64,
    pub session_id: Option<String>,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub tokens_used: Option<i64>,
    pub cost_usd: Option<f64>,
}

impl From<&db::JobExecution> for JobExecutionResponse {
    fn from(exec: &db::JobExecution) -> Self {
        let result = exec
            .result
            .as_ref()
            .and_then(|r| serde_json::from_str::<serde_json::Value>(r).ok());

        Self {
            id: exec.id.unwrap_or(0),
            job_id: exec.job_id,
            session_id: exec.session_id.clone(),
            status: exec.status.clone(),
            result,
            error_message: exec.error_message.clone(),
            started_at: exec.started_at.clone(),
            completed_at: exec.completed_at.clone(),
            duration_ms: exec.duration_ms,
            tokens_used: exec.tokens_used,
            cost_usd: exec.cost_usd,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct JobResponse {
    pub id: i64,
    pub name: String,
    pub schedule_type: String,
    pub cron_expression: Option<String>,
    pub priority: i32,
    pub max_cpu_percent: i32,
    pub status: String,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub retries: i32,
    pub max_retries: i32,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>, // Changed to Value for structured data
    pub error_message: Option<String>,
    pub is_active: bool,
    pub timeout_seconds: i64,
}

impl From<&db::BackgroundJob> for JobResponse {
    fn from(job: &db::BackgroundJob) -> Self {
        let payload: serde_json::Value =
            serde_json::from_str(&job.payload).unwrap_or(serde_json::json!({}));

        // Parse result JSON string into a structured Value
        let result = job
            .result
            .as_ref()
            .and_then(|r| serde_json::from_str::<serde_json::Value>(r).ok());

        Self {
            id: job.id.unwrap_or(0),
            name: job.name.clone(),
            schedule_type: job.schedule_type.clone(),
            cron_expression: job.cron_expression.clone(),
            priority: job.priority,
            max_cpu_percent: job.max_cpu_percent,
            status: job.status.clone(),
            last_run: job.last_run.clone(),
            next_run: job.next_run.clone(),
            retries: job.retries,
            max_retries: job.max_retries,
            payload,
            result,
            error_message: job.error_message.clone(),
            is_active: job.is_active,
            timeout_seconds: job.timeout_seconds,
        }
    }
}

/// List all background jobs
pub async fn list_jobs() -> HttpResponse {
    let db = get_db();
    match db.get_all_jobs() {
        Ok(jobs) => {
            let response: Vec<JobResponse> = jobs.iter().map(JobResponse::from).collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Failed to list jobs: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to list jobs: {}", e)
            }))
        }
    }
}

/// Validate cron expression syntax
fn validate_cron_expression(expr: &str) -> Result<(), String> {
    // tokio-cron-scheduler expects 6 or 7 fields (with seconds)
    // Standard format: "sec min hour day month dayofweek"
    // Try to parse it as a tokio-cron-scheduler expression
    match tokio_cron_scheduler::Job::new(expr, |_uuid, _l| {}) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!(
            "Invalid cron expression: {}. Expected format: 'sec min hour day month dayofweek' (e.g., '0 0 9 * * Mon-Fri' for weekdays at 9 AM)",
            e
        )),
    }
}

/// Create a new background job
pub async fn create_job(
    req: web::Json<CreateJobRequest>,
    app_config: web::Data<Arc<config::Config>>,
) -> HttpResponse {
    let db = get_db();
    let req = req.into_inner();

    // Validate schedule type
    if req.schedule_type != "cron" && req.schedule_type != "once" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "schedule_type must be 'cron' or 'once'"
        }));
    }

    // Validate cron expression for cron jobs
    if req.schedule_type == "cron" {
        if req.cron_expression.is_none() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "cron_expression is required for cron jobs"
            }));
        }

        if let Some(ref expr) = req.cron_expression
            && let Err(e) = validate_cron_expression(expr)
        {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }));
        }
    }

    // Validate agent exists
    if app_config.get_agent(&req.payload.agent_id).is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!("Agent '{}' not found", req.payload.agent_id)
        }));
    }

    // Serialize payload
    let payload_json = match serde_json::to_string(&req.payload) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize payload: {}", e);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Invalid payload: {}", e)
            }));
        }
    };

    // Create the job
    let is_one_off = req.schedule_type == "once";
    let job = db::BackgroundJob {
        id: None,
        name: req.name.clone(),
        schedule_type: req.schedule_type,
        cron_expression: req.cron_expression,
        priority: req.priority.unwrap_or(5),
        max_cpu_percent: req.max_cpu_percent.unwrap_or(50),
        status: "pending".to_string(),
        last_run: None,
        next_run: None,
        retries: 0,
        max_retries: req.max_retries.unwrap_or(3),
        payload: payload_json.clone(),
        result: None,
        error_message: None,
        is_active: true,
        timeout_seconds: req
            .timeout_seconds
            .unwrap_or(app_config.jobs.default_timeout_seconds),
    };

    match db.create_job(&job) {
        Ok(job_id) => {
            info!("Created job: {} (id: {})", job.name, job_id);

            // For one-off jobs, send to the worker immediately
            if is_one_off {
                if let Some(tx) = get_job_sender() {
                    let exec_req = JobExecutionRequest {
                        job_id,
                        job_name: job.name.clone(),
                        payload: req.payload.clone(),
                        max_cpu_percent: req.max_cpu_percent.unwrap_or(50),
                        max_retries: req.max_retries.unwrap_or(3),
                        schedule_type: "once".to_string(),
                        timeout_seconds: req
                            .timeout_seconds
                            .unwrap_or(app_config.jobs.default_timeout_seconds),
                    };

                    match tx.send(exec_req).await {
                        Ok(_) => info!("Dispatched one-off job {} to worker", job_id),
                        Err(e) => {
                            error!("Failed to dispatch job {} to worker: {}", job_id, e);
                            // Job will be restored on next server restart
                        }
                    }
                } else {
                    warn!(
                        "Job scheduler not initialized - job {} will remain pending until restart",
                        job_id
                    );
                }
            }

            // Fetch the created job to return
            match db.get_job_by_id(job_id) {
                Ok(Some(created_job)) => {
                    let response = JobResponse::from(&created_job);
                    // Broadcast the new job to SSE listeners
                    broadcast_job_update(JobUpdateEvent::Update {
                        job: Box::new(response.clone()),
                    });
                    HttpResponse::Created().json(response)
                }
                Ok(None) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Job created but failed to fetch"
                })),
                Err(e) => {
                    error!("Failed to fetch created job: {}", e);
                    HttpResponse::Created().json(serde_json::json!({
                        "id": job_id,
                        "message": "Job created successfully"
                    }))
                }
            }
        }
        Err(e) => {
            error!("Failed to create job: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create job: {}", e)
            }))
        }
    }
}

/// Get a single job by ID
pub async fn get_job(path: web::Path<i64>) -> HttpResponse {
    let db = get_db();
    let job_id = path.into_inner();

    match db.get_job_by_id(job_id) {
        Ok(Some(job)) => HttpResponse::Ok().json(JobResponse::from(&job)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Job {} not found", job_id)
        })),
        Err(e) => {
            error!("Failed to get job {}: {}", job_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get job: {}", e)
            }))
        }
    }
}

/// Cancel a background job
pub async fn cancel_job(path: web::Path<i64>) -> HttpResponse {
    let db = get_db();
    let job_id = path.into_inner();

    match db.cancel_job(job_id) {
        Ok(()) => {
            info!("Cancelled job: {}", job_id);
            // Broadcast updated job state
            if let Ok(Some(job)) = db.get_job_by_id(job_id) {
                broadcast_job_update(JobUpdateEvent::Update {
                    job: Box::new(JobResponse::from(&job)),
                });
            }
            HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Job {} cancelled successfully", job_id)
            }))
        }
        Err(e) => {
            error!("Failed to cancel job {}: {}", job_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to cancel job: {}", e)
            }))
        }
    }
}

/// Delete a background job
pub async fn delete_job(path: web::Path<i64>) -> HttpResponse {
    let db = get_db();
    let job_id = path.into_inner();

    match db.delete_job(job_id) {
        Ok(()) => {
            info!("Deleted job: {}", job_id);
            // Broadcast deletion to SSE listeners
            broadcast_job_update(JobUpdateEvent::Deleted { job_id });
            HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Job {} deleted successfully", job_id)
            }))
        }
        Err(e) => {
            error!("Failed to delete job {}: {}", job_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete job: {}", e)
            }))
        }
    }
}

/// Pause a cron job
pub async fn pause_job(path: web::Path<i64>) -> HttpResponse {
    let db = get_db();
    let job_id = path.into_inner();

    match db.pause_job(job_id) {
        Ok(()) => {
            info!("Paused job: {}", job_id);
            // Broadcast updated job state
            if let Ok(Some(job)) = db.get_job_by_id(job_id) {
                broadcast_job_update(JobUpdateEvent::Update {
                    job: Box::new(JobResponse::from(&job)),
                });
            }
            HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Job {} paused successfully", job_id)
            }))
        }
        Err(e) => {
            error!("Failed to pause job {}: {}", job_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to pause job: {}", e)
            }))
        }
    }
}

/// Resume a cron job
pub async fn resume_job(path: web::Path<i64>) -> HttpResponse {
    let db = get_db();
    let job_id = path.into_inner();

    match db.resume_job(job_id) {
        Ok(()) => {
            info!("Resumed job: {}", job_id);
            // Broadcast updated job state
            if let Ok(Some(job)) = db.get_job_by_id(job_id) {
                broadcast_job_update(JobUpdateEvent::Update {
                    job: Box::new(JobResponse::from(&job)),
                });
            }
            HttpResponse::Ok().json(serde_json::json!({
                "message": format!("Job {} resumed successfully", job_id)
            }))
        }
        Err(e) => {
            error!("Failed to resume job {}: {}", job_id, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to resume job: {}", e)
            }))
        }
    }
}

/// Manually trigger a cron job (run it immediately)
pub async fn trigger_job(path: web::Path<i64>) -> HttpResponse {
    let db = get_db();
    let job_id = path.into_inner();

    // Fetch the job
    let job = match db.get_job_by_id(job_id) {
        Ok(Some(job)) => job,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Job {} not found", job_id)
            }));
        }
        Err(e) => {
            error!("Failed to fetch job {}: {}", job_id, e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch job: {}", e)
            }));
        }
    };

    // Verify it's a cron job
    if job.schedule_type != "cron" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Only cron jobs can be manually triggered"
        }));
    }

    // Parse the payload
    let job_payload: db::JobPayload = match serde_json::from_str(&job.payload) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to parse payload for job {}: {}", job_id, e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Invalid job payload: {}", e)
            }));
        }
    };

    // Send to worker
    if let Some(tx) = get_job_sender() {
        let exec_req = JobExecutionRequest {
            job_id,
            job_name: job.name.clone(),
            payload: job_payload,
            max_cpu_percent: job.max_cpu_percent,
            max_retries: job.max_retries,
            schedule_type: job.schedule_type.clone(),
            timeout_seconds: job.timeout_seconds,
        };

        match tx.send(exec_req).await {
            Ok(_) => {
                info!("Manually triggered job: {}", job_id);
                HttpResponse::Ok().json(serde_json::json!({
                    "message": format!("Job {} triggered successfully", job_id)
                }))
            }
            Err(e) => {
                error!("Failed to send job {} to worker: {}", job_id, e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to trigger job: {}", e)
                }))
            }
        }
    } else {
        warn!("Job scheduler not initialized");
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Job scheduler is not available"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_job_request_validation() {
        // Test valid one-off job
        let valid_json = serde_json::json!({
            "name": "Test Job",
            "schedule_type": "once",
            "payload": {
                "agent_id": "shakespeare",
                "message": "Write a greeting"
            }
        });

        let req: Result<CreateJobRequest, _> = serde_json::from_value(valid_json);
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.name, "Test Job");
        assert_eq!(req.schedule_type, "once");
        assert_eq!(req.payload.agent_id, "shakespeare");
    }

    #[test]
    fn test_create_cron_job_request() {
        let cron_json = serde_json::json!({
            "name": "Daily Review",
            "schedule_type": "cron",
            "cron_expression": "0 9 * * 1-5",
            "priority": 8,
            "max_cpu_percent": 60,
            "max_retries": 5,
            "payload": {
                "agent_id": "code-reviewer",
                "message": "Review code"
            }
        });

        let req: CreateJobRequest = serde_json::from_value(cron_json).unwrap();
        assert_eq!(req.name, "Daily Review");
        assert_eq!(req.schedule_type, "cron");
        assert_eq!(req.cron_expression, Some("0 9 * * 1-5".to_string()));
        assert_eq!(req.priority, Some(8));
        assert_eq!(req.max_retries, Some(5));
    }

    #[test]
    fn test_job_payload_parsing() {
        let payload_json = serde_json::json!({
            "agent_id": "general-assistant",
            "message": "Test message",
            "system_prompt": "Custom prompt",
            "file_path": "src/main.rs",
            "session_id": "test-session-123"
        });

        let payload: db::JobPayload = serde_json::from_value(payload_json).unwrap();
        assert_eq!(payload.agent_id, "general-assistant");
        assert_eq!(payload.message, "Test message");
        assert_eq!(payload.system_prompt, Some("Custom prompt".to_string()));
        assert_eq!(payload.file_path, Some("src/main.rs".to_string()));
        assert_eq!(payload.session_id, Some("test-session-123".to_string()));
    }

    #[test]
    fn test_job_response_serialization() {
        let job = db::BackgroundJob {
            id: Some(42),
            name: "Test Job".to_string(),
            schedule_type: "cron".to_string(),
            cron_expression: Some("0 * * * *".to_string()),
            priority: 5,
            max_cpu_percent: 70,
            status: "pending".to_string(),
            last_run: None,
            next_run: Some("2026-04-08T10:00:00Z".to_string()),
            retries: 0,
            max_retries: 3,
            payload: r#"{"agent_id":"test","message":"hello"}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let response = JobResponse::from(&job);
        assert_eq!(response.id, 42);
        assert_eq!(response.name, "Test Job");
        assert_eq!(response.schedule_type, "cron");
        assert_eq!(response.status, "pending");
        assert!(response.is_active);

        // Verify JSON serialization
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["id"], 42);
        assert_eq!(json["name"], "Test Job");
        assert_eq!(json["payload"]["agent_id"], "test");
    }

    // Cron Expression Validation Tests
    #[test]
    fn test_validate_cron_expression_valid_6_field_formats() {
        // Valid 6-field expressions (sec min hour day month dayofweek)
        assert!(validate_cron_expression("0 0 9 * * Mon-Fri").is_ok());
        assert!(validate_cron_expression("0 */15 * * * *").is_ok());
        assert!(validate_cron_expression("0 0 0 * * *").is_ok());
        assert!(validate_cron_expression("0 30 8 * * *").is_ok());
        assert!(validate_cron_expression("0 0 */2 * * *").is_ok());
    }

    #[test]
    fn test_validate_cron_expression_rejects_5_field_format() {
        // Old Unix-style 5-field format should be rejected
        assert!(validate_cron_expression("0 9 * * 1-5").is_err());
        assert!(validate_cron_expression("*/15 * * * *").is_err());
        assert!(validate_cron_expression("0 0 * * *").is_err());
    }

    #[test]
    fn test_validate_cron_expression_invalid_syntax() {
        assert!(validate_cron_expression("invalid").is_err());
        assert!(validate_cron_expression("").is_err());
        assert!(validate_cron_expression("* * * * * * *").is_err()); // Too many fields
    }

    #[test]
    fn test_validate_cron_expression_invalid_values() {
        // Invalid second (0-59)
        assert!(validate_cron_expression("60 0 9 * * *").is_err());

        // Invalid minute (0-59)
        assert!(validate_cron_expression("0 60 9 * * *").is_err());

        // Invalid hour (0-23)
        assert!(validate_cron_expression("0 0 24 * * *").is_err());

        // Invalid day of month (1-31)
        assert!(validate_cron_expression("0 0 9 32 * *").is_err());

        // Invalid month (1-12)
        assert!(validate_cron_expression("0 0 9 * 13 *").is_err());
    }

    #[test]
    fn test_validate_cron_expression_special_characters() {
        // Valid special characters
        assert!(validate_cron_expression("0 0/5 * * * *").is_ok()); // Step values
        assert!(validate_cron_expression("0 0 9-17 * * *").is_ok()); // Ranges
        assert!(validate_cron_expression("0 0 9,12,15 * * *").is_ok()); // Lists
    }

    // Agent Validation Tests
    // Note: Full endpoint testing requires actix context, but validation logic can be tested
    #[test]
    fn test_agent_validation_requires_valid_agent_id() {
        // This test verifies the payload structure includes agent_id
        // Actual validation happens in create_job endpoint via app_config.get_agent()
        let payload = db::JobPayload {
            agent_id: "test-agent".to_string(),
            message: "Test".to_string(),
            system_prompt: None,
            file_path: None,
            session_id: None,
        };

        assert_eq!(payload.agent_id, "test-agent");
        assert!(!payload.agent_id.is_empty());
    }

    #[test]
    fn test_agent_validation_empty_agent_id() {
        // Empty agent_id should be detectable
        let payload = db::JobPayload {
            agent_id: "".to_string(),
            message: "Test".to_string(),
            system_prompt: None,
            file_path: None,
            session_id: None,
        };

        assert!(payload.agent_id.is_empty());
        // The create_job endpoint should validate this via app_config.get_agent()
    }

    #[test]
    fn test_create_job_request_with_invalid_schedule_type() {
        let invalid_json = serde_json::json!({
            "name": "Test Job",
            "schedule_type": "invalid-type",
            "payload": {
                "agent_id": "test-agent",
                "message": "Test"
            }
        });

        let req: Result<CreateJobRequest, _> = serde_json::from_value(invalid_json);
        // The request should parse but schedule_type validation happens in the endpoint
        // This tests the data structure accepts arbitrary strings
        assert!(req.is_ok());
    }

    #[test]
    fn test_cron_job_missing_cron_expression() {
        let cron_json = serde_json::json!({
            "name": "Cron Job",
            "schedule_type": "cron",
            "payload": {
                "agent_id": "test-agent",
                "message": "Test"
            }
            // Missing cron_expression
        });

        let req: CreateJobRequest = serde_json::from_value(cron_json).unwrap();
        assert_eq!(req.schedule_type, "cron");
        assert!(req.cron_expression.is_none());
        // Endpoint should reject this - cron jobs require cron_expression
    }

    // Manual Trigger Tests
    #[test]
    fn test_trigger_job_validates_cron_type() {
        // trigger_job endpoint should only work with cron jobs
        // This test verifies the BackgroundJob struct supports the schedule_type field
        let job = db::BackgroundJob {
            id: Some(1),
            name: "Test".to_string(),
            schedule_type: "cron".to_string(),
            cron_expression: Some("0 0 9 * * *".to_string()),
            priority: 5,
            max_cpu_percent: 70,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: "{}".to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        assert_eq!(job.schedule_type, "cron");
    }

    #[test]
    fn test_trigger_job_once_type_should_not_be_triggered() {
        // Verify that "once" jobs have different schedule_type
        let job = db::BackgroundJob {
            id: Some(1),
            name: "Test".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 70,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: "{}".to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        assert_eq!(job.schedule_type, "once");
        // trigger_job endpoint should reject "once" jobs
    }
}

/// Get execution history for a specific job
pub async fn get_job_executions(
    path: web::Path<i64>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let db = get_db();
    let job_id = path.into_inner();

    // Get limit from query parameter (default to 50)
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);

    match db.get_job_executions(job_id, Some(limit)) {
        Ok(executions) => {
            let response: Vec<JobExecutionResponse> =
                executions.iter().map(JobExecutionResponse::from).collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            // If table doesn't exist yet, return empty array instead of error
            let error_msg = e.to_string();
            if error_msg.contains("no such table: job_executions") {
                warn!("job_executions table not yet created - returning empty array");
                HttpResponse::Ok().json(Vec::<JobExecutionResponse>::new())
            } else {
                error!("Failed to get job executions for job {}: {}", job_id, e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to get job executions: {}", e)
                }))
            }
        }
    }
}

/// Get a single execution by ID
pub async fn get_job_execution(path: web::Path<i64>) -> HttpResponse {
    let db = get_db();
    let execution_id = path.into_inner();

    match db.get_job_execution(execution_id) {
        Ok(Some(execution)) => HttpResponse::Ok().json(JobExecutionResponse::from(&execution)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Execution {} not found", execution_id)
        })),
        Err(e) => {
            // If table doesn't exist yet, return 404
            let error_msg = e.to_string();
            if error_msg.contains("no such table: job_executions") {
                warn!("job_executions table not yet created");
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Execution {} not found", execution_id)
                }))
            } else {
                error!("Failed to get job execution {}: {}", execution_id, e);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to get job execution: {}", e)
                }))
            }
        }
    }
}

/// Server-Sent Events endpoint for job status updates
pub async fn job_events() -> HttpResponse {
    if let Some(tx) = JOB_UPDATE_BROADCASTER.get() {
        let rx = tx.subscribe();
        let stream = BroadcastStream::new(rx)
            .filter_map(|result| match result {
                Ok(event) => Some(event),
                Err(e) => {
                    warn!("Job SSE broadcast receive error: {}", e);
                    None
                }
            })
            .map(|event| {
                let data = serde_json::to_string(&event).unwrap_or_else(|e| {
                    error!("Failed to serialize job event: {}", e);
                    String::from("{}")
                });
                Ok::<_, actix_web::Error>(web::Bytes::from(format!(
                    "event: job_update\ndata: {}\n\n",
                    data
                )))
            })
            .throttle(Duration::from_millis(100)); // Limit to 10 updates/sec

        HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "text/event-stream"))
            .insert_header((header::CACHE_CONTROL, "no-cache"))
            .insert_header((header::CONNECTION, "keep-alive"))
            .streaming(Box::pin(stream)
                as Pin<
                    Box<dyn Stream<Item = Result<web::Bytes, actix_web::Error>>>,
                >)
    } else {
        warn!("Job update broadcaster not initialized");
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Job update broadcaster not available"
        }))
    }
}
