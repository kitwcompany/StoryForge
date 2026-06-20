import { useEffect, useMemo, useState, useCallback } from 'react';
import {
  ChevronDown,
  ChevronRight,
  FileText,
  RotateCcw,
  Save,
  Search,
  X,
  AlertTriangle,
} from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { loggedInvoke } from '@/services/api/core';
import { cn } from '@/utils/cn';
import toast from 'react-hot-toast';

const VAR_TAG_OPEN = '{' + '{';
const VAR_TAG_CLOSE = '}' + '}';

type PromptCategory =
  | 'Writer'
  | 'Inspector'
  | 'Commentator'
  | 'Planner'
  | 'Analyzer'
  | 'Probe'
  | 'System'
  | 'Memory'
  | 'Knowledge'
  | 'Skill'
  | 'Methodology'
  | 'World'
  | 'Character'
  | 'Narrative'
  | 'Other';

interface PromptEntry {
  id: string;
  name: string;
  description: string;
  category: PromptCategory;
  default_content: string;
  current_content: string;
  is_overridden: boolean;
  variables: string[];
}

const CATEGORY_LABELS: Record<PromptCategory, string> = {
  Writer: '写作核心',
  Inspector: '质检与审校',
  Commentator: '古典评点',
  Planner: '大纲规划',
  Analyzer: '分析',
  Probe: '探测与基准',
  System: '系统',
  Memory: '记忆',
  Knowledge: '知识',
  Skill: '技能',
  Methodology: '创作方法论',
  World: '世界观与场景',
  Character: '角色',
  Narrative: '叙事结构',
  Other: '其他',
};

const CATEGORY_ORDER: PromptCategory[] = [
  'Writer',
  'Inspector',
  'Commentator',
  'Planner',
  'Analyzer',
  'World',
  'Character',
  'Narrative',
  'Methodology',
  'Skill',
  'Memory',
  'Knowledge',
  'Probe',
  'System',
  'Other',
];

const CATEGORY_COLORS: Record<PromptCategory, string> = {
  Writer: 'bg-amber-500/20 text-amber-400',
  Inspector: 'bg-blue-500/20 text-blue-400',
  Commentator: 'bg-purple-500/20 text-purple-400',
  Planner: 'bg-green-500/20 text-green-400',
  Analyzer: 'bg-cyan-500/20 text-cyan-400',
  Probe: 'bg-gray-500/20 text-gray-400',
  System: 'bg-indigo-500/20 text-indigo-400',
  Memory: 'bg-teal-500/20 text-teal-400',
  Knowledge: 'bg-rose-500/20 text-rose-400',
  Skill: 'bg-orange-500/20 text-orange-400',
  Methodology: 'bg-pink-500/20 text-pink-400',
  World: 'bg-emerald-500/20 text-emerald-400',
  Character: 'bg-violet-500/20 text-violet-400',
  Narrative: 'bg-sky-500/20 text-sky-400',
  Other: 'bg-slate-500/20 text-slate-400',
};

export function PromptsPanel() {
  const [entries, setEntries] = useState<PromptEntry[]>([]);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [savingId, setSavingId] = useState<string | null>(null);
  const [edited, setEdited] = useState<Record<string, string>>({});
  const [searchQuery, setSearchQuery] = useState('');
  const [activeCategory, setActiveCategory] = useState<PromptCategory | 'all'>('all');
  const [showResetAllConfirm, setShowResetAllConfirm] = useState(false);

  const fetchEntries = async () => {
    setLoading(true);
    try {
      const data = await loggedInvoke<PromptEntry[]>('list_prompt_entries');
      setEntries(data);
      const edits: Record<string, string> = {};
      for (const e of data) {
        edits[e.id] = e.current_content;
      }
      setEdited(edits);
    } catch (e) {
      toast.error('加载提示词列表失败');
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchEntries();
  }, []);

  const filteredEntries = useMemo(() => {
    let result = entries;

    if (activeCategory !== 'all') {
      result = result.filter((e) => e.category === activeCategory);
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (e) =>
          e.id.toLowerCase().includes(q) ||
          e.name.toLowerCase().includes(q) ||
          e.description.toLowerCase().includes(q) ||
          e.current_content.toLowerCase().includes(q)
      );
    }

    return result;
  }, [entries, activeCategory, searchQuery]);

  const grouped = useMemo(() => {
    const g: Record<string, PromptEntry[]> = {};
    for (const e of filteredEntries) {
      if (!g[e.category]) g[e.category] = [];
      g[e.category].push(e);
    }
    // 按 CATEGORY_ORDER 排序
    const sorted: Record<string, PromptEntry[]> = {};
    for (const cat of CATEGORY_ORDER) {
      if (g[cat]) {
        sorted[cat] = g[cat];
      }
    }
    // 添加任何未在 ORDER 中的分类
    for (const [cat, list] of Object.entries(g)) {
      if (!sorted[cat]) {
        sorted[cat] = list;
      }
    }
    return sorted;
  }, [filteredEntries]);

  const handleSaveOverride = async (id: string) => {
    setSavingId(id);
    try {
      await loggedInvoke('save_prompt_override', {
        prompt_id: id,
        content: edited[id] || '',
      });
      toast.success('提示词已保存，下次生成时生效');
      await fetchEntries();
    } catch (e) {
      toast.error('保存失败');
      console.error(e);
    } finally {
      setSavingId(null);
    }
  };

  const handleReset = async (id: string) => {
    if (!confirm('确定重置为内置默认值吗？该操作不可撤销。')) return;
    try {
      await loggedInvoke('reset_prompt_override', { prompt_id: id });
      toast.success('已重置为默认值');
      await fetchEntries();
    } catch (e) {
      toast.error('重置失败');
      console.error(e);
    }
  };

  const handleResetAll = async () => {
    try {
      await loggedInvoke('reset_all_prompt_overrides');
      toast.success('已重置所有提示词为默认值');
      setShowResetAllConfirm(false);
      await fetchEntries();
    } catch (e) {
      toast.error('批量重置失败');
      console.error(e);
    }
  };

  const handleClearSearch = useCallback(() => {
    setSearchQuery('');
  }, []);

  const overriddenCount = entries.filter((e) => e.is_overridden).length;

  if (loading) {
    return (
      <div className="text-center py-16 text-gray-500">
        <FileText className="w-8 h-8 mx-auto mb-2 animate-pulse" />
        正在加载提示词注册表...
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between flex-wrap gap-4">
        <div>
          <h2 className="text-xl font-semibold text-white flex items-center gap-2">
            <FileText className="w-5 h-5 text-cinema-gold" />
            提示词注册表
          </h2>
          <p className="text-sm text-gray-500 mt-1">
            所有内置 LLM 提示词都可以在此查看、编辑、保存覆盖。已覆盖的提示词在运行时自动取代内置默认。
          </p>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-xs text-gray-500">
            共 {entries.length} 条 · {overriddenCount} 条已覆盖
          </span>
          {overriddenCount > 0 && (
            <Button
              size="sm"
              variant="ghost"
              className="text-red-400 hover:text-red-300"
              onClick={() => setShowResetAllConfirm(true)}
            >
              <RotateCcw className="w-3.5 h-3.5 mr-1" />
              全部重置
            </Button>
          )}
        </div>
      </div>

      {/* Search and Filter */}
      <div className="flex items-center gap-3 flex-wrap">
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
          <input
            type="text"
            placeholder="搜索提示词 ID、名称、描述或内容..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-9 py-2 bg-cinema-900 border border-cinema-700 rounded text-sm text-white placeholder-gray-500"
          />
          {searchQuery && (
            <button
              onClick={handleClearSearch}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-white"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
        <select
          value={activeCategory}
          onChange={(e) => setActiveCategory(e.target.value as PromptCategory | 'all')}
          className="px-3 py-2 bg-cinema-900 border border-cinema-700 rounded text-sm text-white"
        >
          <option value="all">全部分类</option>
          {CATEGORY_ORDER.map((cat) => (
            <option key={cat} value={cat}>
              {CATEGORY_LABELS[cat]}
            </option>
          ))}
        </select>
      </div>

      {/* Results count */}
      {searchQuery && (
        <div className="text-sm text-gray-400">
          搜索 "{searchQuery}" 找到 {filteredEntries.length} 条结果
        </div>
      )}

      {/* Prompt Entries */}
      {Object.entries(grouped).map(([category, list]) => {
        const cat = category as PromptCategory;
        return (
          <Card key={category}>
            <CardContent className="p-0">
              <div className="px-4 py-3 border-b border-cinema-700 flex items-center gap-2">
                <span className={cn('px-2 py-0.5 rounded text-xs', CATEGORY_COLORS[cat])}>
                  {CATEGORY_LABELS[cat] ?? category}
                </span>
                <span className="text-sm text-gray-400">{list.length} 条</span>
              </div>
              <div className="divide-y divide-cinema-700">
                {list.map((entry) => {
                  const isExpanded = expandedId === entry.id;
                  const draft = edited[entry.id] ?? entry.current_content;
                  const isDirty = draft !== entry.current_content;
                  return (
                    <div key={entry.id} className="px-4 py-3">
                      <button
                        onClick={() => setExpandedId(isExpanded ? null : entry.id)}
                        className="w-full flex items-center justify-between text-left hover:bg-cinema-800/30 -mx-4 px-4 py-1 transition rounded"
                      >
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 flex-wrap">
                            <span className="text-sm text-white">{entry.name}</span>
                            <code className="text-xs text-gray-500 font-mono">{entry.id}</code>
                            {entry.is_overridden && (
                              <span className="text-xs px-2 py-0.5 rounded bg-amber-500/20 text-amber-400">
                                已覆盖
                              </span>
                            )}
                            {isDirty && (
                              <span className="text-xs px-2 py-0.5 rounded bg-blue-500/20 text-blue-400">
                                未保存
                              </span>
                            )}
                          </div>
                          <p className="text-xs text-gray-400 mt-0.5 truncate">
                            {entry.description}
                          </p>
                        </div>
                        {isExpanded ? (
                          <ChevronDown className="w-4 h-4 text-gray-500 flex-shrink-0 ml-2" />
                        ) : (
                          <ChevronRight className="w-4 h-4 text-gray-500 flex-shrink-0 ml-2" />
                        )}
                      </button>

                      {isExpanded && (
                        <div className="mt-3 space-y-3">
                          {entry.variables.length > 0 && (
                            <div className="text-xs text-gray-400 flex flex-wrap gap-1">
                              <span>支持的模板变量：</span>
                              {entry.variables.map((v) => (
                                <code
                                  key={v}
                                  className="px-1.5 py-0.5 rounded bg-cinema-800 text-cinema-gold text-xs font-mono"
                                >
                                  {VAR_TAG_OPEN + v + VAR_TAG_CLOSE}
                                </code>
                              ))}
                            </div>
                          )}

                          {/* Default content preview */}
                          {entry.is_overridden && (
                            <div className="space-y-1">
                              <div className="text-xs text-gray-500 font-medium">内置默认值（只读）：</div>
                              <div className="w-full px-3 py-2 bg-cinema-950 border border-cinema-800 rounded text-sm text-gray-400 font-mono max-h-32 overflow-y-auto">
                                {entry.default_content}
                              </div>
                            </div>
                          )}

                          <textarea
                            value={draft}
                            onChange={(e) =>
                              setEdited((prev) => ({ ...prev, [entry.id]: e.target.value }))
                            }
                            rows={Math.min(20, Math.max(8, draft.split('\n').length))}
                            className="w-full px-3 py-2 bg-cinema-900 border border-cinema-700 rounded text-sm text-white font-mono"
                            spellCheck={false}
                          />

                          <div className="flex items-center justify-between">
                            <span className="text-xs text-gray-500">
                              {draft.length} 字符 · {draft.split('\n').length} 行
                            </span>
                            <div className="flex items-center gap-2">
                              {entry.is_overridden && (
                                <Button
                                  size="sm"
                                  variant="ghost"
                                  onClick={() => handleReset(entry.id)}
                                >
                                  <RotateCcw className="w-3.5 h-3.5 mr-1" />
                                  重置默认
                                </Button>
                              )}
                              <Button
                                size="sm"
                                onClick={() => handleSaveOverride(entry.id)}
                                disabled={!isDirty || savingId === entry.id}
                                isLoading={savingId === entry.id}
                              >
                                <Save className="w-3.5 h-3.5 mr-1" />
                                保存覆盖
                              </Button>
                            </div>
                          </div>
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </CardContent>
          </Card>
        );
      })}

      {filteredEntries.length === 0 && (
        <div className="text-center py-12 text-gray-500">
          <Search className="w-8 h-8 mx-auto mb-2 opacity-50" />
          <p>未找到匹配的提示词</p>
          <p className="text-sm mt-1">尝试调整搜索条件或分类筛选</p>
        </div>
      )}

      {/* Reset All Confirmation Modal */}
      {showResetAllConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
          <div className="bg-cinema-900 border border-cinema-700 rounded-lg p-6 max-w-md w-full mx-4">
            <div className="flex items-center gap-3 mb-4">
              <AlertTriangle className="w-6 h-6 text-red-400" />
              <h3 className="text-lg font-semibold text-white">确认重置所有提示词</h3>
            </div>
            <p className="text-sm text-gray-400 mb-6">
              这将删除所有 {overriddenCount} 条自定义提示词覆盖，恢复为内置默认值。此操作不可撤销。
            </p>
            <div className="flex justify-end gap-3">
              <Button variant="ghost" onClick={() => setShowResetAllConfirm(false)}>
                取消
              </Button>
              <Button variant="danger" onClick={handleResetAll}>
                <RotateCcw className="w-3.5 h-3.5 mr-1" />
                确认重置全部
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
