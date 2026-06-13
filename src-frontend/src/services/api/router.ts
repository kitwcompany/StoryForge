import { loggedInvoke } from './core';
import type {
  ModelHealthReport,
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
