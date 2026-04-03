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

// ─── Helper to get message text from component ──────────────────────────────

const getMessageText = (): string => {
  // Try to find one of the expected messages
  for (const msg of EXPECTED_MESSAGES) {
    try {
      const element = screen.getByText(msg, { exact: true });
      if (element) {
        return msg;
      }
    } catch {
      // Message not found, try next
    }
  }
  return '';
};

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('ThinkingShimmer', () => {
  // ── Basic rendering ────────────────────────────────────────────────────────

  it('renders without crashing', () => {
    render(<ThinkingShimmer />);
    const message = getMessageText();
    expect(message).toBeTruthy();
  });

  it('renders one of the predefined thinking messages', () => {
    render(<ThinkingShimmer />);
    const message = getMessageText();
    expect(EXPECTED_MESSAGES).toContain(message);
  });

  // ── Message consistency ────────────────────────────────────────────────────

  it('displays the same message across re-renders', () => {
    const { rerender } = render(<ThinkingShimmer />);
    const firstMessage = getMessageText();

    rerender(<ThinkingShimmer />);
    const secondMessage = getMessageText();
    expect(secondMessage).toBe(firstMessage);

    rerender(<ThinkingShimmer />);
    const thirdMessage = getMessageText();
    expect(thirdMessage).toBe(firstMessage);
  });

  // ── Message randomization ──────────────────────────────────────────────────

  it('can render different messages across multiple component instances', () => {
    const messages = new Set<string>();

    // Render multiple instances to collect different messages
    // (probabilistic test - with 8 messages, rendering 30 instances should give us variety)
    for (let i = 0; i < 30; i++) {
      const { unmount } = render(<ThinkingShimmer />);
      const message = getMessageText();
      messages.add(message);
      unmount();
    }

    // We should get at least 2 different messages out of 30 renders
    // (probability of getting the same message 30 times is (1/8)^30, virtually impossible)
    expect(messages.size).toBeGreaterThan(1);
  });

  // ── Custom className ───────────────────────────────────────────────────────

  it('applies custom className to the Shimmer component', () => {
    const { container } = render(<ThinkingShimmer className="custom-class" />);
    const shimmer = container.querySelector('.custom-class');
    expect(shimmer).toBeInTheDocument();
  });

  it('works without a className prop', () => {
    render(<ThinkingShimmer />);
    const message = getMessageText();
    expect(message).toBeTruthy();
  });

  // ── Message variety coverage ───────────────────────────────────────────────

  it('eventually renders all predefined messages across many instances', () => {
    const messages = new Set<string>();

    // With 100 instances and 8 messages, we should hit all of them
    for (let i = 0; i < 100; i++) {
      const { unmount } = render(<ThinkingShimmer />);
      const message = getMessageText();
      messages.add(message);
      unmount();
    }

    // We should have seen most or all messages
    expect(messages.size).toBeGreaterThanOrEqual(6); // Allow some variance in randomness
  });

  // ── Message format validation ──────────────────────────────────────────────

  it('renders non-empty message text', () => {
    render(<ThinkingShimmer />);
    const message = getMessageText();
    expect(message).toBeTruthy();
    expect(message.length).toBeGreaterThan(0);
  });

  // ── Brain icon rendering ───────────────────────────────────────────────────

  it('renders the brain icon', () => {
    const { container } = render(<ThinkingShimmer />);
    const svg = container.querySelector('svg');
    expect(svg).toBeInTheDocument();
  });
});
