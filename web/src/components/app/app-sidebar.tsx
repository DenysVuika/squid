import * as React from 'react';
import { MessageSquare, Plus, Pencil, Trash2, MoreHorizontal, Minus } from 'lucide-react';
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
import { NavUser } from './nav-user';

interface ChatSession {
  id: string;
  title: string;
  isActive?: boolean;
}

interface AppSidebarProps extends React.ComponentProps<typeof Sidebar> {
  sessions?: ChatSession[];
  onSessionSelect?: (sessionId: string) => void;
  onNewChat?: () => void;
  activeSessionId?: string;
}

export function AppSidebar({ sessions = [], onSessionSelect, onNewChat, activeSessionId, ...props }: AppSidebarProps) {
  const [deleteDialogOpen, setDeleteDialogOpen] = React.useState(false);
  const [sessionToDelete, setSessionToDelete] = React.useState<string | null>(null);
  const [editDialogOpen, setEditDialogOpen] = React.useState(false);
  const [sessionToEdit, setSessionToEdit] = React.useState<string | null>(null);
  const [editTitle, setEditTitle] = React.useState('');

  const { deleteSession, updateSessionTitle } = useSessionStore();

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
        <div className="flex items-center gap-2 mb-3">
          <span className="text-2xl">🦑</span>
          <span className="font-bold text-xl">Squid</span>
        </div>
        <Button onClick={onNewChat} className="w-full justify-start gap-2" variant="outline">
          <Plus className="h-4 w-4" />
          New Chat
        </Button>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarMenu>
            <Collapsible defaultOpen className="group/collapsible">
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
                          <SidebarMenuSubButton
                            asChild
                            isActive={session.id === activeSessionId}
                          >
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <button
                                  className="w-full flex items-center gap-2 pr-7"
                                  onClick={() => onSessionSelect?.(session.id)}
                                >
                                  <MessageSquare className="h-4 w-4 shrink-0" />
                                  <span className="truncate">{session.title}</span>
                                </button>
                              </TooltipTrigger>
                              <TooltipContent side="right" align="start">
                                {session.title}
                              </TooltipContent>
                            </Tooltip>
                          </SidebarMenuSubButton>
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
