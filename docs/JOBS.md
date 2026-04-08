# Background Jobs

Schedule recurring AI tasks and manage one-off background jobs with resource control.

## Overview

Squid's background jobs system enables:
- **Cron scheduling** - Run jobs on recurring schedules (e.g., hourly code reviews)
- **One-off tasks** - Queue immediate background jobs
- **Resource control** - Limit CPU usage and concurrent job executions
- **Real-time updates** - Monitor job status via SSE streaming
- **Persistent storage** - Jobs survive server restarts

## Configuration

Enable background jobs in `squid.config.json`:

```json
{
  "jobs": {
    "enabled": true,
    "max_concurrent_jobs": 2,
    "max_cpu_percent": 70,
    "default_retries": 3
  }
}
```

| Setting | Default | Description |
|---------|---------|-------------|
| `enabled` | `false` | Enable background job scheduler |
| `max_concurrent_jobs` | `2` | Maximum jobs running simultaneously |
| `max_cpu_percent` | `70` | CPU threshold before jobs are paused |
| `default_retries` | `3` | Retry attempts for failed jobs |
| `default_timeout_seconds` | `3600` | Default timeout for jobs (1 hour) |
| `retention_days` | `30` | Days to keep completed/failed jobs (0 = forever) |

**Environment variable overrides:**
- `SQUID_JOBS_ENABLED`
- `SQUID_MAX_CONCURRENT_JOBS`
- `SQUID_JOBS_MAX_CPU_PERCENT`
- `SQUID_JOBS_DEFAULT_RETRIES`

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/jobs` | GET | List all jobs |
| `/api/jobs` | POST | Create a new job |
| `/api/jobs/{id}` | GET | Get job details |
| `/api/jobs/{id}` | DELETE | Delete a job permanently |
| `/api/jobs/{id}/pause` | POST | Pause a cron job |
| `/api/jobs/{id}/resume` | POST | Resume a paused cron job |
| `/api/jobs/{id}/trigger` | POST | Manually trigger a cron job |

## Creating Jobs

### Cron Job

Run a job on a recurring schedule using standard cron expressions:

```bash
curl -X POST http://localhost:8080/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Daily Code Review",
    "schedule_type": "cron",
    "cron_expression": "0 0 9 * * Mon-Fri",
    "priority": 8,
    "max_cpu_percent": 60,
    "payload": {
      "agent_id": "code-reviewer",
      "message": "Review all changed files in the workspace",
      "file_path": "src/main.rs"
    }
  }'
```

### One-Off Task

Queue an immediate background task:

```bash
curl -X POST http://localhost:8080/api/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Generate Documentation",
    "schedule_type": "once",
    "priority": 5,
    "payload": {
      "agent_id": "general-assistant",
      "message": "Generate comprehensive documentation for this codebase"
    }
  }'
```

## Job Payload

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `agent_id` | string | Yes | Agent to execute the job |
| `message` | string | Yes | The prompt/question for the agent |
| `system_prompt` | string | No | Custom system prompt (overrides agent default) |
| `file_path` | string | No | File to provide as context (must be within workspace) |
| `session_id` | string | No | Session to save results (auto-generated if omitted) |

## Job Creation Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | Yes | - | Job name |
| `schedule_type` | string | Yes | - | Either "cron" or "once" |
| `cron_expression` | string | Conditional | - | Required for cron jobs (e.g., "0 0 9 * * Mon-Fri") |
| `priority` | number | No | 5 | Priority 0-10 (higher = runs first) |
| `max_cpu_percent` | number | No | 50 | Skip execution if CPU exceeds this % |
| `max_retries` | number | No | 3 | Retry attempts on failure |
| `timeout_seconds` | number | No | 3600 | Timeout in seconds (0 = no timeout) |
| `payload` | object | Yes | - | Job payload (see above) |

## Job Response

```json
{
  "id": 1,
  "name": "Daily Code Review",
  "schedule_type": "cron",
  "cron_expression": "0 0 9 * * Mon-Fri",
  "priority": 8,
  "max_cpu_percent": 60,
  "status": "completed",
  "last_run": "2026-04-08T09:00:00Z",
  "next_run": "2026-04-09T09:00:00Z",
  "retries": 0,
  "max_retries": 3,
  "payload": {
    "agent_id": "code-reviewer",
    "message": "Review all changed files"
  },
  "result": {
    "session_id": "job-code-reviewer-1234567890",
    "response": "The code review has been completed...",
    "completed_at": "2026-04-08T09:05:23Z"
  },
  "error_message": null,
  "is_active": true,
  "timeout_seconds": 3600
}
```

## Job Statuses

| Status | Description |
|--------|-------------|
| `pending` | Waiting to be executed |
| `running` | Currently executing |
| `completed` | Finished successfully |
| `failed` | Execution failed (may retry) |
| `cancelled` | Manually cancelled by user |

## Real-Time Updates

Job status changes are broadcast via Server-Sent Events. Connect to the SSE stream to receive updates:

```javascript
const eventSource = new EventSource('/api/sse');
eventSource.addEventListener('job_status', (event) => {
  const { job_id, job_name, status, result, error, timestamp } = JSON.parse(event.data);
  console.log(`Job ${job_name} (${job_id}): ${status}`);
});
```

## Cron Expression Format

Extended cron syntax with seconds (6 fields):

```
┌───────────── second (0 - 59)
│ ┌───────────── minute (0 - 59)
│ │ ┌───────────── hour (0 - 23)
│ │ │ ┌───────────── day of month (1 - 31)
│ │ │ │ ┌───────────── month (1 - 12)
│ │ │ │ │ ┌───────────── day of week (0 - 6, Sunday = 0)
│ │ │ │ │ │
* * * * * *
```

**Common patterns:**
- `0 0 * * * *` - Every hour (at the top of the hour)
- `0 0 */2 * * *` - Every 2 hours
- `0 0 9 * * Mon-Fri` - Weekdays at 9 AM
- `0 0 0 * * Sun` - Every Sunday at midnight
- `0 */15 * * * *` - Every 15 minutes
- `0 30 8 * * *` - Every day at 8:30 AM

## Resource Control

The job scheduler prevents resource exhaustion:

1. **Concurrency limit** - `max_concurrent_jobs` controls parallel executions
2. **CPU monitoring** - Jobs pause if CPU usage exceeds `max_cpu_percent`
3. **Priority queue** - Higher priority jobs execute first (0-10 scale)
4. **Retry logic** - Failed jobs retry automatically up to `max_retries`
5. **Timeout protection** - Jobs are terminated if they exceed `timeout_seconds`
6. **Auto cleanup** - Old completed/failed jobs are deleted after `retention_days`

## Database Schema

Jobs are stored in SQLite (`background_jobs` table) and persist across restarts. The table is created automatically via migration `014_background_jobs.sql`.

## Example Workflows

### Automated Daily Code Review

```json
{
  "name": "Daily Review",
  "schedule_type": "cron",
  "cron_expression": "0 0 9 * * Mon-Fri",
  "payload": {
    "agent_id": "code-reviewer",
    "message": "Review recent changes and identify potential issues"
  }
}
```

### Periodic Documentation Updates

```json
{
  "name": "Update Docs",
  "schedule_type": "cron",
  "cron_expression": "0 0 0 * * *",
  "payload": {
    "agent_id": "general-assistant",
    "message": "Update API documentation based on recent code changes"
  }
}
```

### One-Time Migration Task

```json
{
  "name": "Database Migration Analysis",
  "schedule_type": "once",
  "priority": 10,
  "payload": {
    "agent_id": "general-assistant",
    "message": "Analyze database schema and suggest optimizations",
    "file_path": "migrations/latest.sql"
  }
}
```

## Managing Jobs

### Pause a Cron Job

Temporarily stop a cron job from running (it won't trigger on schedule):

```bash
curl -X POST http://localhost:8080/api/jobs/1/pause
```

### Resume a Cron Job

Resume a paused cron job:

```bash
curl -X POST http://localhost:8080/api/jobs/1/resume
```

### Manually Trigger a Cron Job

Run a cron job immediately (outside its normal schedule):

```bash
curl -X POST http://localhost:8080/api/jobs/1/trigger
```

### Delete a Job

Permanently delete a job:

```bash
curl -X DELETE http://localhost:8080/api/jobs/1
```
