import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";
import userEvent from "@testing-library/user-event";
import { SessionList } from "./session-list";
import type { SessionListItem } from "@/lib/chat-api";

// JSDOM polyfills required by Radix UI
beforeAll(() => {
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn(() => false);
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
  window.HTMLElement.prototype.setPointerCapture = vi.fn();
});

// Mocks
vi.mock("@/lib/chat-api", () => ({
  listSessions: vi.fn(),
  deleteSession: vi.fn(),
  updateSessionTitle: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

import { listSessions, deleteSession, updateSessionTitle } from "@/lib/chat-api";
import { toast } from "sonner";

const mockListSessions = vi.mocked(listSessions);
const mockDeleteSession = vi.mocked(deleteSession);
const mockUpdateSessionTitle = vi.mocked(updateSessionTitle);

// Fixtures
const makeSession = (overrides: Partial<SessionListItem> = {}): SessionListItem => ({
  session_id: "session-1",
  message_count: 10,
  created_at: 1700000000,
  updated_at: 1700000600,
  preview: "This is a preview of the conversation",
  title: "My Chat Session",
  agent_id: "agent-1",
  token_usage: {
    total_tokens: 1000,
    input_tokens: 600,
    output_tokens: 400,
    reasoning_tokens: 0,
    cache_tokens: 0,
    context_window: 4096,
    context_utilization: 0.24,
  },
  cost_usd: 0.01,
  is_readonly: false,
  ...overrides,
});

const defaultProps = {
  currentSessionId: "session-1",
  onSessionSelect: vi.fn(),
  onNewChat: vi.fn(),
};

// Helpers
const renderComponent = (props = {}) => {
  return render(<SessionList {...defaultProps} {...props} />);
};

// Tests
describe("SessionList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // Basic rendering
  it("renders New Chat button", async () => {
    mockListSessions.mockResolvedValue({ sessions: [], total: 0 });
    renderComponent();
    expect(await screen.findByText("New Chat")).toBeInTheDocument();
  });

  it("renders loading state message", () => {
    mockListSessions.mockReturnValue(new Promise(() => {}));
    renderComponent();
    expect(screen.getByText("Loading sessions...")).toBeInTheDocument();
  });

  it("renders empty state message when no sessions", async () => {
    mockListSessions.mockResolvedValue({ sessions: [], total: 0 });
    renderComponent();
    expect(await screen.findByText("No sessions yet. Start a new chat!")).toBeInTheDocument();
  });

  it("renders session list when sessions exist", async () => {
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    renderComponent();
    expect(await screen.findByText("My Chat Session")).toBeInTheDocument();
  });

  // Session item rendering
  it("renders session title when available", async () => {
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ title: "Custom Title" })], total: 1 });
    renderComponent();
    expect(await screen.findByText("Custom Title")).toBeInTheDocument();
  });

  it("falls back to preview when title is missing", async () => {
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ title: null, preview: "Preview text" })], total: 1 });
    renderComponent();
    expect(await screen.findByText("Preview text")).toBeInTheDocument();
  });

  it("falls back to New conversation when both title and preview are missing", async () => {
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ title: null, preview: null })], total: 1 });
    renderComponent();
    expect(await screen.findByText("New conversation")).toBeInTheDocument();
  });

  it("renders message count", async () => {
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ message_count: 42 })], total: 1 });
    renderComponent();
    expect(await screen.findByText("42 messages")).toBeInTheDocument();
  });

  it("renders formatted date", async () => {
    const now = new Date();
    const oneHourAgo = Math.floor((now.getTime() - 60 * 60 * 1000) / 1000);
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ updated_at: oneHourAgo })], total: 1 });
    renderComponent();
    await waitFor(() => {
      const elements = screen.getAllByText(/[\d:]+ (AM|PM)?/);
      expect(elements.length).toBeGreaterThan(0);
    });
  });

  it("highlights current session with active styling", async () => {
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ session_id: "current-session" })], total: 1 });
    renderComponent({ currentSessionId: "current-session" });
    const sessionItem = await screen.findByText("My Chat Session");
    const parentDiv = sessionItem.closest(".group");
    expect(parentDiv).toHaveClass("bg-accent");
  });

  // Session selection
  it("calls onSessionSelect with session ID when clicking a session", async () => {
    const onSessionSelect = vi.fn();
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ session_id: "clickable-session" })], total: 1 });
    renderComponent({ onSessionSelect });
    const sessionItem = await screen.findByText("My Chat Session");
    fireEvent.click(sessionItem);
    expect(onSessionSelect).toHaveBeenCalledWith("clickable-session");
  });

  // New chat
  it("calls onNewChat when New Chat button is clicked", async () => {
    const onNewChat = vi.fn();
    mockListSessions.mockResolvedValue({ sessions: [], total: 0 });
    renderComponent({ onNewChat });
    const newChatBtn = await screen.findByText("New Chat");
    fireEvent.click(newChatBtn);
    expect(onNewChat).toHaveBeenCalledTimes(1);
  });

  // Delete dialog
  it("opens delete confirmation dialog when delete button is clicked", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const trashBtn = parentDiv.querySelectorAll("button")[1];
    await user.click(trashBtn);
    expect(await screen.findByText("Delete Session")).toBeInTheDocument();
  });

  it("calls deleteSession API with correct URL and session ID on confirm", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ session_id: "del-session" })], total: 1 });
    mockDeleteSession.mockResolvedValue(true);
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const trashBtn = parentDiv.querySelectorAll("button")[1];
    await user.click(trashBtn);
    const deleteBtn = await screen.findByText("Delete");
    await user.click(deleteBtn);
    await waitFor(() => {
      expect(mockDeleteSession).toHaveBeenCalledWith("", "del-session");
    });
  });

  it("removes session from list on successful delete", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({
      sessions: [makeSession({ session_id: "to-delete" }), makeSession({ session_id: "keep", title: "Keep This" })],
      total: 2,
    });
    mockDeleteSession.mockResolvedValue(true);
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const trashBtn = parentDiv.querySelectorAll("button")[1];
    await user.click(trashBtn);
    const deleteBtn = await screen.findByText("Delete");
    await user.click(deleteBtn);
    await waitFor(() => {
      expect(screen.queryByText("My Chat Session")).not.toBeInTheDocument();
    });
    expect(screen.getByText("Keep This")).toBeInTheDocument();
  });

  it("shows error toast on failed delete", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    mockDeleteSession.mockResolvedValue(false);
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const trashBtn = parentDiv.querySelectorAll("button")[1];
    await user.click(trashBtn);
    const deleteBtn = await screen.findByText("Delete");
    await user.click(deleteBtn);
    await waitFor(() => {
      expect(toast.error).toHaveBeenCalledWith("Failed to delete session");
    });
  });

  it("calls onNewChat if deleted session was the current one", async () => {
    const user = userEvent.setup();
    const onNewChat = vi.fn();
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ session_id: "current-session" })], total: 1 });
    mockDeleteSession.mockResolvedValue(true);
    renderComponent({ currentSessionId: "current-session", onNewChat });
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const trashBtn = parentDiv.querySelectorAll("button")[1];
    await user.click(trashBtn);
    const deleteBtn = await screen.findByText("Delete");
    await user.click(deleteBtn);
    await waitFor(() => {
      expect(onNewChat).toHaveBeenCalledTimes(1);
    });
  });

  it("closes delete dialog on cancel", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    mockDeleteSession.mockResolvedValue(true);
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const trashBtn = parentDiv.querySelectorAll("button")[1];
    await user.click(trashBtn);
    const cancelBtn = await screen.findByText("Cancel");
    await user.click(cancelBtn);
    await waitFor(() => {
      expect(screen.queryByText("Delete Session")).not.toBeInTheDocument();
    });
  });

  // Edit/Rename dialog
  it("opens rename dialog when edit button is clicked", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    expect(await screen.findByText("Rename Session")).toBeInTheDocument();
  });

  it("populates input with current title", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ title: "Existing Title" })], total: 1 });
    renderComponent();
    const sessionCard = await screen.findByText("Existing Title");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    const input = await screen.findByRole("textbox");
    expect(input).toHaveValue("Existing Title");
  });

  it("populates input with preview when title is null", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ title: null, preview: "Preview Title" })], total: 1 });
    renderComponent();
    const sessionCard = await screen.findByText("Preview Title");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    const input = await screen.findByRole("textbox");
    expect(input).toHaveValue("Preview Title");
  });

  it("populates input with New conversation when both title and preview are null", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ title: null, preview: null })], total: 1 });
    renderComponent();
    const sessionCard = await screen.findByText("New conversation");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    const input = await screen.findByRole("textbox");
    expect(input).toHaveValue("New conversation");
  });

  it("calls updateSessionTitle API with correct parameters on confirm", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession({ session_id: "edit-session" })], total: 1 });
    mockUpdateSessionTitle.mockResolvedValue(true);
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    const input = await screen.findByRole("textbox");
    await user.clear(input);
    await user.type(input, "New Title");
    const renameBtn = screen.getByText("Rename");
    await user.click(renameBtn);
    await waitFor(() => {
      expect(mockUpdateSessionTitle).toHaveBeenCalledWith("", "edit-session", "New Title");
    });
  });

  it("updates session in list on successful rename", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    mockUpdateSessionTitle.mockResolvedValue(true);
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    const input = await screen.findByRole("textbox");
    await user.clear(input);
    await user.type(input, "Renamed Session");
    const renameBtn = screen.getByText("Rename");
    await user.click(renameBtn);
    await waitFor(() => {
      expect(screen.getByText("Renamed Session")).toBeInTheDocument();
    });
  });

  it("shows success toast on successful rename", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    mockUpdateSessionTitle.mockResolvedValue(true);
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    const input = await screen.findByRole("textbox");
    await user.clear(input);
    await user.type(input, "New Title");
    const renameBtn = screen.getByText("Rename");
    await user.click(renameBtn);
    await waitFor(() => {
      expect(toast.success).toHaveBeenCalledWith("Session renamed");
    });
  });

  it("shows error toast on failed rename", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    mockUpdateSessionTitle.mockResolvedValue(false);
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    const input = await screen.findByRole("textbox");
    await user.clear(input);
    await user.type(input, "New Title");
    const renameBtn = screen.getByText("Rename");
    await user.click(renameBtn);
    await waitFor(() => {
      expect(toast.error).toHaveBeenCalledWith("Failed to rename session");
    });
  });

  it("closes edit dialog on cancel", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    const cancelBtn = await screen.findByText("Cancel");
    await user.click(cancelBtn);
    await waitFor(() => {
      expect(screen.queryByText("Rename Session")).not.toBeInTheDocument();
    });
  });

  it("disables rename button when input is empty", async () => {
    const user = userEvent.setup();
    mockListSessions.mockResolvedValue({ sessions: [makeSession()], total: 1 });
    renderComponent();
    const sessionCard = await screen.findByText("My Chat Session");
    const parentDiv = sessionCard.closest(".group")!;
    fireEvent.mouseEnter(parentDiv);
    const editBtn = parentDiv.querySelectorAll("button")[0];
    await user.click(editBtn);
    const input = await screen.findByRole("textbox");
    await user.clear(input);
    const renameBtn = screen.getByText("Rename");
    expect(renameBtn).toBeDisabled();
  });

  // Data loading
  it("calls listSessions API on mount", async () => {
    mockListSessions.mockResolvedValue({ sessions: [], total: 0 });
    renderComponent();
    await waitFor(() => {
      expect(mockListSessions).toHaveBeenCalledWith("");
    });
  });

  it("refreshes when refreshTrigger changes", async () => {
    mockListSessions.mockResolvedValue({ sessions: [], total: 0 });
    const { rerender } = render(<SessionList {...defaultProps} />);
    await waitFor(() => {
      expect(mockListSessions).toHaveBeenCalledTimes(2);
    });
    rerender(<SessionList {...defaultProps} refreshTrigger={1} />);
    await waitFor(() => {
      expect(mockListSessions).toHaveBeenCalledTimes(3);
    });
  });

  it("shows error toast when loading fails", async () => {
    // Suppress expected error logs during this test
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    mockListSessions.mockRejectedValue(new Error("Network error"));
    renderComponent();
    await waitFor(() => {
      expect(toast.error).toHaveBeenCalledWith("Failed to load sessions");
    });

    consoleSpy.mockRestore();
  });
});
