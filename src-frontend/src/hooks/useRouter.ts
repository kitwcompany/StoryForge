import { useMutation, useQuery } from '@tanstack/react-query';
import {
  benchmarkModelForTask,
  getModelHealthReports,
  simulateRoute,
  submitRouteFeedback,
} from '@/services/api/router';
import type { ModelHealthReport, RouteFeedback, RoutingRequest, TaskType } from '@/types/llm';

const HEALTH_KEY = 'model-health-reports';

export function useSimulateRoute() {
  return useMutation({
    mutationFn: (request: RoutingRequest) => simulateRoute(request),
  });
}

export function useBenchmarkModel() {
  return useMutation({
    mutationFn: ({ modelId, task }: { modelId: string; task: TaskType }) =>
      benchmarkModelForTask(modelId, task),
  });
}

export function useModelHealthReports(windowLimit?: number) {
  return useQuery<ModelHealthReport[]>({
    queryKey: [HEALTH_KEY, windowLimit],
    queryFn: () => getModelHealthReports(windowLimit),
    // v0.17.1: 健康报告是实时数据，关掉缓存以保证每次打开 Settings/模型健康
    // 都拿到最新结果。否则用户看到的是 staleTime 默认 0 但 React Query
    // 仍会先 hydrate 旧缓存再悄悄 refetch，造成"数据不是当前的"的体感。
    staleTime: 0,
    gcTime: 0,
    refetchOnMount: 'always',
    refetchOnWindowFocus: true,
  });
}

export function useSubmitRouteFeedback() {
  return useMutation({
    mutationFn: (feedback: RouteFeedback) => submitRouteFeedback(feedback),
  });
}
