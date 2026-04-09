import * as React from 'react';
import { MessageSquare, Plus, Pencil, Trash2, MoreHorizontal, Minus, Bot, Clock, Play, Pause, Trash, Ban } from 'lucide-react';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarRail,
} from '@/components/ui/sidebar';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useSessionStore } from '@/stores/session-store';
import { useAgentStore } from '@/stores/agent-store';
import { useJobStore } from '@/stores/job-store';
import { NavUser } from './nav-user';
import { pauseJob, resumeJob, deleteJob, triggerJob, cancelJob } from '@/lib/chat-api';

interface ChatSession {
  id: string;
  title: string;
  isActive?: boolean;
}

interface AppSidebarProps extends React.ComponentProps<typeof Sidebar> {
  sessions?: ChatSession[];
  onSessionSelect?: (sessionId: string) => void;
  activeSessionId?: string;
  onAgentSelect?: (agentId: string) => void;
  selectedAgentId?: string;
  onJobSelect?: (jobId: number) => void;
  selectedJobId?: number;
}

export function AppSidebar({ sessions = [], onSessionSelect, activeSessionId, onAgentSelect, selectedAgentId, onJobSelect, selectedJobId, ...props }: AppSidebarProps) {
  const [deleteDialogOpen, setDeleteDialogOpen] = React.useState(false);
  const [sessionToDelete, setSessionToDelete] = React.useState<string | null>(null);
  const [editDialogOpen, setEditDialogOpen] = React.useState(false);
  const [sessionToEdit, setSessionToEdit] = React.useState<string | null>(null);
  const [editTitle, setEditTitle] = React.useState('');

  const { deleteSession, updateSessionTitle } = useSessionStore();
  const { agents, loadAgents } = useAgentStore();
  const { jobs, loadJobs, startSSE, stopSSE } = useJobStore();

  // Determine which sections should be open based on what's selected
  const [sessionsOpen, setSessionsOpen] = React.useState(() => !!activeSessionId);
  const [agentsOpen, setAgentsOpen] = React.useState(() => !!selectedAgentId);
  const [jobsOpen, setJobsOpen] = React.useState(() => !!selectedJobId);

  // Update open state when selections change (e.g., on page reload or navigation)
  React.useEffect(() => {
    if (activeSessionId) {
      setSessionsOpen(true);
    }
  }, [activeSessionId]);

  React.useEffect(() => {
    if (selectedAgentId) {
      setAgentsOpen(true);
    }
  }, [selectedAgentId]);

  React.useEffect(() => {
    if (selectedJobId) {
      setJobsOpen(true);
    }
  }, [selectedJobId]);

  // Load agents on mount
  React.useEffect(() => {
    void loadAgents();
  }, [loadAgents]);

  // Load jobs and start SSE on mount
  React.useEffect(() => {
    void loadJobs();
    startSSE();
    return () => {
      stopSSE();
    };
  }, [loadJobs, startSSE, stopSSE]);

  const handleDeleteClick = (sessionId: string) => {
    setSessionToDelete(sessionId);
    setDeleteDialogOpen(true);
  };

  const handleEditClick = (session: ChatSession) => {
    setSessionToEdit(session.id);
    setEditTitle(session.title || 'New Chat');
    setEditDialogOpen(true);
  };

  const handleDeleteConfirm = async () => {
    if (!sessionToDelete) return;

    await deleteSession(sessionToDelete);
    setDeleteDialogOpen(false);
    setSessionToDelete(null);
  };

  const handleEditConfirm = async () => {
    if (!sessionToEdit || !editTitle.trim()) return;

    await updateSessionTitle(sessionToEdit, editTitle.trim());
    setEditDialogOpen(false);
    setSessionToEdit(null);
    setEditTitle('');
  };

  const data = {
    user: {
      name: "user",
      email: "m@example.com",
      avatar: "/avatars/panda.png",
    }
  };

  return (
    <Sidebar variant="inset" {...props}>
      <SidebarHeader className="border-b p-4">
        <div className="flex items-center gap-2">
          <span className="text-2xl">🦑</span>
          <span className="font-bold text-xl">Squid</span>
        </div>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarMenu>
            <Collapsible open={sessionsOpen} onOpenChange={setSessionsOpen} className="group/collapsible">
              <SidebarMenuItem>
                <CollapsibleTrigger asChild>
                  <SidebarMenuButton>
                    Sessions{" "}
                    <Plus className="ml-auto group-data-[state=open]/collapsible:hidden" />
                    <Minus className="ml-auto group-data-[state=closed]/collapsible:hidden" />
                  </SidebarMenuButton>
                </CollapsibleTrigger>
                <CollapsibleContent>
                  <SidebarMenuSub className="mx-0 border-l-0 px-1">
                    {sessions.length === 0 ? (
                      <SidebarMenuSubItem>
                        <div className="px-2 py-1.5 text-sm text-muted-foreground">No chat sessions yet</div>
                      </SidebarMenuSubItem>
                    ) : (
                      sessions.map((session) => (
                        <SidebarMenuSubItem key={session.id} className="group/item relative">
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <SidebarMenuSubButton
                                asChild
                                isActive={session.id === activeSessionId}
                              >
                                <button
                                  className="w-full flex items-center gap-2 pr-7"
                                  onClick={() => onSessionSelect?.(session.id)}
                                >
                                  <MessageSquare className="h-4 w-4 shrink-0" />
                                  <span className="truncate">{session.title}</span>
                                </button>
                              </SidebarMenuSubButton>
                            </TooltipTrigger>
                            <TooltipContent side="right" align="start">
                              {session.title}
                            </TooltipContent>
                          </Tooltip>
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <button className="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover/item:opacity-100 transition-opacity p-1 hover:bg-sidebar-accent rounded">
                                <MoreHorizontal className="h-4 w-4" />
                                <span className="sr-only">More</span>
                              </button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent side="right" align="start">
                              <DropdownMenuItem onClick={() => handleEditClick(session)}>
                                <Pencil className="h-4 w-4" />
                                <span>Rename</span>
                              </DropdownMenuItem>
                              <DropdownMenuItem variant="destructive" onClick={() => handleDeleteClick(session.id)}>
                                <Trash2 className="h-4 w-4" />
                                <span>Delete</span>
                              </DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </SidebarMenuSubItem>
                      ))
                    )}
                  </SidebarMenuSub>
                </CollapsibleContent>
              </SidebarMenuItem>
            </Collapsible>
            <Collapsible open={agentsOpen} onOpenChange={setAgentsOpen} className="group/collapsible">
              <SidebarMenuItem>
                <CollapsibleTrigger asChild>
                  <SidebarMenuButton>
                    Agents{" "}
                    <Plus className="ml-auto group-data-[state=open]/collapsible:hidden" />
                    <Minus className="ml-auto group-data-[state=closed]/collapsible:hidden" />
                  </SidebarMenuButton>
                </CollapsibleTrigger>
                <CollapsibleContent>
                  <SidebarMenuSub className="mx-0 border-l-0 px-1">
                    {agents.length === 0 ? (
                      <SidebarMenuSubItem>
                        <div className="px-2 py-1.5 text-sm text-muted-foreground">No agents available</div>
                      </SidebarMenuSubItem>
                    ) : (
                      agents.map((agent) => (
                        <SidebarMenuSubItem key={agent.id}>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <SidebarMenuSubButton
                                asChild
                                isActive={agent.id === selectedAgentId}
                              >
                                <button
                                  className="w-full flex items-center gap-2"
                                  onClick={() => onAgentSelect?.(agent.id)}
                                >
                                  <Bot className="h-4 w-4 shrink-0" />
                                  <span className="truncate">{agent.name}</span>
                                </button>
                              </SidebarMenuSubButton>
                            </TooltipTrigger>
                            <TooltipContent side="right" align="start">
                              <div className="max-w-xs">
                                <div className="font-medium">{agent.name}</div>
                                {agent.description && (
                                  <div className="text-xs text-muted-foreground mt-1">{agent.description}</div>
                                )}
                              </div>
                            </TooltipContent>
                          </Tooltip>
                        </SidebarMenuSubItem>
                      ))
                    )}
                  </SidebarMenuSub>
                </CollapsibleContent>
              </SidebarMenuItem>
            </Collapsible>
            <Collapsible open={jobsOpen} onOpenChange={setJobsOpen} className="group/collapsible">
              <SidebarMenuItem>
                <CollapsibleTrigger asChild>
                  <SidebarMenuButton>
                    Jobs{" "}
                    <Plus className="ml-auto group-data-[state=open]/collapsible:hidden" />
                    <Minus className="ml-auto group-data-[state=closed]/collapsible:hidden" />
                  </SidebarMenuButton>
                </CollapsibleTrigger>
                <CollapsibleContent>
                  <SidebarMenuSub className="mx-0 border-l-0 px-1">
                    {jobs.length === 0 ? (
                      <SidebarMenuSubItem>
                        <div className="px-2 py-1.5 text-sm text-muted-foreground">No jobs available</div>
                      </SidebarMenuSubItem>
                    ) : (
                      jobs.map((job) => {
                        const statusColor =
                          job.status === 'running' ? 'text-blue-500' :
                          job.status === 'completed' ? 'text-green-500' :
                          job.status === 'failed' ? 'text-red-500' :
                          job.status === 'pending' ? 'text-yellow-500' :
                          'text-gray-500';

                        const activeIndicator = job.schedule_type === 'cron' ? (job.is_active ? '●' : '○') : '';

                        return (
                          <SidebarMenuSubItem key={job.id} className="group/item relative">
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <SidebarMenuSubButton asChild isActive={job.id === selectedJobId}>
                                  <button
                                    className="w-full flex items-center gap-2 pr-7"
                                    onClick={() => onJobSelect?.(job.id)}
                                  >
                                    <Clock className={`h-4 w-4 shrink-0 ${statusColor}`} />
                                    <span className="truncate flex-1 text-left">
                                      {activeIndicator && <span className="mr-1">{activeIndicator}</span>}
                                      {job.name}
                                    </span>
                                  </button>
                                </SidebarMenuSubButton>
                              </TooltipTrigger>
                              <TooltipContent side="right" align="start">
                                <div className="max-w-xs space-y-1">
                                  <div className="font-medium">{job.name}</div>
                                  <div className="text-xs text-muted-foreground">
                                    Type: {job.schedule_type}
                                  </div>
                                  <div className="text-xs text-muted-foreground">
                                    Status: {job.status}
                                  </div>
                                  {job.cron_expression && (
                                    <div className="text-xs text-muted-foreground">
                                      Schedule: {job.cron_expression}
                                    </div>
                                  )}
                                  {job.last_run && (
                                    <div className="text-xs text-muted-foreground">
                                      Last run: {new Date(job.last_run).toLocaleString()}
                                    </div>
                                  )}
                                </div>
                              </TooltipContent>
                            </Tooltip>
                            <DropdownMenu>
                              <DropdownMenuTrigger asChild>
                                <button className="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover/item:opacity-100 transition-opacity p-1 hover:bg-sidebar-accent rounded">
                                  <MoreHorizontal className="h-4 w-4" />
                                  <span className="sr-only">More</span>
                                </button>
                              </DropdownMenuTrigger>
                              <DropdownMenuContent side="right" align="start">
                                {job.schedule_type === 'cron' && (
                                  <>
                                    <DropdownMenuItem
                                      onClick={async () => {
                                        const success = await triggerJob('', job.id);
                                        if (success) {
                                          console.log(`Triggered job ${job.id}`);
                                        }
                                      }}
                                    >
                                      <Play className="h-4 w-4" />
                                      <span>Trigger Now</span>
                                    </DropdownMenuItem>
                                    {job.is_active ? (
                                      <DropdownMenuItem
                                        onClick={async () => {
                                          const success = await pauseJob('', job.id);
                                          if (success) {
                                            await loadJobs();
                                          }
                                        }}
                                      >
                                        <Pause className="h-4 w-4" />
                                        <span>Pause</span>
                                      </DropdownMenuItem>
                                    ) : (
                                      <DropdownMenuItem
                                        onClick={async () => {
                                          const success = await resumeJob('', job.id);
                                          if (success) {
                                            await loadJobs();
                                          }
                                        }}
                                      >
                                        <Play className="h-4 w-4" />
                                        <span>Resume</span>
                                      </DropdownMenuItem>
                                    )}
                                  </>
                                )}
                                {job.status === 'running' && (
                                  <DropdownMenuItem
                                    onClick={async () => {
                                      const success = await cancelJob('', job.id);
                                      if (success) {
                                        await loadJobs();
                                      }
                                    }}
                                  >
                                    <Ban className="h-4 w-4" />
                                    <span>Cancel</span>
                                  </DropdownMenuItem>
                                )}
                                <DropdownMenuItem
                                  variant="destructive"
                                  onClick={async () => {
                                    const success = await deleteJob('', job.id);
                                    if (success) {
                                      await loadJobs();
                                    }
                                  }}
                                >
                                  <Trash className="h-4 w-4" />
                                  <span>Delete</span>
                                </DropdownMenuItem>
                              </DropdownMenuContent>
                            </DropdownMenu>
                          </SidebarMenuSubItem>
                        );
                      })
                    )}
                  </SidebarMenuSub>
                </CollapsibleContent>
              </SidebarMenuItem>
            </Collapsible>
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />

      {/* Delete Confirmation Dialog */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Session</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this conversation? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleDeleteConfirm}>
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Session Title Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rename Session</DialogTitle>
            <DialogDescription>Enter a new name for this conversation.</DialogDescription>
          </DialogHeader>
          <Input
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            placeholder="Session title"
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleEditConfirm();
              }
            }}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleEditConfirm} disabled={!editTitle.trim()}>
              Rename
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Sidebar>
  );
}
