import { useCallback } from 'react';
import { CheckIcon } from 'lucide-react';
import {
  ModelSelectorItem,
  ModelSelectorName,
} from '@/components/ai-elements/model-selector';
import type { ModelInfo } from '@/lib/chat-api';

export const ModelItem = ({
  m,
  isSelected,
  onSelect,
}: {
  m: ModelInfo;
  isSelected: boolean;
  onSelect: (id: string) => void;
}) => {
  const handleSelect = useCallback(() => {
    onSelect(m.id);
  }, [onSelect, m.id]);

  return (
    <ModelSelectorItem onSelect={handleSelect} value={m.id}>
      <ModelSelectorName>{m.name}</ModelSelectorName>
      {isSelected ? <CheckIcon className="ml-auto size-4" /> : <div className="ml-auto size-4" />}
    </ModelSelectorItem>
  );
};
