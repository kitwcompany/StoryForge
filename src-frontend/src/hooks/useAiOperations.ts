import { useQuery, useMutation } from '@tanstack/react-query';
import { loggedInvoke } from '@/services/tauri';
import toast from 'react-hot-toast';

export interface AiOperation {
  id: string;
  story_id: string;
  scene_id?: string;
  chapter_id?: string;
  operation_type: string;
  operation_name: string;
  input_summary?: string;
  output_summary?: string;
  previous_content?: string;
  new_content?: string;
  metadata?: string;
  status: string;
  created_at: string;
}

export function useAiOperations(storyId: string | undefined) {
  return useQuery({
    queryKey: ['ai-operations', storyId],
    queryFn: async () => {
      if (!storyId) return [];
      return loggedInvoke<AiOperation[]>('list_ai_operations', { story_id: storyId });
    },
    enabled: !!storyId,
  });
}

export function useRollbackOperation() {
  return useMutation({
    mutationFn: async (operationId: string) => {
      return loggedInvoke<void>('rollback_ai_operation', { operation_id: operationId });
    },
    onSuccess: () => {
      toast.success('已回滚到操作前的状态');
    },
    onError: (error: Error) => {
      toast.error('回滚失败: ' + error.message);
    },
  });
}
