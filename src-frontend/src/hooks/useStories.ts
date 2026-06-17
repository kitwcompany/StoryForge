import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { listStories, createStory, updateStory, deleteStory } from '@/services/tauri';
import { useAppStore } from '@/stores/appStore';
import type { Story } from '@/types/index';
import toast from 'react-hot-toast';

const STORIES_KEY = 'stories';

export function useStories() {
  return useQuery<Story[]>({
    queryKey: [STORIES_KEY],
    queryFn: listStories,
  });
}

export function useCreateStory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: createStory,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [STORIES_KEY] });
      toast.success('故事创建成功');
    },
    onError: (error: Error) => {
      toast.error('创建失败: ' + error.message);
    },
  });
}

export function useUpdateStory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, updates }: { id: string; updates: Partial<Story> }) =>
      updateStory(id, updates),
    onMutate: async ({ id, updates }) => {
      await queryClient.cancelQueries({ queryKey: [STORIES_KEY] });
      const previousStories = queryClient.getQueryData<Story[]>([STORIES_KEY]);

      queryClient.setQueryData<Story[]>([STORIES_KEY], old => {
        if (!old) return old;
        return old.map(s => (s.id === id ? { ...s, ...updates } : s));
      });

      // 同步更新当前选中故事，确保后台设置页即时刷新
      const currentStory = useAppStore.getState().currentStory;
      if (currentStory?.id === id) {
        useAppStore.getState().setCurrentStory({ ...currentStory, ...updates });
      }

      return { previousStories };
    },
    onError: (error: Error, _vars, context) => {
      if (context?.previousStories) {
        queryClient.setQueryData([STORIES_KEY], context.previousStories);
      }
      // 失败时回滚当前选中故事到缓存中的版本
      const currentStory = useAppStore.getState().currentStory;
      const rolledBack = context?.previousStories?.find(s => s.id === currentStory?.id);
      if (rolledBack) {
        useAppStore.getState().setCurrentStory(rolledBack);
      }
      toast.error('更新失败: ' + error.message);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [STORIES_KEY] });
      toast.success('故事更新成功');
    },
  });
}

export function useDeleteStory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: deleteStory,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [STORIES_KEY] });
      toast.success('故事已删除');
    },
    onError: (error: Error) => {
      toast.error('删除失败: ' + error.message);
    },
  });
}
