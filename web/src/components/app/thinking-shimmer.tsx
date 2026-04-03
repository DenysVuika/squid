import { useMemo, useRef, useEffect } from 'react';
import { Shimmer } from '@/components/ai-elements/shimmer';
import { BrainIcon, type BrainIconHandle } from './BrainIcon';

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
 * A Shimmer component that displays a random thinking message with an animated brain icon.
 * The message is selected once per component instance to provide variety
 * while maintaining consistency during the component's lifetime.
 */
export const ThinkingShimmer = ({ className }: ThinkingShimmerProps) => {
  const brainRef = useRef<BrainIconHandle>(null);
  // Memoize the message so it doesn't change on re-renders
  const message = useMemo(() => getRandomThinkingMessage(), []);

  // Start brain animation on mount
  useEffect(() => {
    brainRef.current?.startAnimation();
  }, []);

  return (
    <span className="inline-flex items-center gap-2">
      <BrainIcon ref={brainRef} size={16} className="text-muted-foreground" />
      <Shimmer className={className}>{message}</Shimmer>
    </span>
  );
};
