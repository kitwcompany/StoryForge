/**
 * 模型服务层
 *
 * v0.23.14: 移除对已删除 config/models.ts 的依赖。
 * 仅保留 checkModelStatus（幕前唯一活跃消费者），其余方法均为死代码已清除。
 */

import { loggedInvoke } from '@/services/tauri';
import { createLogger } from '@/utils/logger';

const modelServiceLogger = createLogger('services:modelService');

// v0.23.14: 保留 ChatMessage 类型（useIntent.ts 仍引用），其余死代码已清除
export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

class ModelService {
  // 检查模型连接状态（通过后端 Rust 代理，绕过 CSP/CORS 限制）
  async checkModelStatus(): Promise<'connected' | 'disconnected' | 'connecting'> {
    try {
      const status = await loggedInvoke<string>('check_model_status');
      return status as 'connected' | 'disconnected';
    } catch (error) {
      modelServiceLogger.warn('Model status check failed', { error });
      return 'disconnected';
    }
  }
}

// 导出单例
export const modelService = new ModelService();
