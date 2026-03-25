import { useCallback } from 'react';
import { CheckIcon } from 'lucide-react';
import { ModelSelectorItem, ModelSelectorName } from '@/components/ai-elements/model-selector';
import type { AgentInfo } from '@/lib/chat-api';

export const AgentItem = ({
  agent,
  isSelected,
  onSelect,
}: {
  agent: AgentInfo;
  isSelected: boolean;
  onSelect: (id: string) => void;
}) => {
  const handleSelect = useCallback(() => {
    onSelect(agent.id);
  }, [onSelect, agent.id]);

  return (
    <ModelSelectorItem onSelect={handleSelect} value={agent.id}>
      <div className="flex flex-col flex-1">
        <ModelSelectorName>{agent.name}</ModelSelectorName>
        <span className="text-xs text-muted-foreground">{agent.description}</span>
        <span className="text-xs text-muted-foreground/60 mt-0.5">Model: {agent.model}</span>
      </div>
      {isSelected ? <CheckIcon className="ml-auto size-4" /> : <div className="ml-auto size-4" />}
    </ModelSelectorItem>
  );
};
