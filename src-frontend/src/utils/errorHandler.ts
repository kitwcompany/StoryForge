/**
 * StoryForge 统一错误处理系统
 *
 * 替换现有五套不一致的错误处理模式：
 * - 完全静默 → 统一记录日志
 * - Toast 通知 → 标准化用户提示
 * - console.error → 分级日志 + 后端同步
 * - 空 catch → 至少记录 warn
 * - 混合模式 → 统一入口
 */

import toast from 'react-hot-toast';
import { createLogger, type Logger } from './logger';

export interface ErrorHandlerOptions {
  /** 错误上下文标识，如 "create_chapter", "auto_save", "load_story" */
  context: string;
  /** 是否向用户显示 Toast 通知 */
  notifyUser?: boolean;
  /** 通知消息（默认从 error 提取） */
  userMessage?: string;
  /** 日志级别 */
  logLevel?: 'warn' | 'error';
  /** 附加元数据 */
  metadata?: Record<string, unknown>;
  /** 是否将错误重新抛出 */
  rethrow?: boolean;
}

const defaultLogger = createLogger('error:handler');

function extractMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  if (
    error &&
    typeof error === 'object' &&
    'message' in error &&
    typeof (error as Record<string, unknown>).message === 'string'
  ) {
    return (error as Error).message;
  }
  return 'Unknown error';
}

function extractStack(error: unknown): string | undefined {
  if (error instanceof Error && error.stack) {
    return error.stack;
  }
  return undefined;
}

/**
 * 统一错误处理入口
 *
 * 使用示例：
 * ```ts
 * try {
 *   await createChapter(data);
 * } catch (error) {
 *   handleError(error, {
 *     context: 'create_chapter',
 *     notifyUser: true,
 *     userMessage: '创建章节失败，请重试',
 *   });
 * }
 * ```
 */
export function handleError(error: unknown, options: ErrorHandlerOptions): void {
  const {
    context,
    notifyUser = false,
    userMessage,
    logLevel = 'error',
    metadata,
    rethrow = false,
  } = options;

  const message = extractMessage(error);
  const stack = extractStack(error);

  // 构建日志上下文
  const logContext = {
    context,
    errorMessage: message,
    errorType: error instanceof Error ? error.name : typeof error,
    stack,
    ...metadata,
  };

  // 记录日志
  if (logLevel === 'error') {
    defaultLogger.error(`[${context}] ${message}`, logContext);
  } else {
    defaultLogger.warn(`[${context}] ${message}`, logContext);
  }

  // 用户通知
  if (notifyUser) {
    const displayMessage = userMessage || message;
    toast.error(displayMessage);
  }

  // 重新抛出
  if (rethrow) {
    throw error;
  }
}

/**
 * 包装异步函数，自动捕获并处理错误
 *
 * 使用示例：
 * ```ts
 * const safeCreateChapter = withErrorHandling(createChapter, {
 *   context: 'create_chapter',
 *   notifyUser: true,
 * });
 * ```
 */
export function withErrorHandling<TArgs extends unknown[], TReturn>(
  fn: (...args: TArgs) => Promise<TReturn>,
  options: Omit<ErrorHandlerOptions, 'context'> & { context: string }
): (...args: TArgs) => Promise<TReturn | undefined> {
  return async (...args: TArgs) => {
    try {
      return await fn(...args);
    } catch (error) {
      handleError(error, options);
      return undefined;
    }
  };
}

/**
 * 静默包装：捕获错误但不通知用户，仅记录日志
 *
 * 使用示例：
 * ```ts
 * const silentRefresh = silently(refreshData, { context: 'refresh_data' });
 * ```
 */
export function silently<TArgs extends unknown[], TReturn>(
  fn: (...args: TArgs) => Promise<TReturn>,
  options: { context: string; metadata?: Record<string, unknown> }
): (...args: TArgs) => Promise<TReturn | undefined> {
  return async (...args: TArgs) => {
    try {
      return await fn(...args);
    } catch (error) {
      handleError(error, {
        context: options.context,
        notifyUser: false,
        logLevel: 'warn',
        metadata: options.metadata,
      });
      return undefined;
    }
  };
}

/**
 * 为 React Query / TanStack Query 的 onError 提供统一处理
 *
 * 使用示例：
 * ```ts
 * useMutation({
 *   mutationFn: createChapter,
 *   onError: queryOnError({ context: 'create_chapter', notifyUser: true }),
 * });
 * ```
 */
export function queryOnError(
  options: Omit<ErrorHandlerOptions, 'context'> & { context: string }
) {
  return (error: unknown) => {
    handleError(error, options);
  };
}

/**
 * 创建一个带上下文的错误处理 logger
 *
 * 使用示例：
 * ```ts
 * const chapterError = createErrorLogger('chapter');
 * chapterError.handle(err, { notifyUser: true });
 * ```
 */
export function createErrorLogger(contextPrefix: string) {
  const logger = createLogger(`error:${contextPrefix}`);

  return {
    logger,
    handle: (error: unknown, options: Omit<ErrorHandlerOptions, 'context'>) => {
      handleError(error, { ...options, context: contextPrefix });
    },
    warn: (message: string, metadata?: Record<string, unknown>) => {
      logger.warn(message, metadata);
    },
    error: (message: string, metadata?: Record<string, unknown>) => {
      logger.error(message, metadata);
    },
  };
}
