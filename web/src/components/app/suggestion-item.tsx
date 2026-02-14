import { useCallback } from 'react';
import { Suggestion } from '@/components/ai-elements/suggestion';

export const SuggestionItem = ({ 
  suggestion, 
  onClick 
}: { 
  suggestion: string; 
  onClick: (suggestion: string) => void;
}) => {
  const handleClick = useCallback(() => {
    onClick(suggestion);
  }, [onClick, suggestion]);

  return <Suggestion onClick={handleClick} suggestion={suggestion} />;
};
