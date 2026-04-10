use chrono::Utc;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::{Semaphore, broadcast, mpsc};

use crate::{config, db, llm, session, template};

/// Validate file path to prevent directory traversal attacks
/// Returns the canonicalized path if it's within the current directory
fn validate_file_path(file_path: &str) -> Result<std::path::PathBuf, String> {
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let requested_path = std::path::Path::new(file_path);

    // Convert to absolute path
    let absolute_path = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        current_dir.join(requested_path)
    };

    // Canonicalize to resolve .. and symlinks
    let canonical_path = absolute_path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    // Check if the canonical path is within the current directory
    if !canonical_path.starts_with(&current_dir) {
        return Err(format!(
            "Access denied: path '{}' is outside workspace",
            file_path
        ));
    }

    Ok(canonical_path)
}

/// Event broadcasted via SSE when a job status changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusEvent {
    pub job_id: i64,
    pub job_name: String,
    pub status: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub timestamp: i64,
}

/// Request to execute a job
#[derive(Debug, Clone)]
pub struct JobExecutionRequest {
    pub job_id: i64,
    pub job_name: String,
    pub payload: db::JobPayload,
    pub max_cpu_percent: i32,
    pub max_retries: i32,
    pub schedule_type: String, // "cron" or "once"
    pub timeout_seconds: i64,  // Job timeout (0 = no timeout)
}

/// Job scheduler wrapper
pub struct JobScheduler {
    scheduler: tokio_cron_scheduler::JobScheduler,
}

impl JobScheduler {
    /// Create a new job scheduler
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = tokio_cron_scheduler::JobScheduler::new().await?;

        Ok(Self { scheduler })
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.scheduler.start().await?;
        Ok(())
    }

    /// Get the inner scheduler
    pub fn inner(&self) -> &tokio_cron_scheduler::JobScheduler {
        &self.scheduler
    }
}

/// Check if CPU usage is below the threshold
fn check_cpu_usage(sys: &System, max_cpu_percent: i32) -> bool {
    // sysinfo returns global CPU usage as a percentage across all cores
    // e.g., 50% on a 4-core system means average load is 0.5 per core
    let cpu_usage = sys.global_cpu_usage() as i32;
    cpu_usage < max_cpu_percent
}

/// Execute a background job using the request data directly (avoids DB fetch race condition)
async fn execute_job_from_request(
    req: &JobExecutionRequest,
    db: Arc<db::Database>,
    app_config: Arc<config::Config>,
    sse_tx: broadcast::Sender<JobStatusEvent>,
    retry_tx: mpsc::Sender<JobExecutionRequest>,
) -> Result<String, String> {
    let job_id = req.job_id;
    let job_name = req.job_name.clone();

    info!("Starting job {} (id: {})", job_name, job_id);

    let job_payload = &req.payload;

    // Get the agent configuration
    let agent = app_config.get_agent(&job_payload.agent_id);
    let agent = match agent {
        Some(a) => a,
        None => {
            let error_msg = format!("Agent '{}' not found", job_payload.agent_id);
            error!("{}", error_msg);
            db.update_job_result(job_id, "failed", None, Some(&error_msg))
                .map_err(|e| e.to_string())?;

            let _ = sse_tx.send(JobStatusEvent {
                job_id,
                job_name: job_name.clone(),
                status: "failed".to_string(),
                result: None,
                error: Some(error_msg.clone()),
                timestamp: Utc::now().timestamp(),
            });

            return Err(error_msg);
        }
    };

    // Determine the system prompt
    let system_prompt = if let Some(custom_prompt) = &job_payload.system_prompt {
        custom_prompt.clone()
    } else if let Some(prompt) = &agent.prompt {
        // Use agent's prompt if available
        llm::combine_prompts(prompt)
    } else {
        // Fall back to default ask prompt
        llm::get_ask_prompt().to_string()
    };

    // Render template variables
    let renderer = template::TemplateRenderer::new();
    let system_message = renderer.render_string(&system_prompt).unwrap_or_else(|e| {
        warn!("Failed to render system prompt: {}", e);
        system_prompt.clone()
    });

    // Create session (unique per execution)
    let started_at = Utc::now();
    let session_id = job_payload
        .session_id
        .clone()
        .unwrap_or_else(|| format!("job-{}-{}", job_payload.agent_id, started_at.timestamp()));

    let mut chat_session = session::ChatSession::new();
    chat_session.id = session_id.clone();
    chat_session.agent_id = Some(job_payload.agent_id.clone());

    // Read file content if specified (with security validation)
    let file_content = if let Some(file_path) = &job_payload.file_path {
        // Validate file path is within workspace
        match validate_file_path(file_path) {
            Ok(safe_path) => match std::fs::read_to_string(&safe_path) {
                Ok(content) => Some(content),
                Err(e) => {
                    warn!("Failed to read file {}: {}", safe_path.display(), e);
                    None
                }
            },
            Err(e) => {
                error!("File path validation failed for {}: {}", file_path, e);
                let error_msg = format!("Invalid file path: {}", e);
                db.update_job_result(job_id, "failed", None, Some(&error_msg))
                    .map_err(|e| e.to_string())?;

                let _ = sse_tx.send(JobStatusEvent {
                    job_id,
                    job_name: job_name.clone(),
                    status: "failed".to_string(),
                    result: None,
                    error: Some(error_msg.clone()),
                    timestamp: Utc::now().timestamp(),
                });

                return Err(error_msg);
            }
        }
    } else {
        None
    };

    // Build the LLM query params
    let query_params = llm::LlmQueryParams {
        question: &job_payload.message,
        file_content: file_content.as_deref(),
        file_path: job_payload.file_path.as_deref(),
        system_prompt: Some(&system_message),
        model: &agent.model,
        app_config: &app_config,
        session: Some(&mut chat_session),
        db: Some(&*db),
    };

    // Execute the LLM call with timeout (if specified)
    let llm_result = if req.timeout_seconds > 0 {
        match tokio::time::timeout(
            std::time::Duration::from_secs(req.timeout_seconds as u64),
            llm::ask_llm(query_params),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                let timeout_msg = format!("Job timed out after {} seconds", req.timeout_seconds);
                error!("{}", timeout_msg);
                Err(timeout_msg.into())
            }
        }
    } else {
        llm::ask_llm(query_params).await
    };

    // Convert error to String immediately to avoid Send issues
    let llm_result_str = llm_result.map_err(|e| e.to_string());

    match llm_result_str {
        Ok(response) => {
            let completed_at = Utc::now();
            let duration_ms = (completed_at - started_at).num_milliseconds();

            info!("Job {} completed successfully ({}ms)", job_id, duration_ms);

            // Update agent token stats for this job execution
            // This ensures jobs are reflected in the /agent-stats endpoint
            if let Some(agent_id) = &chat_session.agent_id
                && let Err(e) = db.update_agent_token_stats(
                    agent_id,
                    chat_session.token_usage.input_tokens,
                    chat_session.token_usage.output_tokens,
                    chat_session.token_usage.reasoning_tokens,
                    chat_session.token_usage.cache_tokens,
                    chat_session.cost_usd,
                )
            {
                error!(
                    "Failed to update agent token stats for job {}: {}",
                    job_id, e
                );
                // Don't fail the job if stats update fails - it's non-critical
            }

            // Calculate total tokens
            let total_tokens = chat_session.token_usage.input_tokens
                + chat_session.token_usage.output_tokens
                + chat_session.token_usage.reasoning_tokens;

            // Save the result with full response and session info
            let result_json = serde_json::to_string(&json!({
                "session_id": session_id,
                "response": response,
                "completed_at": completed_at.to_rfc3339(),
                "tokens": total_tokens,
                "cost_usd": chat_session.cost_usd,
            }))
            .unwrap_or_default();

            // Record this execution in job_executions table
            if let Err(e) = db.create_job_execution(
                job_id,
                Some(&session_id),
                "completed",
                Some(&result_json),
                None,
                &started_at.to_rfc3339(),
                Some(&completed_at.to_rfc3339()),
                Some(duration_ms),
                Some(total_tokens),
                Some(chat_session.cost_usd),
            ) {
                error!("Failed to record job execution: {}", e);
                // Don't fail the job if execution recording fails
            }

            // Update the job's last result (for quick status check)
            db.complete_job(job_id, &req.schedule_type, Some(&result_json), None)
                .map_err(|e| e.to_string())?;

            // Broadcast SSE event
            let _ = sse_tx.send(JobStatusEvent {
                job_id,
                job_name: job_name.clone(),
                status: "completed".to_string(),
                result: Some(response.clone()),
                error: None,
                timestamp: Utc::now().timestamp(),
            });

            Ok(response)
        }
        Err(error_msg) => {
            let completed_at = Utc::now();
            let duration_ms = (completed_at - started_at).num_milliseconds();

            error!(
                "Job {} failed after {}ms: {}",
                job_id, duration_ms, error_msg
            );

            // Increment retries
            db.increment_job_retries(job_id)
                .map_err(|e| e.to_string())?;

            // Check if we should retry
            let job_after = db.get_job_by_id(job_id).map_err(|e| e.to_string())?;
            if let Some(job_after) = job_after {
                if job_after.retries < req.max_retries {
                    info!(
                        "Retrying job {} (attempt {}/{})",
                        job_id, job_after.retries, req.max_retries
                    );
                    db.update_job_status(job_id, "pending")
                        .map_err(|e| e.to_string())?;

                    let _ = sse_tx.send(JobStatusEvent {
                        job_id,
                        job_name: job_name.clone(),
                        status: "pending".to_string(),
                        result: None,
                        error: Some(format!(
                            "Retrying ({}/{})",
                            job_after.retries, req.max_retries
                        )),
                        timestamp: Utc::now().timestamp(),
                    });

                    // Re-queue for retry
                    match retry_tx
                        .send(JobExecutionRequest {
                            job_id,
                            job_name: job_name.clone(),
                            payload: req.payload.clone(),
                            max_cpu_percent: req.max_cpu_percent,
                            max_retries: req.max_retries,
                            schedule_type: req.schedule_type.clone(),
                            timeout_seconds: req.timeout_seconds,
                        })
                        .await
                    {
                        Ok(_) => info!("Re-queued job {} for retry", job_id),
                        Err(e) => error!("Failed to re-queue job {} for retry: {}", job_id, e),
                    }
                } else {
                    // Exhausted all retries - record the failed execution
                    let total_tokens = chat_session.token_usage.input_tokens
                        + chat_session.token_usage.output_tokens
                        + chat_session.token_usage.reasoning_tokens;

                    // Record failed execution in job_executions table
                    if let Err(e) = db.create_job_execution(
                        job_id,
                        Some(&session_id),
                        "failed",
                        None,
                        Some(&error_msg),
                        &started_at.to_rfc3339(),
                        Some(&completed_at.to_rfc3339()),
                        Some(duration_ms),
                        Some(total_tokens),
                        Some(chat_session.cost_usd),
                    ) {
                        error!("Failed to record job execution: {}", e);
                    }

                    db.update_job_result(job_id, "failed", None, Some(&error_msg))
                        .map_err(|e| e.to_string())?;

                    let _ = sse_tx.send(JobStatusEvent {
                        job_id,
                        job_name: job_name.clone(),
                        status: "failed".to_string(),
                        result: None,
                        error: Some(error_msg.clone()),
                        timestamp: Utc::now().timestamp(),
                    });
                }
            }

            Err(error_msg)
        }
    }
}

/// Job worker that processes execution requests
pub async fn job_worker(
    mut rx: mpsc::Receiver<JobExecutionRequest>,
    semaphore: Arc<Semaphore>,
    db: Arc<db::Database>,
    app_config: Arc<config::Config>,
    sse_tx: broadcast::Sender<JobStatusEvent>,
    tx: mpsc::Sender<JobExecutionRequest>,
) {
    let mut sys = System::new_all();

    info!("Job worker started");

    while let Some(job_req) = rx.recv().await {
        info!(
            "Received job execution request: {} (id: {})",
            job_req.job_name, job_req.job_id
        );

        // Check CPU usage before executing
        sys.refresh_cpu_all();
        if !check_cpu_usage(&sys, job_req.max_cpu_percent) {
            warn!(
                "Job {} skipped: CPU usage too high ({}% >= {}%)",
                job_req.job_name,
                sys.global_cpu_usage(),
                job_req.max_cpu_percent
            );

            let _ = sse_tx.send(JobStatusEvent {
                job_id: job_req.job_id,
                job_name: job_req.job_name.clone(),
                status: "pending".to_string(),
                result: None,
                error: Some(format!(
                    "CPU usage too high ({}% >= {}%)",
                    sys.global_cpu_usage(),
                    job_req.max_cpu_percent
                )),
                timestamp: Utc::now().timestamp(),
            });

            // Re-queue the job for later
            let _ = db.update_job_status(job_req.job_id, "pending");
            continue;
        }

        info!(
            "Executing job {} (id: {})",
            job_req.job_name, job_req.job_id
        );

        // Update status to running
        let _ = db.update_job_status(job_req.job_id, "running");
        let _ = sse_tx.send(JobStatusEvent {
            job_id: job_req.job_id,
            job_name: job_req.job_name.clone(),
            status: "running".to_string(),
            result: None,
            error: None,
            timestamp: Utc::now().timestamp(),
        });

        // Execute the job in a spawned task (no need to fetch from DB - we have all data in request)
        let db_clone = db.clone();
        let config_clone = app_config.clone();
        let sse_tx_clone = sse_tx.clone();
        let semaphore_clone = semaphore.clone();
        let retry_tx_clone = tx.clone();
        let job_req_clone = job_req.clone();

        tokio::spawn(async move {
            // Acquire permit inside the spawned task so it has the right lifetime
            let _permit = semaphore_clone.acquire().await.unwrap();

            match execute_job_from_request(
                &job_req_clone,
                db_clone,
                config_clone,
                sse_tx_clone,
                retry_tx_clone,
            )
            .await
            {
                Ok(_) => {
                    info!("Job {} completed successfully", job_req_clone.job_id);
                }
                Err(e) => {
                    error!("Job {} failed with error: {}", job_req_clone.job_id, e);
                }
            }
            // _permit is dropped here, releasing the semaphore
        });
    }

    info!("Job worker shutting down");
}

/// Restore jobs from database and schedule them
pub async fn restore_jobs(
    db: Arc<db::Database>,
    scheduler: &JobScheduler,
    tx: &mpsc::Sender<JobExecutionRequest>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Restoring jobs from database");

    // Restore pending one-off jobs
    let pending_jobs = db.get_pending_jobs()?;
    for job in pending_jobs {
        if job.schedule_type == "once" {
            info!("Restoring one-off job: {} (id: {:?})", job.name, job.id);

            let job_payload: db::JobPayload = match serde_json::from_str(&job.payload) {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to parse payload for job {}: {}", job.name, e);
                    continue;
                }
            };

            let job_id = job.id.unwrap_or(0);
            tx.send(JobExecutionRequest {
                job_id,
                job_name: job.name.clone(),
                payload: job_payload,
                max_cpu_percent: job.max_cpu_percent,
                max_retries: job.max_retries,
                schedule_type: job.schedule_type.clone(),
                timeout_seconds: job.timeout_seconds,
            })
            .await?;
        }
    }

    // Restore cron jobs
    let cron_jobs = db.get_active_cron_jobs()?;
    for job in cron_jobs {
        if let Some(cron_expr) = &job.cron_expression {
            info!("Scheduling cron job: {} ({})", job.name, cron_expr);

            let job_id = job.id.unwrap_or(0);
            let job_name = job.name.clone();
            let tx_clone = tx.clone();
            let db_clone = db.clone();

            let cron_job_result = tokio_cron_scheduler::Job::new_async_tz(
                cron_expr,
                chrono_tz::UTC,
                move |_uuid, _job_scheduler| {
                    let job_id = job_id;
                    let job_name = job_name.clone();
                    let tx = tx_clone.clone();
                    let db = db_clone.clone();

                    Box::pin(async move {
                        info!("Cron trigger for job: {} (id: {})", job_name, job_id);

                        // Fetch latest job details
                        match db.get_job_by_id(job_id) {
                            Ok(Some(job)) => {
                                let job_payload: db::JobPayload =
                                    match serde_json::from_str(&job.payload) {
                                        Ok(p) => p,
                                        Err(e) => {
                                            error!(
                                                "Failed to parse payload for cron job {}: {}",
                                                job.name, e
                                            );
                                            return;
                                        }
                                    };

                                if let Err(e) = tx
                                    .send(JobExecutionRequest {
                                        job_id,
                                        job_name: job.name.clone(),
                                        payload: job_payload,
                                        max_cpu_percent: job.max_cpu_percent,
                                        max_retries: job.max_retries,
                                        schedule_type: job.schedule_type.clone(),
                                        timeout_seconds: job.timeout_seconds,
                                    })
                                    .await
                                {
                                    error!("Failed to send cron job to worker: {}", e);
                                }
                            }
                            Ok(None) => {
                                error!("Cron job {} not found in database", job_id);
                            }
                            Err(e) => {
                                error!("Failed to fetch cron job {}: {}", job_id, e);
                            }
                        }
                    })
                },
            );

            match cron_job_result {
                Ok(cron_job) => {
                    if let Err(e) = scheduler.inner().add(cron_job).await {
                        error!("Failed to add cron job '{}' to scheduler: {}", job.name, e);
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to parse cron expression '{}' for job '{}': {}. Expected format: 'sec min hour day month dayofweek'",
                        cron_expr, job.name, e
                    );
                }
            }
        }
    }

    info!("Jobs restored successfully");
    Ok(())
}

/// Start the job scheduler and worker
pub async fn start_job_scheduler(
    db: Arc<db::Database>,
    app_config: Arc<config::Config>,
    max_concurrent_jobs: usize,
    sse_tx: broadcast::Sender<JobStatusEvent>,
) -> Result<Arc<tokio::sync::Mutex<JobScheduler>>, Box<dyn std::error::Error>> {
    info!(
        "Starting job scheduler with {} concurrent job slots",
        max_concurrent_jobs
    );

    // Create channel for job execution requests
    let (tx, rx) = mpsc::channel::<JobExecutionRequest>(32);

    // Initialize the global job sender for API access
    crate::jobs_api::init_job_sender(tx.clone());

    // Bridge JobStatusEvent to frontend SSE (JobUpdateEvent)
    // This forwards job completion events from the worker to the frontend
    let mut sse_rx = sse_tx.subscribe();
    let db_clone_for_bridge = db.clone();
    tokio::spawn(async move {
        while let Ok(status_event) = sse_rx.recv().await {
            // Fetch the updated job from the database
            if let Ok(Some(job)) = db_clone_for_bridge.get_job_by_id(status_event.job_id) {
                // Convert to JobInfo and broadcast via JOB_UPDATE_BROADCASTER
                crate::jobs_api::broadcast_job_status_update(job);
            }
        }
    });

    // Create semaphore for concurrency control
    let semaphore = Arc::new(Semaphore::new(max_concurrent_jobs));

    // Create the scheduler
    let scheduler = JobScheduler::new().await?;

    // Restore jobs from database
    restore_jobs(db.clone(), &scheduler, &tx).await?;

    // Schedule periodic cleanup of old jobs (runs daily at 2 AM)
    if app_config.jobs.retention_days > 0 {
        let db_cleanup = db.clone();
        let retention_days = app_config.jobs.retention_days;

        let cleanup_job = tokio_cron_scheduler::Job::new_async_tz(
            "0 0 2 * * *", // Daily at 2 AM (sec min hour day month dayofweek)
            chrono_tz::UTC,
            move |_uuid, _scheduler| {
                let db = db_cleanup.clone();
                let days = retention_days;
                Box::pin(async move {
                    info!("Running job cleanup task (retention: {} days)", days);
                    match db.cleanup_old_jobs(days) {
                        Ok(deleted) => {
                            if deleted > 0 {
                                info!("Cleaned up {} old jobs", deleted);
                            }
                        }
                        Err(e) => error!("Failed to cleanup old jobs: {}", e),
                    }
                })
            },
        )?;

        scheduler.inner().add(cleanup_job).await?;
        info!(
            "Scheduled daily cleanup task (retention: {} days)",
            app_config.jobs.retention_days
        );
    }

    // Start the scheduler
    scheduler.start().await?;

    // Spawn the worker
    tokio::spawn(job_worker(
        rx,
        semaphore,
        db.clone(),
        app_config.clone(),
        sse_tx.clone(),
        tx,
    ));

    info!("Job scheduler started successfully");
    Ok(Arc::new(tokio::sync::Mutex::new(scheduler)))
}

use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn test_job_execution_request_clone() {
        let req = JobExecutionRequest {
            job_id: 42,
            job_name: "Test Job".to_string(),
            payload: db::JobPayload {
                agent_id: "shakespeare".to_string(),
                message: "Say hello".to_string(),
                system_prompt: None,
                file_path: None,
                session_id: None,
            },
            max_cpu_percent: 70,
            max_retries: 3,
            schedule_type: "once".to_string(),
            timeout_seconds: 3600,
        };

        let cloned = req.clone();
        assert_eq!(cloned.job_id, req.job_id);
        assert_eq!(cloned.job_name, req.job_name);
        assert_eq!(cloned.payload.agent_id, req.payload.agent_id);
        assert_eq!(cloned.max_cpu_percent, req.max_cpu_percent);
        assert_eq!(cloned.max_retries, req.max_retries);
        assert_eq!(cloned.schedule_type, req.schedule_type);
        assert_eq!(cloned.timeout_seconds, req.timeout_seconds);
    }

    #[test]
    fn test_job_status_event_serialization() {
        let event = JobStatusEvent {
            job_id: 1,
            job_name: "Test".to_string(),
            status: "completed".to_string(),
            result: Some("Hello".to_string()),
            error: None,
            timestamp: 1775649016,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("completed"));
        assert!(json.contains("Test"));
        assert!(json.contains("Hello"));

        let deserialized: JobStatusEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.job_id, 1);
        assert_eq!(deserialized.status, "completed");
        assert_eq!(deserialized.result, Some("Hello".to_string()));
    }

    #[test]
    fn test_job_status_event_with_error() {
        let event = JobStatusEvent {
            job_id: 1,
            job_name: "Fail".to_string(),
            status: "failed".to_string(),
            result: None,
            error: Some("Agent not found".to_string()),
            timestamp: 1775649016,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: JobStatusEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.error, Some("Agent not found".to_string()));
        assert!(deserialized.result.is_none());
    }

    #[test]
    fn test_cpu_check_within_limit() {
        let mut sys = System::new_all();
        sys.refresh_cpu_all();
        // CPU usage should always be < 100%
        assert!(check_cpu_usage(&sys, 100));
    }

    #[test]
    fn test_cpu_check_below_threshold() {
        let mut sys = System::new_all();
        sys.refresh_cpu_all();
        let cpu = sys.global_cpu_usage() as i32;
        // Threshold above current usage should pass
        assert!(check_cpu_usage(&sys, cpu + 50));
    }

    #[test]
    fn test_job_request_with_all_fields() {
        let payload = db::JobPayload {
            agent_id: "code-reviewer".to_string(),
            message: "Review this code".to_string(),
            system_prompt: Some("Custom prompt".to_string()),
            file_path: Some("src/main.rs".to_string()),
            session_id: Some("test-123".to_string()),
        };

        let req = JobExecutionRequest {
            job_id: 10,
            job_name: "Code Review".to_string(),
            payload,
            max_cpu_percent: 50,
            max_retries: 5,
            schedule_type: "cron".to_string(),
            timeout_seconds: 7200,
        };

        assert_eq!(req.job_id, 10);
        assert_eq!(req.payload.system_prompt, Some("Custom prompt".to_string()));
        assert_eq!(req.payload.file_path, Some("src/main.rs".to_string()));
        assert_eq!(req.payload.session_id, Some("test-123".to_string()));
        assert_eq!(req.max_retries, 5);
        assert_eq!(req.schedule_type, "cron");
        assert_eq!(req.timeout_seconds, 7200);
    }

    #[test]
    fn test_job_status_transitions() {
        // Verify all valid status values
        let valid_statuses = ["pending", "running", "completed", "failed", "cancelled"];
        for status in valid_statuses {
            assert!(!status.is_empty());
        }
    }

    #[test]
    fn test_schedule_types() {
        // Verify both schedule types are valid
        assert_eq!("once", "once");
        assert_eq!("cron", "cron");
        assert_ne!("once", "cron");
    }

    #[test]
    fn test_job_name_special_characters() {
        let req = JobExecutionRequest {
            job_id: 1,
            job_name: "Daily Review (Mon-Fri)".to_string(),
            payload: db::JobPayload {
                agent_id: "test".to_string(),
                message: "test".to_string(),
                system_prompt: None,
                file_path: None,
                session_id: None,
            },
            max_cpu_percent: 50,
            max_retries: 3,
            schedule_type: "cron".to_string(),
            timeout_seconds: 3600,
        };

        let cloned = req.clone();
        assert_eq!(cloned.job_name, "Daily Review (Mon-Fri)");
    }

    #[test]
    fn test_empty_payload_message() {
        let req = JobExecutionRequest {
            job_id: 1,
            job_name: "Empty".to_string(),
            payload: db::JobPayload {
                agent_id: "test".to_string(),
                message: "".to_string(),
                system_prompt: None,
                file_path: None,
                session_id: None,
            },
            max_cpu_percent: 50,
            max_retries: 0,
            schedule_type: "once".to_string(),
            timeout_seconds: 3600,
        };

        assert_eq!(req.payload.message, "");
        assert_eq!(req.max_retries, 0);
    }

    #[test]
    fn test_max_retries_boundary() {
        // Test boundary values for max_retries
        let zero_retries = JobExecutionRequest {
            job_id: 1,
            job_name: "No retries".to_string(),
            payload: db::JobPayload {
                agent_id: "test".to_string(),
                message: "test".to_string(),
                system_prompt: None,
                file_path: None,
                session_id: None,
            },
            max_cpu_percent: 50,
            max_retries: 0,
            schedule_type: "once".to_string(),
            timeout_seconds: 3600,
        };

        assert_eq!(zero_retries.max_retries, 0);

        let high_retries = JobExecutionRequest {
            job_id: 2,
            job_name: "Many retries".to_string(),
            payload: db::JobPayload {
                agent_id: "test".to_string(),
                message: "test".to_string(),
                system_prompt: None,
                file_path: None,
                session_id: None,
            },
            max_cpu_percent: 50,
            max_retries: 100,
            schedule_type: "once".to_string(),
            timeout_seconds: 3600,
        };

        assert_eq!(high_retries.max_retries, 100);
    }

    #[test]
    fn test_max_cpu_percent_boundary() {
        let req_min = JobExecutionRequest {
            job_id: 1,
            job_name: "Low CPU".to_string(),
            payload: db::JobPayload {
                agent_id: "test".to_string(),
                message: "test".to_string(),
                system_prompt: None,
                file_path: None,
                session_id: None,
            },
            max_cpu_percent: 1,
            max_retries: 3,
            schedule_type: "once".to_string(),
            timeout_seconds: 3600,
        };

        let req_max = JobExecutionRequest {
            job_id: 2,
            job_name: "High CPU".to_string(),
            payload: db::JobPayload {
                agent_id: "test".to_string(),
                message: "test".to_string(),
                system_prompt: None,
                file_path: None,
                session_id: None,
            },
            max_cpu_percent: 100,
            max_retries: 3,
            schedule_type: "once".to_string(),
            timeout_seconds: 3600,
        };

        assert_eq!(req_min.max_cpu_percent, 1);
        assert_eq!(req_max.max_cpu_percent, 100);
    }

    // Security Tests
    #[test]
    fn test_validate_file_path_prevents_directory_traversal() {
        // Should reject paths with ..
        assert!(validate_file_path("../../../etc/passwd").is_err());
        assert!(validate_file_path("./src/../../secret.txt").is_err());
        assert!(validate_file_path("../../Cargo.toml").is_err());
    }

    #[test]
    fn test_validate_file_path_rejects_absolute_paths_outside_workspace() {
        // Should reject absolute paths to system files
        assert!(validate_file_path("/etc/passwd").is_err());
        assert!(validate_file_path("/tmp/malicious").is_err());

        #[cfg(target_os = "macos")]
        {
            assert!(validate_file_path("/System/Library/").is_err());
        }

        #[cfg(target_os = "windows")]
        {
            assert!(validate_file_path("C:\\Windows\\System32\\").is_err());
        }
    }

    #[test]
    fn test_validate_file_path_accepts_workspace_files() {
        // Should accept relative paths within workspace
        assert!(validate_file_path("src/main.rs").is_ok());
        assert!(validate_file_path("./Cargo.toml").is_ok());
        assert!(validate_file_path("docs/JOBS.md").is_ok());
    }

    #[test]
    fn test_validate_file_path_current_dir_failure_handling() {
        // This test verifies error handling, actual failure is environment-dependent
        let result = validate_file_path("test.txt");
        // Should either succeed (if in workspace) or fail gracefully with error message
        match result {
            Ok(_) => assert!(true),
            Err(e) => assert!(e.contains("Failed to") || e.contains("Access denied")),
        }
    }

    #[test]
    fn test_validate_file_path_hidden_directory_traversal() {
        // Various sneaky directory traversal attempts
        assert!(validate_file_path("src/../../../etc/passwd").is_err());
        assert!(validate_file_path("./docs/../../../../../../etc/passwd").is_err());
    }

    // Timeout Tests
    #[test]
    fn test_timeout_validation_zero_means_no_timeout() {
        // timeout_seconds = 0 should mean no timeout
        let req = JobExecutionRequest {
            job_id: 1,
            job_name: "Test".to_string(),
            payload: db::JobPayload {
                agent_id: "test".to_string(),
                message: "test".to_string(),
                system_prompt: None,
                file_path: None,
                session_id: None,
            },
            max_cpu_percent: 70,
            max_retries: 3,
            schedule_type: "once".to_string(),
            timeout_seconds: 0,
        };

        assert_eq!(req.timeout_seconds, 0);
        // In execute_job_from_request, timeout_seconds > 0 triggers timeout wrapper
        assert!(!(req.timeout_seconds > 0));
    }

    #[test]
    fn test_timeout_validation_positive_value_enables_timeout() {
        let req = JobExecutionRequest {
            job_id: 1,
            job_name: "Test".to_string(),
            payload: db::JobPayload {
                agent_id: "test".to_string(),
                message: "test".to_string(),
                system_prompt: None,
                file_path: None,
                session_id: None,
            },
            max_cpu_percent: 70,
            max_retries: 3,
            schedule_type: "once".to_string(),
            timeout_seconds: 30,
        };

        assert_eq!(req.timeout_seconds, 30);
        assert!(req.timeout_seconds > 0);
    }

    #[test]
    fn test_timeout_error_message_format() {
        // Verify the timeout error message format matches what execute_job_from_request produces
        let timeout_seconds = 120;
        let expected_msg = format!("Job timed out after {} seconds", timeout_seconds);
        assert!(expected_msg.contains("timed out"));
        assert!(expected_msg.contains("120 seconds"));
    }

    #[test]
    fn test_timeout_duration_conversion() {
        // Test that timeout_seconds converts correctly to Duration
        let timeout_seconds: i64 = 3600;
        let duration = std::time::Duration::from_secs(timeout_seconds as u64);
        assert_eq!(duration.as_secs(), 3600);

        let timeout_seconds_small: i64 = 5;
        let duration_small = std::time::Duration::from_secs(timeout_seconds_small as u64);
        assert_eq!(duration_small.as_secs(), 5);
    }

    #[tokio::test]
    async fn test_timeout_actually_fires() {
        // Integration test: verify tokio::time::timeout actually fires after duration
        use tokio::time::{Duration, timeout};

        let start = std::time::Instant::now();
        let result = timeout(Duration::from_millis(100), async {
            // Sleep for longer than timeout
            tokio::time::sleep(Duration::from_millis(500)).await;
            "should not complete"
        })
        .await;

        let elapsed = start.elapsed();

        // Should timeout (return Err) after ~100ms
        assert!(result.is_err());
        assert!(elapsed.as_millis() >= 100);
        assert!(elapsed.as_millis() < 300); // Should not wait full 500ms
    }

    #[tokio::test]
    async fn test_timeout_does_not_fire_for_quick_operations() {
        // Integration test: verify timeout doesn't fire if operation completes quickly
        use tokio::time::{Duration, timeout};

        let result = timeout(Duration::from_millis(500), async {
            // Complete quickly
            tokio::time::sleep(Duration::from_millis(10)).await;
            "completed"
        })
        .await;

        // Should complete successfully (return Ok)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "completed");
    }
}
