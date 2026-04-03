import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, beforeAll } from 'vitest';
import { ThinkingShimmer } from './thinking-shimmer';

// ─── JSDOM polyfills required by Shimmer component ───────────────────────────

beforeAll(() => {
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn(() => false);
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
  window.HTMLElement.prototype.setPointerCapture = vi.fn();
});

// ─── Expected messages ───────────────────────────────────────────────────────

const EXPECTED_MESSAGES = [
  'Thinking...',
  'Processing your request...',
  'Analyzing...',
  'Working on it...',
  'Let me think about that...',
  'Considering options...',
  'Formulating a response...',
  'Computing...',
];

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('ThinkingShimmer', () => {
  // ── Basic rendering ────────────────────────────────────────────────────────

  it('renders without crashing', () => {
    render(<ThinkingShimmer />);
    expect(screen.getByText(/./)).toBeInTheDocument();
  });

  it('renders one of the predefined thinking messages', () => {
    render(<ThinkingShimmer />);
    const content = screen.getByText(/./);
    expect(EXPECTED_MESSAGES).toContain(content.textContent);
  });

  // ── Message consistency ────────────────────────────────────────────────────

  it('displays the same message across re-renders', () => {
    const { rerender } = render(<ThinkingShimmer />);
    const firstMessage = screen.getByText(/./);
    const firstMessageText = firstMessage.textContent;

    rerender(<ThinkingShimmer />);
    const secondMessage = screen.getByText(/./);
    expect(secondMessage.textContent).toBe(firstMessageText);

    rerender(<ThinkingShimmer />);
    const thirdMessage = screen.getByText(/./);
    expect(thirdMessage.textContent).toBe(firstMessageText);
  });

  // ── Message randomization ──────────────────────────────────────────────────

  it('can render different messages across multiple component instances', () => {
    const messages = new Set<string>();

    // Render multiple instances to collect different messages
    // (probabilistic test - with 8 messages, rendering 30 instances should give us variety)
    for (let i = 0; i < 30; i++) {
      const { unmount } = render(<ThinkingShimmer />);
      const message = screen.getByText(/./);
      messages.add(message.textContent || '');
      unmount();
    }

    // We should get at least 2 different messages out of 30 renders
    // (probability of getting the same message 30 times is (1/8)^30, virtually impossible)
    expect(messages.size).toBeGreaterThan(1);
  });

  // ── Custom className ───────────────────────────────────────────────────────

  it('applies custom className to the Shimmer component', () => {
    render(<ThinkingShimmer className="custom-class" />);
    const shimmer = screen.getByText(/./);
    expect(shimmer.className).toContain('custom-class');
  });

  it('works without a className prop', () => {
    render(<ThinkingShimmer />);
    const shimmer = screen.getByText(/./);
    expect(shimmer).toBeInTheDocument();
  });

  // ── Message variety coverage ───────────────────────────────────────────────

  it('eventually renders all predefined messages across many instances', () => {
    const messages = new Set<string>();

    // With 100 instances and 8 messages, we should hit all of them
    for (let i = 0; i < 100; i++) {
      const { unmount } = render(<ThinkingShimmer />);
      const message = screen.getByText(/./);
      messages.add(message.textContent || '');
      unmount();
    }

    // We should have seen most or all messages
    expect(messages.size).toBeGreaterThanOrEqual(6); // Allow some variance in randomness
  });

  // ── Message format validation ──────────────────────────────────────────────

  it('renders non-empty message text', () => {
    render(<ThinkingShimmer />);
    const shimmer = screen.getByText(/./);
    expect(shimmer.textContent).toBeTruthy();
    expect(shimmer.textContent!.length).toBeGreaterThan(0);
  });
});
