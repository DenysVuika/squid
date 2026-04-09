import * as React from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { createJob } from '@/lib/chat-api';
import { toast } from 'sonner';

interface Agent {
  id: string;
  name: string;
  description?: string;
}

interface JobCreateDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  agents: Agent[];
  onJobCreated?: () => void;
}

export function JobCreateDialog({
  open,
  onOpenChange,
  agents,
  onJobCreated,
}: JobCreateDialogProps) {
  const [isCreating, setIsCreating] = React.useState(false);
  const [jobName, setJobName] = React.useState('');
  const [selectedAgent, setSelectedAgent] = React.useState<string>('');
  const [message, setMessage] = React.useState('');
  const [scheduleType, setScheduleType] = React.useState<'once' | 'cron'>('once');
  const [cronExpression, setCronExpression] = React.useState('');
  const [filePath, setFilePath] = React.useState('');
  const [priority, setPriority] = React.useState('5');
  const [maxCpu, setMaxCpu] = React.useState('70');
  const [timeout, setTimeout] = React.useState('3600');

  // Reset form when dialog opens
  React.useEffect(() => {
    if (open) {
      setJobName('');
      setSelectedAgent(agents.length > 0 ? agents[0].id : '');
      setMessage('');
      setScheduleType('once');
      setCronExpression('');
      setFilePath('');
      setPriority('5');
      setMaxCpu('70');
      setTimeout('3600');
    }
  }, [open, agents]);

  const handleCreate = async () => {
    // Validation
    if (!jobName.trim()) {
      toast.error('Job name is required');
      return;
    }

    if (!selectedAgent) {
      toast.error('Please select an agent');
      return;
    }

    if (!message.trim()) {
      toast.error('Message/prompt is required');
      return;
    }

    if (scheduleType === 'cron' && !cronExpression.trim()) {
      toast.error('Cron expression is required for cron jobs');
      return;
    }

    setIsCreating(true);

    try {
      await createJob('', {
        name: jobName.trim(),
        schedule_type: scheduleType,
        cron_expression: scheduleType === 'cron' ? cronExpression.trim() : undefined,
        priority: parseInt(priority, 10),
        max_cpu_percent: parseInt(maxCpu, 10),
        timeout_seconds: parseInt(timeout, 10),
        payload: {
          agent_id: selectedAgent,
          message: message.trim(),
          system_prompt: undefined,
          file_path: filePath.trim() || undefined,
          session_id: undefined,
        },
      });

      toast.success('Job created successfully!');
      onJobCreated?.();
      onOpenChange(false);
    } catch (error) {
      console.error('Failed to create job:', error);
      toast.error('Failed to create job', {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create New Job</DialogTitle>
          <DialogDescription>
            Create a background job to run an agent task once or on a schedule.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Job Name */}
          <div className="space-y-2">
            <Label htmlFor="job-name">Job Name *</Label>
            <Input
              id="job-name"
              placeholder="e.g., Daily Code Review"
              value={jobName}
              onChange={(e) => setJobName(e.target.value)}
            />
          </div>

          {/* Agent Selection */}
          <div className="space-y-2">
            <Label htmlFor="agent">Agent *</Label>
            <Select value={selectedAgent} onValueChange={setSelectedAgent}>
              <SelectTrigger id="agent">
                <SelectValue placeholder="Select an agent">
                  {selectedAgent && agents.find((a) => a.id === selectedAgent)?.name}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {agents.map((agent) => (
                  <SelectItem key={agent.id} value={agent.id}>
                    <div className="flex flex-col gap-0.5 py-1">
                      <div className="font-medium">{agent.name}</div>
                      {agent.description && (
                        <div className="text-xs text-muted-foreground line-clamp-2">
                          {agent.description}
                        </div>
                      )}
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Message/Prompt */}
          <div className="space-y-2">
            <Label htmlFor="message">Message/Prompt *</Label>
            <Textarea
              id="message"
              placeholder="The prompt or question for the agent to process"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              rows={4}
            />
          </div>

          {/* Schedule Type */}
          <div className="space-y-2">
            <Label htmlFor="schedule-type">Schedule Type *</Label>
            <Select
              value={scheduleType}
              onValueChange={(value) => setScheduleType(value as 'once' | 'cron')}
            >
              <SelectTrigger id="schedule-type">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="once">Once (run immediately)</SelectItem>
                <SelectItem value="cron">Cron (recurring schedule)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Cron Expression (only for cron jobs) */}
          {scheduleType === 'cron' && (
            <div className="space-y-2">
              <Label htmlFor="cron">Cron Expression *</Label>
              <Input
                id="cron"
                placeholder="e.g., 0 0 9 * * Mon-Fri"
                value={cronExpression}
                onChange={(e) => setCronExpression(e.target.value)}
              />
              <p className="text-xs text-muted-foreground">
                6-field format: sec min hour day month dayofweek
              </p>
            </div>
          )}

          {/* Optional: File Path */}
          <div className="space-y-2">
            <Label htmlFor="file-path">File Path (optional)</Label>
            <Input
              id="file-path"
              placeholder="Path to file for context"
              value={filePath}
              onChange={(e) => setFilePath(e.target.value)}
            />
          </div>

          {/* Advanced Settings */}
          <div className="grid grid-cols-3 gap-4">
            <div className="space-y-2">
              <Label htmlFor="priority">Priority</Label>
              <Input
                id="priority"
                type="number"
                min="0"
                max="10"
                value={priority}
                onChange={(e) => setPriority(e.target.value)}
              />
              <p className="text-xs text-muted-foreground">0-10 (higher runs first)</p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="max-cpu">Max CPU %</Label>
              <Input
                id="max-cpu"
                type="number"
                min="1"
                max="100"
                value={maxCpu}
                onChange={(e) => setMaxCpu(e.target.value)}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="timeout">Timeout (s)</Label>
              <Input
                id="timeout"
                type="number"
                min="0"
                value={timeout}
                onChange={(e) => setTimeout(e.target.value)}
              />
              <p className="text-xs text-muted-foreground">0 = no timeout</p>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isCreating}>
            Cancel
          </Button>
          <Button onClick={handleCreate} disabled={isCreating}>
            {isCreating ? 'Creating...' : 'Create Job'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
