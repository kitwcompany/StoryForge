/**
 * 智能文思 — 感知层：Web Worker
 *
 * 将全文分析（分句、分词、词频、节奏、分布）移出主线程，
 * 避免长文档输入时阻塞 UI。
 */

import { analyzeText } from './textAnalyzer';
import type { PerceptionResult } from './types';

interface AnalyzeMessage {
  type: 'analyze';
  id: number;
  htmlContent: string;
}

interface CancelMessage {
  type: 'cancel';
  id: number;
}

type WorkerMessage = AnalyzeMessage | CancelMessage;

// 分析结果缓存：同一内容不重复分析
const resultCache = new Map<string, PerceptionResult>();
const MAX_CACHE_SIZE = 10;

let currentTaskId: number | null = null;

function ensureCacheSize() {
  if (resultCache.size >= MAX_CACHE_SIZE) {
    const firstKey = resultCache.keys().next().value;
    if (firstKey !== undefined) {
      resultCache.delete(firstKey);
    }
  }
}

self.onmessage = (event: MessageEvent<WorkerMessage>) => {
  const { type, id } = event.data;

  if (type === 'cancel') {
    if (currentTaskId === id) {
      currentTaskId = null;
    }
    return;
  }

  if (type !== 'analyze') return;

  const { htmlContent } = event.data as AnalyzeMessage;
  currentTaskId = id;

  const cacheKey = htmlContent;
  let result: PerceptionResult;
  if (resultCache.has(cacheKey)) {
    result = resultCache.get(cacheKey)!;
  } else {
    result = analyzeText(htmlContent);
    ensureCacheSize();
    resultCache.set(cacheKey, result);
  }

  // 若任务未被取消，再回传结果
  if (currentTaskId === id) {
    self.postMessage({ id, result });
    currentTaskId = null;
  }
};

export {};
