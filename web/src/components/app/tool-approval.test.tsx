import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, beforeAll } from 'vitest';
import userEvent from '@testing-library/user-event';
import { ToolApprovalComponent } from './tool-approval';
import type { ToolApproval } from '@/stores/chat-store';

// ─── JSDOM polyfills required by Radix UI ────────────────────────────────────

beforeAll(() => {
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn(() => false);
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
  window.HTMLElement.prototype.setPointerCapture = vi.fn();
});

// ─── Fixtures ────────────────────────────────────────────────────────────────

const makeApproval = (overrides: Partial<ToolApproval> = {}): ToolApproval => ({
  approval_id: 'approval-1',
  tool_name: 'write_file',
  tool_args: { path: '/test/file.txt', content: 'Hello World' },
  tool_description: 'Write content to a file',
  message_id: 'msg-1',
  ...overrides,
});

const defaultProps = {
  onApprove: vi.fn(),
  onReject: vi.fn(),
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

const renderComponent = (approval: ToolApproval, props = {}) => {
  return render(
    <ToolApprovalComponent
      approval={approval}
      onApprove={defaultProps.onApprove}
      onReject={defaultProps.onReject}
      {...props}
    />
  );
};

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('ToolApprovalComponent', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ── Basic rendering ────────────────────────────────────────────────────────

  it('renders tool name in the approval request', () => {
    renderComponent(makeApproval());
    expect(screen.getByText('write_file')).toBeInTheDocument();
  });

  it('renders tool description when provided', () => {
    renderComponent(makeApproval());
    expect(screen.getByText('Write content to a file')).toBeInTheDocument();
  });

  it('renders "Tool Execution Request" heading', () => {
    renderComponent(makeApproval());
    expect(screen.getByText('Tool Execution Request')).toBeInTheDocument();
  });

  it('shows "Always..." button initially', () => {
    renderComponent(makeApproval());
    expect(screen.getByText('Always...')).toBeInTheDocument();
  });

  // ── Tool args formatting ───────────────────────────────────────────────────

  it('renders tool arguments as key-value pairs', () => {
    renderComponent(makeApproval());
    expect(screen.getByText('path:')).toBeInTheDocument();
    expect(screen.getByText('/test/file.txt')).toBeInTheDocument();
    expect(screen.getByText('content:')).toBeInTheDocument();
    expect(screen.getByText('Hello World')).toBeInTheDocument();
  });

  it('truncates long argument values (>100 chars)', () => {
    const longContent = 'x'.repeat(150);
    const approval = makeApproval({
      tool_args: { content: longContent },
    });
    renderComponent(approval);

    expect(screen.getByText(/x{100}\.\.\. \(150 chars\)/)).toBeInTheDocument();
  });

  it('handles empty tool args gracefully', () => {
    const approval = makeApproval({ tool_args: {} });
    renderComponent(approval);

    // Should still render the tool name and heading
    expect(screen.getByText('write_file')).toBeInTheDocument();
    expect(screen.getByText('Tool Execution Request')).toBeInTheDocument();
  });

  it('displays multiple key-value pairs correctly', () => {
    const approval = makeApproval({
      tool_args: { arg1: 'value1', arg2: 'value2', arg3: 'value3' },
    });
    renderComponent(approval);

    expect(screen.getByText('arg1:')).toBeInTheDocument();
    expect(screen.getByText('value1')).toBeInTheDocument();
    expect(screen.getByText('arg2:')).toBeInTheDocument();
    expect(screen.getByText('value2')).toBeInTheDocument();
    expect(screen.getByText('arg3:')).toBeInTheDocument();
    expect(screen.getByText('value3')).toBeInTheDocument();
  });

  // ── Conditional rendering ──────────────────────────────────────────────────

  it('hides tool description when tool_description is empty', () => {
    const approval = makeApproval({ tool_description: '' });
    renderComponent(approval);

    expect(screen.queryByText('Write content to a file')).not.toBeInTheDocument();
  });

  // ── Approval state display ─────────────────────────────────────────────────

  it('shows approved state message when isApproved is true', () => {
    renderComponent(makeApproval(), { isApproved: true });
    expect(screen.getByText('Tool execution approved')).toBeInTheDocument();
  });

  it('shows rejected state message when isRejected is true', () => {
    renderComponent(makeApproval(), { isRejected: true });
    expect(screen.getByText('Tool execution rejected')).toBeInTheDocument();
  });

  it('shows approval request state when neither approved nor rejected', () => {
    renderComponent(makeApproval());
    expect(screen.getByText('Tool Execution Request')).toBeInTheDocument();
  });

  // ── Approve/Reject actions ─────────────────────────────────────────────────

  it('calls onApprove(false) when Approve button is clicked', async () => {
    const user = userEvent.setup();
    renderComponent(makeApproval());

    const approveBtn = screen.getByText('Approve');
    await user.click(approveBtn);

    expect(defaultProps.onApprove).toHaveBeenCalledWith(false);
  });

  it('calls onReject(false) when Reject button is clicked', async () => {
    const user = userEvent.setup();
    renderComponent(makeApproval());

    const rejectBtn = screen.getByText('Reject');
    await user.click(rejectBtn);

    expect(defaultProps.onReject).toHaveBeenCalledWith(false);
  });

  it('does not save decision when clicking Approve', async () => {
    const user = userEvent.setup();
    renderComponent(makeApproval());

    const approveBtn = screen.getByText('Approve');
    await user.click(approveBtn);

    // Second argument should not be present (no save decision)
    expect(defaultProps.onApprove).toHaveBeenCalledWith(false);
  });

  it('does not save decision when clicking Reject', async () => {
    const user = userEvent.setup();
    renderComponent(makeApproval());

    const rejectBtn = screen.getByText('Reject');
    await user.click(rejectBtn);

    expect(defaultProps.onReject).toHaveBeenCalledWith(false);
  });

  // ── Always options ─────────────────────────────────────────────────────────

  it('shows "Always Reject" and "Always Approve" buttons when "Always..." is clicked', async () => {
    const user = userEvent.setup();
    renderComponent(makeApproval());

    const alwaysBtn = screen.getByText('Always...');
    await user.click(alwaysBtn);

    expect(screen.getByText('Always Reject write_file')).toBeInTheDocument();
    expect(screen.getByText('Always Approve write_file')).toBeInTheDocument();
  });

  it('hides always options when "Hide" is clicked', async () => {
    const user = userEvent.setup();
    renderComponent(makeApproval());

    const alwaysBtn = screen.getByText('Always...');
    await user.click(alwaysBtn);
    expect(screen.getByText('Hide')).toBeInTheDocument();

    const hideBtn = screen.getByText('Hide');
    await user.click(hideBtn);

    expect(screen.queryByText('Always Reject write_file')).not.toBeInTheDocument();
    expect(screen.queryByText('Always Approve write_file')).not.toBeInTheDocument();
    expect(screen.getByText('Always...')).toBeInTheDocument();
  });

  it('calls onApprove(true, tool_name) when "Always Approve" is clicked', async () => {
    const user = userEvent.setup();
    renderComponent(makeApproval());

    const alwaysBtn = screen.getByText('Always...');
    await user.click(alwaysBtn);

    const alwaysApproveBtn = screen.getByText('Always Approve write_file');
    await user.click(alwaysApproveBtn);

    expect(defaultProps.onApprove).toHaveBeenCalledWith(true, 'write_file');
  });

  it('calls onReject(true) when "Always Reject" is clicked', async () => {
    const user = userEvent.setup();
    renderComponent(makeApproval());

    const alwaysBtn = screen.getByText('Always...');
    await user.click(alwaysBtn);

    const alwaysRejectBtn = screen.getByText('Always Reject write_file');
    await user.click(alwaysRejectBtn);

    expect(defaultProps.onReject).toHaveBeenCalledWith(true);
  });

  it('saves decision with tool scope when always approving', async () => {
    const user = userEvent.setup();
    renderComponent(makeApproval());

    const alwaysBtn = screen.getByText('Always...');
    await user.click(alwaysBtn);

    const alwaysApproveBtn = screen.getByText('Always Approve write_file');
    await user.click(alwaysApproveBtn);

    expect(defaultProps.onApprove).toHaveBeenCalledWith(true, 'write_file');
  });
});
