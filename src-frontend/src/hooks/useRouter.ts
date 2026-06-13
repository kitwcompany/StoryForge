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
  });
}

export function useSubmitRouteFeedback() {
  return useMutation({
    mutationFn: (feedback: RouteFeedback) => submitRouteFeedback(feedback),
  });
}
