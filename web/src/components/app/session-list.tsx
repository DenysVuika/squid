import { deleteSession, listSessions, updateSessionTitle, type SessionListItem } from '@/lib/chat-api';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { Input } from '@/components/ui/input';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { MessageSquare, Trash2, Plus, Pencil } from 'lucide-react';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

interface SessionListProps {
  currentSessionId: string | null;
  onSessionSelect: (sessionId: string) => void;
  onNewChat: () => void;
  refreshTrigger?: number;
  apiUrl?: string;
}

export function SessionList({
  currentSessionId,
  onSessionSelect,
  onNewChat,
  refreshTrigger,
  apiUrl = '',
}: SessionListProps) {
  const [sessions, setSessions] = useState<SessionListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [sessionToDelete, setSessionToDelete] = useState<string | null>(null);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [sessionToEdit, setSessionToEdit] = useState<string | null>(null);
  const [editTitle, setEditTitle] = useState('');

  const loadSessions = async () => {
    setLoading(true);
    try {
      const data = await listSessions(apiUrl);
      setSessions(data.sessions);
    } catch (error) {
      console.error('Failed to load sessions:', error);
      toast.error('Failed to load sessions');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSessions();
  }, [apiUrl]);

  // Refresh when trigger changes
  useEffect(() => {
    if (refreshTrigger !== undefined && refreshTrigger > 0) {
      loadSessions();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refreshTrigger]);

  // Refresh session list when currentSessionId changes (new session created)
  useEffect(() => {
    if (currentSessionId) {
      // Check if this session is already in the list
      const sessionExists = sessions.some((s) => s.session_id === currentSessionId);
      if (!sessionExists) {
        // Reload sessions to include the new one
        loadSessions();
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentSessionId]);

  const handleDeleteClick = (sessionId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setSessionToDelete(sessionId);
    setDeleteDialogOpen(true);
  };

  const handleEditClick = (session: SessionListItem, e: React.MouseEvent) => {
    e.stopPropagation();
    setSessionToEdit(session.session_id);
    setEditTitle(session.title || session.preview || 'New conversation');
    setEditDialogOpen(true);
  };

  const handleEditConfirm = async () => {
    if (!sessionToEdit || !editTitle.trim()) return;

    try {
      const success = await updateSessionTitle(apiUrl, sessionToEdit, editTitle.trim());
      if (success) {
        toast.success('Session renamed');
        // Update the session in the list
        setSessions((prev) =>
          prev.map((s) => (s.session_id === sessionToEdit ? { ...s, title: editTitle.trim() } : s))
        );
      } else {
        toast.error('Failed to rename session');
      }
    } catch (error) {
      console.error('Error renaming session:', error);
      toast.error('Failed to rename session');
    } finally {
      setEditDialogOpen(false);
      setSessionToEdit(null);
      setEditTitle('');
    }
  };

  const handleDeleteConfirm = async () => {
    if (!sessionToDelete) return;

    try {
      const success = await deleteSession(apiUrl, sessionToDelete);
      if (success) {
        toast.success('Session deleted');
        setSessions((prev) => prev.filter((s) => s.session_id !== sessionToDelete));

        // If the deleted session was the current one, start a new chat
        if (sessionToDelete === currentSessionId) {
          onNewChat();
        }
      } else {
        toast.error('Failed to delete session');
      }
    } catch (error) {
      console.error('Error deleting session:', error);
      toast.error('Failed to delete session');
    } finally {
      setDeleteDialogOpen(false);
      setSessionToDelete(null);
    }
  };

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffInHours = (now.getTime() - date.getTime()) / (1000 * 60 * 60);

    if (diffInHours < 24) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (diffInHours < 24 * 7) {
      return date.toLocaleDateString([], { weekday: 'short', hour: '2-digit', minute: '2-digit' });
    } else {
      return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
    }
  };

  return (
    <>
      <div className="flex h-full flex-col border-r bg-muted/10">
        <div className="p-4">
          <Button onClick={onNewChat} className="w-full justify-start gap-2" variant="outline">
            <Plus className="h-4 w-4" />
            New Chat
          </Button>
        </div>

        <Separator />

        <ScrollArea className="flex-1">
          <div className="p-2">
            {loading ? (
              <div className="p-4 text-center text-sm text-muted-foreground">Loading sessions...</div>
            ) : sessions.length === 0 ? (
              <div className="p-4 text-center text-sm text-muted-foreground">No sessions yet. Start a new chat!</div>
            ) : (
              <div className="space-y-1">
                {sessions.map((session) => (
                  <div
                    key={session.session_id}
                    className={`group relative flex cursor-pointer items-start gap-3 rounded-lg px-3 py-2 transition-colors hover:bg-accent ${
                      session.session_id === currentSessionId ? 'bg-accent' : ''
                    }`}
                    onClick={() => onSessionSelect(session.session_id)}
                  >
                    <MessageSquare className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
                    <div className="min-w-0 flex-1">
                      <div className="flex items-start justify-between gap-2">
                        <p className="line-clamp-2 text-sm font-medium">
                          {session.title || session.preview || 'New conversation'}
                        </p>
                        <div className="flex shrink-0 gap-1 opacity-0 transition-opacity group-hover:opacity-100">
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-6 w-6"
                            onClick={(e) => handleEditClick(session, e)}
                          >
                            <Pencil className="h-3 w-3" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-6 w-6"
                            onClick={(e) => handleDeleteClick(session.session_id, e)}
                          >
                            <Trash2 className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                      <div className="mt-1 flex items-center gap-2 text-xs text-muted-foreground">
                        <span>{session.message_count} messages</span>
                        <span>•</span>
                        <span>{formatDate(session.updated_at)}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </ScrollArea>
      </div>

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
    </>
  );
}
