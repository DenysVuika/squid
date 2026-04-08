use chrono::Utc;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::{Semaphore, broadcast, mpsc};

use crate::{config, db, llm, session, template};

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
}

/// Job scheduler wrapper
pub struct JobScheduler {
    scheduler: tokio_cron_scheduler::JobScheduler,
    tx: mpsc::Sender<JobExecutionRequest>,
}

impl JobScheduler {
    /// Create a new job scheduler
    pub async fn new(
        tx: mpsc::Sender<JobExecutionRequest>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = tokio_cron_scheduler::JobScheduler::new().await?;

        Ok(Self { scheduler, tx })
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.scheduler.start().await?;
        Ok(())
    }

    /// Stop the scheduler
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.scheduler.shutdown().await?;
        Ok(())
    }

    /// Get the inner scheduler
    pub fn inner(&self) -> &tokio_cron_scheduler::JobScheduler {
        &self.scheduler
    }

    /// Get the job sender channel
    pub fn sender(&self) -> &mpsc::Sender<JobExecutionRequest> {
        &self.tx
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
) -> Result<String, Box<dyn std::error::Error>> {
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
            db.update_job_result(job_id, "failed", None, Some(&error_msg))?;

            let _ = sse_tx.send(JobStatusEvent {
                job_id,
                job_name: job_name.clone(),
                status: "failed".to_string(),
                result: None,
                error: Some(error_msg.clone()),
                timestamp: Utc::now().timestamp(),
            });

            return Err(error_msg.into());
        }
    };

    // Determine the system prompt
    let system_prompt = if let Some(custom_prompt) = &job_payload.system_prompt {
        custom_prompt.clone()
    } else if let Some(prompt) = &agent.prompt {
        // Use agent's prompt if available
        let combined = llm::combine_prompts(&prompt);
        combined
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

    // Create session
    let session_id = job_payload
        .session_id
        .clone()
        .unwrap_or_else(|| format!("job-{}-{}", job_payload.agent_id, Utc::now().timestamp()));

    let mut chat_session = session::ChatSession::new();
    chat_session.id = session_id.clone();
    chat_session.agent_id = Some(job_payload.agent_id.clone());

    // Read file content if specified
    let file_content = if let Some(file_path) = &job_payload.file_path {
        match std::fs::read_to_string(file_path) {
            Ok(content) => Some(content),
            Err(e) => {
                warn!("Failed to read file {}: {}", file_path, e);
                None
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

    // Execute the LLM call (non-streaming for background jobs)
    match llm::ask_llm(query_params).await {
        Ok(response) => {
            info!("Job {} completed successfully", job_id);

            // Save the result
            let result_json = serde_json::to_string(&json!({
                "session_id": session_id,
                "response_preview": &response[..response.len().min(500)],
            }))
            .unwrap_or_default();

            db.complete_job(job_id, &req.schedule_type, Some(&result_json), None)?;

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
        Err(e) => {
            let error_msg = e.to_string();
            error!("Job {} failed: {}", job_id, error_msg);

            // Increment retries
            db.increment_job_retries(job_id)?;

            // Check if we should retry
            let job_after = db.get_job_by_id(job_id)?;
            if let Some(job_after) = job_after {
                if job_after.retries < req.max_retries {
                    info!(
                        "Retrying job {} (attempt {}/{})",
                        job_id, job_after.retries, req.max_retries
                    );
                    db.update_job_status(job_id, "pending")?;

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
                } else {
                    db.update_job_result(job_id, "failed", None, Some(&error_msg))?;

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

            Err(error_msg.into())
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
        let job_req_clone = job_req.clone();

        tokio::spawn(async move {
            // Acquire permit inside the spawned task so it has the right lifetime
            let _permit = semaphore_clone.acquire().await.unwrap();

            match execute_job_from_request(&job_req_clone, db_clone, config_clone, sse_tx_clone)
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

            let cron_job = tokio_cron_scheduler::Job::new_async_tz(
                &cron_expr,
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
            )?;

            scheduler.inner().add(cron_job).await?;
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

    // Create semaphore for concurrency control
    let semaphore = Arc::new(Semaphore::new(max_concurrent_jobs));

    // Create the scheduler
    let scheduler = JobScheduler::new(tx.clone()).await?;

    // Restore jobs from database
    restore_jobs(db.clone(), &scheduler, &tx).await?;

    // Start the scheduler
    scheduler.start().await?;

    // Spawn the worker
    tokio::spawn(job_worker(
        rx,
        semaphore,
        db.clone(),
        app_config.clone(),
        sse_tx.clone(),
    ));

    info!("Job scheduler started successfully");
    Ok(Arc::new(tokio::sync::Mutex::new(scheduler)))
}

use serde_json::json;
