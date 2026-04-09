import React, { useEffect, useMemo } from 'react';
import { useJobStore } from '@/stores/job-store';
import { fetchJob, fetchJobExecutions, pauseJob, resumeJob, triggerJob, deleteJob, cancelJob, type JobInfo, type JobExecution } from '@/lib/chat-api';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Play, Pause, Trash, Clock, AlertCircle, CheckCircle, XCircle, Loader2, Ban, ExternalLink } from 'lucide-react';
import { toast } from 'sonner';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useNavigate, useParams } from 'react-router-dom';

const JobDetails = () => {
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const { selectedJob, setSelectedJob, loadJobs } = useJobStore();
  const [job, setJob] = React.useState<JobInfo | null>(null);
  const [executions, setExecutions] = React.useState<JobExecution[]>([]);
  const [isLoading, setIsLoading] = React.useState(false);
  const [isActionLoading, setIsActionLoading] = React.useState(false);

  // Sync URL param with store on mount
  useEffect(() => {
    if (id) {
      const jobId = parseInt(id, 10);
      if (!isNaN(jobId) && jobId !== selectedJob) {
        setSelectedJob(jobId);
      }
    }
  }, [id, selectedJob, setSelectedJob]);

  // Load job details when selectedJob changes
  useEffect(() => {
    if (selectedJob) {
      setIsLoading(true);

      // Load job details first
      fetchJob('', selectedJob)
        .then((jobData) => {
          setJob(jobData);

          // Try to load executions history (optional - may not exist yet)
          return fetchJobExecutions('', selectedJob, 10);
        })
        .then((executionsData) => {
          setExecutions(executionsData);
        })
        .catch((error) => {
          // If just executions fail, log but don't show error
          if (error.message?.includes('executions')) {
            console.log('Execution history not available yet (table may not exist)');
          } else {
            console.error('Failed to load job:', error);
            toast.error('Failed to load job details');
          }
        })
        .finally(() => {
          setIsLoading(false);
        });
    } else {
      setJob(null);
      setExecutions([]);
    }
  }, [selectedJob]);

  const handlePause = async () => {
    if (!job) return;
    setIsActionLoading(true);
    try {
      const success = await pauseJob('', job.id);
      if (success) {
        toast.success('Job paused');
        await loadJobs();
        // Reload job details
        const updated = await fetchJob('', job.id);
        setJob(updated);
      } else {
        toast.error('Failed to pause job');
      }
    } catch (error) {
      console.error('Failed to pause job:', error);
      toast.error('Failed to pause job');
    } finally {
      setIsActionLoading(false);
    }
  };

  const handleResume = async () => {
    if (!job) return;
    setIsActionLoading(true);
    try {
      const success = await resumeJob('', job.id);
      if (success) {
        toast.success('Job resumed');
        await loadJobs();
        // Reload job details
        const updated = await fetchJob('', job.id);
        setJob(updated);
      } else {
        toast.error('Failed to resume job');
      }
    } catch (error) {
      console.error('Failed to resume job:', error);
      toast.error('Failed to resume job');
    } finally {
      setIsActionLoading(false);
    }
  };

  const handleTrigger = async () => {
    if (!job) return;
    setIsActionLoading(true);
    try {
      const success = await triggerJob('', job.id);
      if (success) {
        toast.success('Job triggered');
        await loadJobs();
        // Reload job details
        const updated = await fetchJob('', job.id);
        setJob(updated);
      } else {
        toast.error('Failed to trigger job');
      }
    } catch (error) {
      console.error('Failed to trigger job:', error);
      toast.error('Failed to trigger job');
    } finally {
      setIsActionLoading(false);
    }
  };

  const handleCancel = async () => {
    if (!job) return;
    if (!confirm(`Are you sure you want to cancel job "${job.name}"? This will stop the job but keep it in the database.`)) {
      return;
    }
    setIsActionLoading(true);
    try {
      const success = await cancelJob('', job.id);
      if (success) {
        toast.success('Job cancelled');
        await loadJobs();
        // Reload job details
        const updated = await fetchJob('', job.id);
        setJob(updated);
      } else {
        toast.error('Failed to cancel job');
      }
    } catch (error) {
      console.error('Failed to cancel job:', error);
      toast.error('Failed to cancel job');
    } finally {
      setIsActionLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!job) return;
    if (!confirm(`Are you sure you want to delete job "${job.name}"?`)) {
      return;
    }
    setIsActionLoading(true);
    try {
      const success = await deleteJob('', job.id);
      if (success) {
        toast.success('Job deleted');
        await loadJobs();
        // Clear selection
        setJob(null);
      } else {
        toast.error('Failed to delete job');
      }
    } catch (error) {
      console.error('Failed to delete job:', error);
      toast.error('Failed to delete job');
    } finally {
      setIsActionLoading(false);
    }
  };

  const statusIcon = useMemo(() => {
    if (!job) return null;
    switch (job.status) {
      case 'running':
        return <Loader2 className="h-5 w-5 text-blue-500 animate-spin" />;
      case 'completed':
        return <CheckCircle className="h-5 w-5 text-green-500" />;
      case 'failed':
        return <XCircle className="h-5 w-5 text-red-500" />;
      case 'pending':
        return <Clock className="h-5 w-5 text-yellow-500" />;
      default:
        return <AlertCircle className="h-5 w-5 text-gray-500" />;
    }
  }, [job]);

  const statusBadge = useMemo(() => {
    if (!job) return null;
    const variant =
      job.status === 'running'
        ? 'default'
        : job.status === 'completed'
          ? 'default'
          : job.status === 'failed'
            ? 'destructive'
            : job.status === 'pending'
              ? 'secondary'
              : 'outline';

    return (
      <Badge variant={variant} className="gap-1">
        {statusIcon}
        {job.status}
      </Badge>
    );
  }, [job, statusIcon]);

  if (!selectedJob) {
    return (
      <div className="flex flex-1 items-center justify-center h-full">
        <div className="text-center text-muted-foreground">
          <Clock className="h-16 w-16 mx-auto mb-4 opacity-50" />
          <p className="text-lg">Select a job to view details</p>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex flex-1 items-center justify-center h-full">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!job) {
    return (
      <div className="flex flex-1 items-center justify-center h-full">
        <div className="text-center text-muted-foreground">
          <AlertCircle className="h-16 w-16 mx-auto mb-4 opacity-50" />
          <p className="text-lg">Job not found</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="border-b bg-white dark:bg-gray-950 px-6 py-4">
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <div className="flex items-center gap-3 mb-2">
              <h1 className="text-2xl font-bold">{job.name}</h1>
              {statusBadge}
              {job.schedule_type === 'cron' && (
                job.is_active ? (
                  <Badge variant="outline" className="gap-1">
                    <span className="text-green-500">●</span>
                    Active
                  </Badge>
                ) : (
                  <Badge variant="outline" className="gap-1">
                    <span className="text-gray-400">○</span>
                    Paused
                  </Badge>
                )
              )}
            </div>
            <p className="text-sm text-muted-foreground">Job ID: {job.id}</p>
          </div>
          <div className="flex gap-2">
            {job.schedule_type === 'cron' && (
              <>
                <Button variant="outline" size="sm" onClick={handleTrigger} disabled={isActionLoading}>
                  <Play className="h-4 w-4 mr-2" />
                  Trigger Now
                </Button>
                {job.is_active ? (
                  <Button variant="outline" size="sm" onClick={handlePause} disabled={isActionLoading}>
                    <Pause className="h-4 w-4 mr-2" />
                    Pause
                  </Button>
                ) : (
                  <Button variant="outline" size="sm" onClick={handleResume} disabled={isActionLoading}>
                    <Play className="h-4 w-4 mr-2" />
                    Resume
                  </Button>
                )}
              </>
            )}
            {job.status === 'running' && (
              <Button variant="outline" size="sm" onClick={handleCancel} disabled={isActionLoading}>
                <Ban className="h-4 w-4 mr-2" />
                Cancel
              </Button>
            )}
            <Button variant="destructive" size="sm" onClick={handleDelete} disabled={isActionLoading}>
              <Trash className="h-4 w-4 mr-2" />
              Delete
            </Button>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-4xl mx-auto space-y-6">
          {/* Configuration */}
          <Card>
            <CardHeader>
              <CardTitle>Configuration</CardTitle>
              <CardDescription>Job configuration and settings</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">Type</div>
                  <div className="text-sm">{job.schedule_type}</div>
                </div>
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">Priority</div>
                  <div className="text-sm">{job.priority}/10</div>
                </div>
                {job.cron_expression && (
                  <div className="col-span-2">
                    <div className="text-sm font-medium text-muted-foreground mb-1">Schedule</div>
                    <div className="text-sm font-mono bg-muted px-2 py-1 rounded">{job.cron_expression}</div>
                  </div>
                )}
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">Timeout</div>
                  <div className="text-sm">{job.timeout_seconds}s</div>
                </div>
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">Max Retries</div>
                  <div className="text-sm">
                    {job.retries}/{job.max_retries}
                  </div>
                </div>
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">CPU Limit</div>
                  <div className="text-sm">{job.max_cpu_percent}%</div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Payload */}
          <Card>
            <CardHeader>
              <CardTitle>Payload</CardTitle>
              <CardDescription>Job execution payload</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">Agent</div>
                  <div className="text-sm">{job.payload.agent_id}</div>
                </div>
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">Message</div>
                  <div className="text-sm bg-muted p-3 rounded whitespace-pre-wrap">{job.payload.message}</div>
                </div>
                {job.payload.system_prompt && (
                  <div>
                    <div className="text-sm font-medium text-muted-foreground mb-1">System Prompt</div>
                    <div className="text-sm bg-muted p-3 rounded whitespace-pre-wrap">{job.payload.system_prompt}</div>
                  </div>
                )}
                {job.payload.file_path && (
                  <div>
                    <div className="text-sm font-medium text-muted-foreground mb-1">File Path</div>
                    <div className="text-sm font-mono bg-muted px-2 py-1 rounded">{job.payload.file_path}</div>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

          {/* Execution History */}
          <Card>
            <CardHeader>
              <CardTitle>Execution History</CardTitle>
              <CardDescription>Job execution timestamps and results</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                {job.last_run && (
                  <div>
                    <div className="text-sm font-medium text-muted-foreground mb-1">Last Run</div>
                    <div className="text-sm">{new Date(job.last_run).toLocaleString()}</div>
                  </div>
                )}
                {job.next_run && (
                  <div>
                    <div className="text-sm font-medium text-muted-foreground mb-1">Next Run</div>
                    <div className="text-sm">{new Date(job.next_run).toLocaleString()}</div>
                  </div>
                )}
              </div>

              {/* Result */}
              {job.result && (
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-2">Result</div>
                  <div className="text-sm bg-muted p-3 rounded">
                    <pre className="whitespace-pre-wrap overflow-x-auto">
                      {JSON.stringify(job.result, null, 2)}
                    </pre>
                  </div>
                </div>
              )}

              {/* Error */}
              {job.error_message && (
                <div>
                  <div className="text-sm font-medium text-red-600 dark:text-red-400 mb-2">Error Message</div>
                  <div className="text-sm bg-red-50 dark:bg-red-950 text-red-900 dark:text-red-100 p-3 rounded">
                    {job.error_message}
                  </div>
                </div>
              )}

              {!job.result && !job.error_message && (
                <div className="text-sm text-muted-foreground text-center py-4">
                  No execution history available yet
                </div>
              )}
            </CardContent>
          </Card>

          {/* Execution History (Detailed) */}
          {executions.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Execution History</CardTitle>
                <CardDescription>Detailed history of all job executions</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {executions.map((execution) => {
                    const statusColor =
                      execution.status === 'completed'
                        ? 'text-green-600 dark:text-green-400'
                        : execution.status === 'failed'
                          ? 'text-red-600 dark:text-red-400'
                          : 'text-gray-600 dark:text-gray-400';

                    return (
                      <div
                        key={execution.id}
                        className="border rounded-lg p-4 hover:bg-muted/50 transition-colors"
                      >
                        <div className="flex items-start justify-between mb-2">
                          <div className="flex items-center gap-2">
                            {execution.status === 'completed' ? (
                              <CheckCircle className="h-5 w-5 text-green-500" />
                            ) : execution.status === 'failed' ? (
                              <XCircle className="h-5 w-5 text-red-500" />
                            ) : (
                              <AlertCircle className="h-5 w-5 text-gray-500" />
                            )}
                            <span className={`font-medium ${statusColor}`}>
                              {execution.status.charAt(0).toUpperCase() + execution.status.slice(1)}
                            </span>
                          </div>
                          <div className="text-xs text-muted-foreground">
                            {new Date(execution.started_at).toLocaleString()}
                          </div>
                        </div>

                        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-xs">
                          {execution.duration_ms && (
                            <div>
                              <span className="text-muted-foreground">Duration:</span>{' '}
                              <span className="font-medium">{execution.duration_ms}ms</span>
                            </div>
                          )}
                          {execution.tokens_used && (
                            <div>
                              <span className="text-muted-foreground">Tokens:</span>{' '}
                              <span className="font-medium">{execution.tokens_used.toLocaleString()}</span>
                            </div>
                          )}
                          {execution.cost_usd !== null && execution.cost_usd !== undefined && (
                            <div>
                              <span className="text-muted-foreground">Cost:</span>{' '}
                              <span className="font-medium">${execution.cost_usd.toFixed(4)}</span>
                            </div>
                          )}
                          {execution.session_id && (
                            <div className="col-span-full md:col-span-1">
                              <button
                                onClick={() => navigate(`/?session=${execution.session_id}`)}
                                className="flex items-center gap-1 text-primary hover:underline"
                              >
                                <ExternalLink className="h-3 w-3" />
                                View Session
                              </button>
                            </div>
                          )}
                        </div>

                        {execution.error_message && (
                          <div className="mt-2 text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950 p-2 rounded">
                            {execution.error_message}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
};

export default JobDetails;
