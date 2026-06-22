import { loggedInvoke } from './core';
import type {
  ModelHealthReport,
  ModelHealthSnapshot,
  RouteFeedback,
  RoutingDecision,
  RoutingRequest,
  TaskBenchmarkResult,
  TaskType,
} from '@/types/llm';

export const simulateRoute = (request: RoutingRequest) =>
  loggedInvoke<RoutingDecision>('simulate_route', { payload: request });

export const benchmarkModelForTask = (modelId: string, task: TaskType) =>
  loggedInvoke<TaskBenchmarkResult>('benchmark_model_for_task', { modelId, task });

export const getModelHealthReports = (windowLimit?: number) =>
  loggedInvoke<ModelHealthReport[]>('get_model_health_reports', { windowLimit });

export const submitRouteFeedback = (feedback: RouteFeedback) =>
  loggedInvoke<void>('submit_route_feedback', { feedback });

// v0.23.14: 手动重新探测单个模型，让词元限制恢复后的模型即时回到可用池
export const refreshModelHealth = (modelId: string) =>
  loggedInvoke<ModelHealthSnapshot>('refresh_model_health', { modelId });
