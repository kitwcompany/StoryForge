/**
 * PeekDrawer - 幕前窥视面板 (v6.0.1)
 *
 * 从右侧滑出的 320px 只读面板
 * 显示角色列表和开放伏笔
 */

import { useEffect, useRef } from 'react';
import { X, User, AlertTriangle, Eye, BookOpen } from 'lucide-react';
import { cn } from '@/utils/cn';
import type { Character } from '@/types/index';
import type { PayoffLedgerItem } from '@/hooks/useForeshadowings';

interface PeekDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  characters: Character[];
  foreshadowings: PayoffLedgerItem[];
  onNavigateToBackstage: (target: 'characters' | 'foreshadowings') => void;
}

export function PeekDrawer({
  isOpen,
  onClose,
  characters,
  foreshadowings,
  onNavigateToBackstage,
}: PeekDrawerProps) {
  const drawerRef = useRef<HTMLDivElement>(null);

  // Click outside to close
  useEffect(() => {
    if (!isOpen) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (drawerRef.current && !drawerRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen, onClose]);

  const openItems = foreshadowings.filter(
    (f) => f.current_status === 'setup' || f.current_status === 'hinted' || f.current_status === 'pending_payoff' || f.current_status === 'overdue'
  );

  const overdueItems = openItems.filter((f) => f.current_status === 'overdue');
  const activeItems = openItems.filter((f) => f.current_status !== 'overdue');

  return (
    <>
      {/* Backdrop */}
      {isOpen && (
        <div className="fixed inset-0 bg-black/20 z-40 transition-opacity" onClick={onClose} />
      )}

      {/* Drawer */}
      <div
        ref={drawerRef}
        className={cn(
          'fixed top-0 right-0 h-full w-80 bg-[var(--parchment)] border-l border-[var(--warm-sand)] shadow-2xl z-50',
          'transform transition-transform duration-300 ease-out',
          isOpen ? 'translate-x-0' : 'translate-x-full'
        )}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--warm-sand)]">
          <div className="flex items-center gap-2">
            <Eye className="w-4 h-4 text-[var(--terracotta)]" />
            <span className="text-sm font-semibold text-[var(--charcoal)]">窥视面板</span>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg hover:bg-[var(--warm-sand)] text-[var(--stone-gray)] transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="overflow-y-auto h-[calc(100%-52px)] p-4 space-y-6">
          {/* Characters Section */}
          <section>
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-xs font-medium uppercase tracking-wider text-[var(--stone-gray)]">
                角色 ({characters.length})
              </h3>
              <button
                onClick={() => onNavigateToBackstage('characters')}
                className="text-[10px] text-[var(--terracotta)] hover:underline"
              >
                详情 →
              </button>
            </div>
            {characters.length === 0 ? (
              <p className="text-xs text-[var(--stone-gray)]/60 py-2">暂无角色</p>
            ) : (
              <div className="space-y-2">
                {characters.map((char) => (
                  <button
                    key={char.id}
                    onClick={() => onNavigateToBackstage('characters')}
                    className="w-full flex items-center gap-2.5 p-2 rounded-lg hover:bg-[var(--warm-sand)]/50 transition-colors text-left"
                  >
                    <div className="w-8 h-8 rounded-full bg-[var(--terracotta)]/10 flex items-center justify-center text-[var(--terracotta)] shrink-0">
                      <User className="w-4 h-4" />
                    </div>
                    <div className="min-w-0">
                      <p className="text-sm text-[var(--charcoal)] truncate">{char.name}</p>
                      {char.appearance && (
                        <p className="text-[10px] text-[var(--stone-gray)] truncate">{char.appearance}</p>
                      )}
                    </div>
                  </button>
                ))}
              </div>
            )}
          </section>

          {/* Foreshadowings Section */}
          <section>
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-xs font-medium uppercase tracking-wider text-[var(--stone-gray)]">
                开放伏笔 ({openItems.length})
              </h3>
              <button
                onClick={() => onNavigateToBackstage('foreshadowings')}
                className="text-[10px] text-[var(--terracotta)] hover:underline"
              >
                详情 →
              </button>
            </div>
            {openItems.length === 0 ? (
              <p className="text-xs text-[var(--stone-gray)]/60 py-2">暂无开放伏笔</p>
            ) : (
              <div className="space-y-2">
                {overdueItems.length > 0 && (
                  <div className="space-y-1.5">
                    <p className="text-[10px] font-medium text-red-500 flex items-center gap-1">
                      <AlertTriangle className="w-3 h-3" />
                      逾期 ({overdueItems.length})
                    </p>
                    {overdueItems.map((item) => (
                      <button
                        key={item.id}
                        onClick={() => onNavigateToBackstage('foreshadowings')}
                        className="w-full p-2 rounded-lg bg-red-50/50 border border-red-100 hover:bg-red-50 transition-colors text-left"
                      >
                        <p className="text-xs text-[var(--charcoal)] truncate">{item.title}</p>
                        <p className="text-[10px] text-red-500 mt-0.5">
                          逾期 {item.target_end_scene ? `· 目标场景 ${item.target_end_scene}` : ''}
                        </p>
                      </button>
                    ))}
                  </div>
                )}
                {activeItems.map((item) => (
                  <button
                    key={item.id}
                    onClick={() => onNavigateToBackstage('foreshadowings')}
                    className="w-full p-2 rounded-lg bg-[var(--warm-sand)]/30 hover:bg-[var(--warm-sand)]/60 transition-colors text-left"
                  >
                    <p className="text-xs text-[var(--charcoal)] truncate">{item.title}</p>
                    <p className="text-[10px] text-[var(--stone-gray)] mt-0.5">
                      {item.scope_type === 'story' ? '故事级' : item.scope_type === 'arc' ? '弧层级' : '场景级'}
                      {item.target_end_scene ? ` · 目标场景 ${item.target_end_scene}` : ''}
                    </p>
                  </button>
                ))}
              </div>
            )}
          </section>

          {/* Scene Annotations Section */}
          <section>
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-xs font-medium uppercase tracking-wider text-[var(--stone-gray)]">
                未解批注
              </h3>
            </div>
            <p className="text-xs text-[var(--stone-gray)]/60 py-2">
              在幕后工作室的「场景」页签查看详细批注
            </p>
          </section>
        </div>
      </div>
    </>
  );
}
