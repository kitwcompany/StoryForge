import { loggedInvoke } from './core';
// ==================== LLM Stream ====================

export const llmGenerateStream = (params: {
  request_id: string;
  prompt: string;
  context?: string;
  max_tokens?: number;
  temperature?: number;
}) => loggedInvoke<void>('llm_generate_stream', { request: params });

export const llmCancelGeneration = (requestId: string) =>
  loggedInvoke<void>('llm_cancel_generation', { request_id: requestId });
// Input hint — LLM智能输入建议
export const getInputHint = (currentContent?: string) =>
  loggedInvoke<string>('get_input_hint', { current_content: currentContent });
