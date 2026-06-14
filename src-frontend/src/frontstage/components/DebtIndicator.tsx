/**
 * DebtIndicator - 债务指示器
 *
 * 设计依据：docs/plans/2026-06-14-time-sliced-intervention-design.md 模块 4/Phase 4
 *
 * 显示当前章节未处理的 AI 审计 annotation 数量。
 * 超阈值（>10 条 high 或 >30 条总计）时变红警告。
 * 点击暂无操作（Phase 4.3 的引导会补充），纯展示。
 */

import React from 'react';
import { AlertCircle, CheckCircle2 } from 'lucide-react';
import { cn } from '@/utils/cn';
import { loggedInvoke } from '@/services/api/core';
import { useTextAnnotationsByChapter } from '@/hooks/useTextAnnotations';

interface DebtIndicatorProps {
  chapterId: string | null;
  storyId: string | null;
}

const HIGH_THRESHOLD = 10;
const TOTAL_THRESHOLD = 30;

const DebtIndicator: React.FC<DebtIndicatorProps> = ({ chapterId, storyId }) => {
  const { data: annotations } = useTextAnnotationsByChapter(chapterId);

  if (!annotations || annotations.length === 0) {
    return null; // 无债务时不显示
  }

  const total = annotations.length;
  const highCount = annotations.filter(a => a.severity === 'high').length;
  const aiAuditCount = annotations.filter(a => a.annotation_type === 'ai_audit').length;

  const isWarning = highCount >= HIGH_THRESHOLD || total >= TOTAL_THRESHOLD;

  const handleClick = () => {
    // 跳转到幕后，用户可在 SceneAnnotationPanel 处置标注
    loggedInvoke('show_backstage', { story_id: storyId }).catch(() => {});
  };

  return (
    <div
      className={cn(
        'debt-indicator flex items-center gap-1.5 px-2 py-0.5 rounded text-xs cursor-pointer transition-colors hover:opacity-80',
        isWarning
          ? 'bg-red-500/20 text-red-400 border border-red-500/40'
          : highCount > 0
            ? 'bg-amber-500/20 text-amber-400 border border-amber-500/40'
            : 'bg-cinema-700/50 text-gray-400'
      )}
      title={`未处理标注：${total} 条（${highCount} 条高优先级，${aiAuditCount} 条 AI 审计）— 点击回幕后处置`}
      onClick={handleClick}
    >
      {isWarning ? (
        <AlertCircle className="w-3 h-3" />
      ) : (
        <CheckCircle2 className="w-3 h-3" />
      )}
      <span className="font-medium">{total}</span>
      {highCount > 0 && (
        <span className={cn('ml-0.5', isWarning ? 'text-red-300' : 'text-amber-300')}>
          ({highCount})
        </span>
      )}
    </div>
  );
};

export default React.memo(DebtIndicator);
