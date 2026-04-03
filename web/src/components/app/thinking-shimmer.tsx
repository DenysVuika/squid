import { useMemo } from 'react';
import { Shimmer } from '@/components/ai-elements/shimmer';

// Random thinking messages for better UX
// Inspired by Claude Code and Qwen Code behavior
const THINKING_MESSAGES = [
  'Thinking...',
  'Processing your request...',
  'Analyzing...',
  'Working on it...',
  'Let me think about that...',
  'Considering options...',
  'Formulating a response...',
  'Computing...',
] as const;

/**
 * Returns a random thinking message from the predefined list
 */
const getRandomThinkingMessage = (): string => {
  return THINKING_MESSAGES[Math.floor(Math.random() * THINKING_MESSAGES.length)];
};

interface ThinkingShimmerProps {
  className?: string;
}

/**
 * A Shimmer component that displays a random thinking message.
 * The message is selected once per component instance to provide variety
 * while maintaining consistency during the component's lifetime.
 */
export const ThinkingShimmer = ({ className }: ThinkingShimmerProps) => {
  // Memoize the message so it doesn't change on re-renders
  const message = useMemo(() => getRandomThinkingMessage(), []);

  return <Shimmer className={className}>{message}</Shimmer>;
};
