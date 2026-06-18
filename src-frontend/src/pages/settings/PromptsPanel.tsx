import { useEffect, useMemo, useState } from 'react';
import { ChevronDown, ChevronRight, FileText, RotateCcw, Save } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { loggedInvoke } from '@/services/api/core';
import { cn } from '@/utils/cn';
import toast from 'react-hot-toast';

const VAR_TAG_OPEN = '{' + '{';
const VAR_TAG_CLOSE = '}' + '}';

type PromptCategory = 'Writer' | 'Audit' | 'Commentary' | 'Planning' | 'Analysis' | 'Probe';

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
  Audit: '审校与质量',
  Commentary: '评点',
  Planning: '规划',
  Analysis: '分析',
  Probe: '探测',
};

const CATEGORY_COLORS: Record<PromptCategory, string> = {
  Writer: 'bg-amber-500/20 text-amber-400',
  Audit: 'bg-blue-500/20 text-blue-400',
  Commentary: 'bg-purple-500/20 text-purple-400',
  Planning: 'bg-green-500/20 text-green-400',
  Analysis: 'bg-cyan-500/20 text-cyan-400',
  Probe: 'bg-gray-500/20 text-gray-400',
};

export function PromptsPanel() {
  const [entries, setEntries] = useState<PromptEntry[]>([]);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [savingId, setSavingId] = useState<string | null>(null);
  const [edited, setEdited] = useState<Record<string, string>>({});

  const fetchEntries = async () => {
    setLoading(true);
    try {
      const data = await loggedInvoke<PromptEntry[]>('list_prompt_entries');
      setEntries(data);
      // 初始化编辑状态
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

  const grouped = useMemo(() => {
    const g: Record<string, PromptEntry[]> = {};
    for (const e of entries) {
      if (!g[e.category]) g[e.category] = [];
      g[e.category].push(e);
    }
    return g;
  }, [entries]);

  const handleSaveOverride = async (id: string) => {
    setSavingId(id);
    try {
      await loggedInvoke('save_prompt_override', {
        prompt_id: id,
        content: edited[id] || '',
      });
      toast.success('提示词已保存，下次生成时生效');
      await fetchEntries(); // 刷新状态
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
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-white flex items-center gap-2">
            <FileText className="w-5 h-5 text-cinema-gold" />
            提示词注册表
          </h2>
          <p className="text-sm text-gray-500 mt-1">
            所有内置 LLM
            提示词都可以在此查看、编辑、保存覆盖。已覆盖的提示词在运行时自动取代内置默认。
          </p>
        </div>
        <span className="text-xs text-gray-500">
          共 {entries.length} 条 · {entries.filter(e => e.is_overridden).length} 条已覆盖
        </span>
      </div>

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
                {list.map(entry => {
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
                          <div className="flex items-center gap-2">
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
                            <div className="text-xs text-gray-400">
                              支持的模板变量：
                              {entry.variables.map(v => (
                                <code
                                  key={v}
                                  className="ml-1 px-1.5 py-0.5 rounded bg-cinema-800 text-cinema-gold text-xs font-mono"
                                >
                                  {VAR_TAG_OPEN + v + VAR_TAG_CLOSE}
                                </code>
                              ))}
                            </div>
                          )}

                          <textarea
                            value={draft}
                            onChange={e =>
                              setEdited(prev => ({ ...prev, [entry.id]: e.target.value }))
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
    </div>
  );
}
