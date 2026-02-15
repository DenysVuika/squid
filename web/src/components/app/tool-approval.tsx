import { useCallback, useMemo, useState } from 'react';
import { CheckIcon, XIcon } from 'lucide-react';
import {
  Confirmation,
  ConfirmationRequest,
  ConfirmationAccepted,
  ConfirmationRejected,
  ConfirmationActions,
  ConfirmationAction,
} from '@/components/ai-elements/confirmation';
import type { ToolApproval } from '@/stores/chat-store';

interface ToolApprovalProps {
  approval: ToolApproval;
  onApprove: (save_decision: boolean, scope?: string) => void;
  onReject: (save_decision: boolean) => void;
  isApproved?: boolean;
  isRejected?: boolean;
}

export const ToolApprovalComponent = ({
  approval,
  onApprove,
  onReject,
  isApproved,
  isRejected,
}: ToolApprovalProps) => {
  const [showAlwaysOptions, setShowAlwaysOptions] = useState(false);

  // Determine the state for the Confirmation component
  const state = useMemo(() => {
    if (isApproved) return 'output-available';
    if (isRejected) return 'output-denied';
    return 'approval-requested';
  }, [isApproved, isRejected]);

  // Create a mock approval object for the Confirmation component
  const confirmationApproval = useMemo(() => {
    if (isApproved) {
      return {
        id: approval.approval_id,
        approved: true,
      };
    }
    if (isRejected) {
      return {
        id: approval.approval_id,
        approved: false,
      };
    }
    return {
      id: approval.approval_id,
    };
  }, [approval.approval_id, isApproved, isRejected]);

  const handleApprove = useCallback(() => {
    onApprove(false);
    setShowAlwaysOptions(false);
  }, [onApprove]);

  const handleReject = useCallback(() => {
    onReject(false);
    setShowAlwaysOptions(false);
  }, [onReject]);

  const handleAlwaysApprove = useCallback(() => {
    onApprove(true, approval.tool_name);
    setShowAlwaysOptions(false);
  }, [onApprove, approval.tool_name]);

  const handleAlwaysReject = useCallback(() => {
    onReject(true);
    setShowAlwaysOptions(false);
  }, [onReject]);

  const toggleAlwaysOptions = useCallback(() => {
    setShowAlwaysOptions((prev) => !prev);
  }, []);

  // Format tool arguments for display
  const formatToolArgs = useCallback(() => {
    const entries = Object.entries(approval.tool_args);
    if (entries.length === 0) return null;

    return (
      <div className="mt-2 space-y-1">
        {entries.map(([key, value]) => {
          let displayValue = String(value);
          
          // Truncate long values
          if (displayValue.length > 100) {
            displayValue = `${displayValue.substring(0, 100)}... (${displayValue.length} chars)`;
          }

          return (
            <div key={key} className="text-sm">
              <span className="font-medium">{key}:</span>{' '}
              <span className="text-muted-foreground">{displayValue}</span>
            </div>
          );
        })}
      </div>
    );
  }, [approval.tool_args]);

  return (
    <Confirmation approval={confirmationApproval} state={state} className="my-2">
      <ConfirmationRequest>
        <div className="space-y-2">
          <div className="font-semibold">Tool Execution Request</div>
          <div className="text-sm">
            The assistant wants to use <span className="font-mono font-semibold">{approval.tool_name}</span>
          </div>
          {approval.tool_description && (
            <div className="text-sm text-muted-foreground">{approval.tool_description}</div>
          )}
          {formatToolArgs()}
        </div>
      </ConfirmationRequest>

      <ConfirmationAccepted>
        <div className="flex items-center gap-2 text-green-600 dark:text-green-400">
          <CheckIcon className="size-4" />
          <span>Tool execution approved</span>
        </div>
      </ConfirmationAccepted>

      <ConfirmationRejected>
        <div className="flex items-center gap-2 text-red-600 dark:text-red-400">
          <XIcon className="size-4" />
          <span>Tool execution rejected</span>
        </div>
      </ConfirmationRejected>

      <ConfirmationActions className="flex flex-col gap-2">
        <div className="flex gap-2">
          <ConfirmationAction variant="outline" onClick={handleReject}>
            Reject
          </ConfirmationAction>
          <ConfirmationAction variant="default" onClick={handleApprove}>
            Approve
          </ConfirmationAction>
          <ConfirmationAction variant="secondary" onClick={toggleAlwaysOptions} className="ml-auto">
            {showAlwaysOptions ? 'Hide' : 'Always...'}
          </ConfirmationAction>
        </div>
        
        {showAlwaysOptions && (
          <div className="flex gap-2 pt-2 border-t">
            <ConfirmationAction variant="outline" size="sm" onClick={handleAlwaysReject}>
              Always Reject {approval.tool_name}
            </ConfirmationAction>
            <ConfirmationAction variant="default" size="sm" onClick={handleAlwaysApprove}>
              Always Approve {approval.tool_name}
            </ConfirmationAction>
          </div>
        )}
      </ConfirmationActions>
    </Confirmation>
  );
};
