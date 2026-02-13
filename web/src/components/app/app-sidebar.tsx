import * as React from "react"
import { ChevronRight, MessageSquare, Plus } from "lucide-react"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar"
import { Button } from "@/components/ui/button"

interface ChatSession {
  id: string
  title: string
  isActive?: boolean
}

interface AppSidebarProps extends React.ComponentProps<typeof Sidebar> {
  sessions?: ChatSession[]
  onSessionSelect?: (sessionId: string) => void
  onNewChat?: () => void
  activeSessionId?: string
}

export function AppSidebar({
  sessions = [],
  onSessionSelect,
  onNewChat,
  activeSessionId,
  ...props
}: AppSidebarProps) {
  return (
    <Sidebar {...props}>
      <SidebarHeader className="border-b p-4">
        <div className="flex items-center gap-2 mb-3">
          <span className="text-2xl">🦑</span>
          <span className="font-bold text-xl">Squid</span>
        </div>
        <Button
          onClick={onNewChat}
          className="w-full justify-start gap-2"
          variant="outline"
        >
          <Plus className="h-4 w-4" />
          New Chat
        </Button>
      </SidebarHeader>
      <SidebarContent className="gap-0">
        <Collapsible
          defaultOpen
          className="group/collapsible"
        >
          <SidebarGroup>
            <SidebarGroupLabel
              asChild
              className="group/label text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground text-sm"
            >
              <CollapsibleTrigger>
                Chats
                <ChevronRight className="ml-auto transition-transform group-data-[state=open]/collapsible:rotate-90" />
              </CollapsibleTrigger>
            </SidebarGroupLabel>
            <CollapsibleContent>
              <SidebarGroupContent>
                <SidebarMenu>
                  {sessions.length === 0 ? (
                    <SidebarMenuItem>
                      <div className="px-2 py-1.5 text-sm text-muted-foreground">
                        No chat sessions yet
                      </div>
                    </SidebarMenuItem>
                  ) : (
                    sessions.map((session) => (
                      <SidebarMenuItem key={session.id}>
                        <SidebarMenuButton
                          asChild
                          isActive={session.id === activeSessionId}
                          onClick={() => onSessionSelect?.(session.id)}
                        >
                          <button className="w-full flex items-center gap-2">
                            <MessageSquare className="h-4 w-4 shrink-0" />
                            <span className="truncate">{session.title}</span>
                          </button>
                        </SidebarMenuButton>
                      </SidebarMenuItem>
                    ))
                  )}
                </SidebarMenu>
              </SidebarGroupContent>
            </CollapsibleContent>
          </SidebarGroup>
        </Collapsible>
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  )
}
