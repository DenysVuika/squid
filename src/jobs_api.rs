use actix_web::{HttpResponse, web};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::db;
use crate::jobs::JobExecutionRequest;

/// Global sender channel for one-off jobs
/// Populated by the job scheduler at startup
static JOB_SENDER: std::sync::OnceLock<mpsc::Sender<JobExecutionRequest>> =
    std::sync::OnceLock::new();

/// Global database path (set at server startup from config)
static DB_PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Cached Database instance (initialized once at startup, shared across all API calls)
static DB_INSTANCE: std::sync::OnceLock<Arc<db::Database>> = std::sync::OnceLock::new();

/// Initialize the global job sender (called by scheduler at startup)
pub fn init_job_sender(tx: mpsc::Sender<JobExecutionRequest>) {
    let _ = JOB_SENDER.set(tx);
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
    pub payload: db::JobPayload,
}

#[derive(Debug, Serialize)]
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
    pub result: Option<String>,
    pub error_message: Option<String>,
    pub is_active: bool,
}

impl From<&db::BackgroundJob> for JobResponse {
    fn from(job: &db::BackgroundJob) -> Self {
        let payload: serde_json::Value =
            serde_json::from_str(&job.payload).unwrap_or(serde_json::json!({}));

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
            result: job.result.clone(),
            error_message: job.error_message.clone(),
            is_active: job.is_active,
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

/// Create a new background job
pub async fn create_job(req: web::Json<CreateJobRequest>) -> HttpResponse {
    let db = get_db();
    let req = req.into_inner();

    // Validate schedule type
    if req.schedule_type != "cron" && req.schedule_type != "once" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "schedule_type must be 'cron' or 'once'"
        }));
    }

    // Validate cron expression for cron jobs
    if req.schedule_type == "cron" && req.cron_expression.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "cron_expression is required for cron jobs"
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
                    };

                    match tx.try_send(exec_req) {
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
                    HttpResponse::Created().json(JobResponse::from(&created_job))
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
}
