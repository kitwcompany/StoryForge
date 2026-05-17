import React from 'react';
import { GitBranch, Eye, BookOpen } from 'lucide-react';
import { cn } from '@/utils/cn';

interface FrontstageSidebarProps {
  isZenMode: boolean;
  isRevisionMode: boolean;
  showPeekDrawer: boolean;
  hasCurrentStory: boolean;
  onToggleRevisionMode: () => void;
  onGenerateCommentary: () => void;
  onTogglePeekDrawer: () => void;
  onOpenBackstage: () => void;
}

const FrontstageSidebar: React.FC<FrontstageSidebarProps> = ({
  isZenMode,
  isRevisionMode,
  showPeekDrawer,
  hasCurrentStory,
  onToggleRevisionMode,
  onGenerateCommentary,
  onTogglePeekDrawer,
  onOpenBackstage,
}) => {
  if (isZenMode) return null;

  return (
    <aside className="frontstage-sidebar" style={{ width: '48px' }}>
      <div className="frontstage-sidebar-content h-full flex flex-col items-center py-3 gap-1">
        <button
          className={cn('sidebar-dock-btn', isRevisionMode && 'active')}
          onClick={onToggleRevisionMode}
          title="修订模式"
        >
          <GitBranch className="w-4 h-4" />
        </button>
        <button
          className="sidebar-dock-btn"
          onClick={onGenerateCommentary}
          disabled={!hasCurrentStory}
          title="生成古典评点"
        >
          <span className="text-xs font-serif">批</span>
        </button>

        <button
          className={cn('sidebar-dock-btn', showPeekDrawer && 'active')}
          onClick={onTogglePeekDrawer}
          disabled={!hasCurrentStory}
          title="窥视面板"
        >
          <BookOpen className="w-4 h-4" />
        </button>

        <div className="flex-1 min-h-0" />

        <button
          className="sidebar-dock-btn backstage-dock-btn"
          onClick={onOpenBackstage}
          title="打开幕后工作室"
        >
          <Eye className="w-4 h-4" />
        </button>
      </div>
    </aside>
  );
};

export default React.memo(FrontstageSidebar);
