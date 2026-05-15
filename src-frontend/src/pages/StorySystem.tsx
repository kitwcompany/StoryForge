import { useState, useEffect } from 'react';
import { useAppStore } from '@/stores/appStore';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import {
  getContractTree, getRuntimeContract, getChapterCommits,
  evaluateReadingPower, getReadingPowerTrend, getChaseDebts,
  buildMemoryPack, getMemoryItems, antiAiReview,
  getGenreProfiles,
} from '@/services/tauri';
import type {
  ContractTree, RuntimeContract, ChapterCommit,
  ReadingPowerEvaluation, ChaseDebt, MemoryPack, MemoryItem,
  AntiAiReview, GenreProfile,
} from '@/services/tauri';
import {
  FileText, TrendingUp, Brain, ShieldAlert, BookOpen,
  ChevronRight, Loader2, Zap,
} from 'lucide-react';
import toast from 'react-hot-toast';

export function StorySystem() {
  const currentStory = useAppStore((s) => s.currentStory);
  const [activeTab, setActiveTab] = useState<'contracts' | 'commits' | 'reading' | 'memory' | 'anti-ai'>('contracts');
  const [isLoading, setIsLoading] = useState(false);

  // Contracts
  const [contractTree, setContractTree] = useState<ContractTree | null>(null);
  const [runtimeContract, setRuntimeContract] = useState<RuntimeContract | null>(null);
  const [selectedChapter, setSelectedChapter] = useState<number>(1);

  // Commits
  const [commits, setCommits] = useState<ChapterCommit[]>([]);

  // Reading Power
  const [readingTrend, setReadingTrend] = useState<ReadingPowerEvaluation[]>([]);
  const [chaseDebts, setChaseDebts] = useState<ChaseDebt[]>([]);

  // Memory
  const [memoryItems, setMemoryItems] = useState<MemoryItem[]>([]);

  // Anti-AI
  const [reviewText, setReviewText] = useState('');
  const [reviewResult, setReviewResult] = useState<AntiAiReview | null>(null);
  const [isReviewing, setIsReviewing] = useState(false);

  // Genre profiles
  const [genres, setGenres] = useState<GenreProfile[]>([]);

  useEffect(() => {
    loadGenres();
  }, []);

  useEffect(() => {
    if (currentStory?.id) {
      loadContracts();
      loadCommits();
      loadReadingPower();
      loadMemory();
    }
  }, [currentStory?.id]);

  const loadGenres = async () => {
    try {
      const data = await getGenreProfiles();
      setGenres(data);
    } catch (e) {
      // silent fail
    }
  };

  const loadContracts = async () => {
    if (!currentStory) return;
    try {
      const tree = await getContractTree(currentStory.id);
      setContractTree(tree);
      const runtime = await getRuntimeContract(currentStory.id, selectedChapter);
      setRuntimeContract(runtime);
    } catch (e) {
      toast.error('加载合同失败');
    }
  };

  const loadCommits = async () => {
    if (!currentStory) return;
    try {
      const data = await getChapterCommits(currentStory.id);
      setCommits(data);
    } catch (e) {
      // silent fail
    }
  };

  const loadReadingPower = async () => {
    if (!currentStory) return;
    try {
      const [trend, debts] = await Promise.all([
        getReadingPowerTrend(currentStory.id, 10),
        getChaseDebts(currentStory.id),
      ]);
      setReadingTrend(trend);
      setChaseDebts(debts);
    } catch (e) {
      // silent fail
    }
  };

  const loadMemory = async () => {
    if (!currentStory) return;
    try {
      const data = await getMemoryItems(currentStory.id);
      setMemoryItems(data);
    } catch (e) {
      // silent fail
    }
  };

  const handleAntiAiReview = async () => {
    if (!reviewText.trim()) {
      toast.error('请输入要审查的文本');
      return;
    }
    setIsReviewing(true);
    try {
      const result = await antiAiReview(reviewText, currentStory?.genre || undefined);
      setReviewResult(result);
      toast.success('审查完成');
    } catch (e) {
      toast.error('审查失败');
    } finally {
      setIsReviewing(false);
    }
  };

  const tabs = [
    { id: 'contracts' as const, label: '合同', icon: FileText },
    { id: 'commits' as const, label: '提交链', icon: BookOpen },
    { id: 'reading' as const, label: '追读力', icon: TrendingUp },
    { id: 'memory' as const, label: '记忆', icon: Brain },
    { id: 'anti-ai' as const, label: 'Anti-AI', icon: ShieldAlert },
  ];

  if (!currentStory) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500">
        请先选择一个故事
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-white mb-1">Story System</h1>
        <p className="text-gray-400 text-sm">{currentStory.title} — 合同驱动写作体系</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 mb-6 border-b border-cinema-800 pb-2">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? 'bg-cinema-gold/20 text-cinema-gold'
                : 'text-gray-400 hover:text-white hover:bg-cinema-800'
            }`}
          >
            <tab.icon className="w-4 h-4" />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Contracts Tab */}
      {activeTab === 'contracts' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <CardContent className="p-4">
              <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <FileText className="w-5 h-5 text-cinema-gold" />
                合同树
              </h3>
              {contractTree?.master_setting ? (
                <div className="space-y-3">
                  <div className="p-3 bg-cinema-800 rounded-lg">
                    <p className="text-sm text-gray-400">MASTER_SETTING</p>
                    <p className="text-white text-sm mt-1">{contractTree.master_setting.contract_json.slice(0, 200)}...</p>
                  </div>
                  <p className="text-sm text-gray-400">章节合同: {Object.keys(contractTree.chapters).length} 个</p>
                </div>
              ) : (
                <p className="text-gray-500 text-sm">暂无合同，请先创建 MASTER_SETTING</p>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <h3 className="text-lg font-semibold text-white mb-4">运行时合同</h3>
              <div className="flex items-center gap-2 mb-4">
                <input
                  type="number"
                  value={selectedChapter}
                  onChange={(e) => setSelectedChapter(parseInt(e.target.value) || 1)}
                  className="bg-cinema-800 border border-cinema-700 rounded px-3 py-1 text-white text-sm w-20"
                  min={1}
                />
                <Button size="sm" onClick={loadContracts}>加载</Button>
              </div>
              {runtimeContract ? (
                <div className="space-y-2 text-sm">
                  <p className="text-gray-400">核心基调: {JSON.parse(runtimeContract.master_setting.contract_json).core_tone || 'N/A'}</p>
                  <p className="text-gray-400">体裁: {JSON.parse(runtimeContract.master_setting.contract_json).genre || 'N/A'}</p>
                  {runtimeContract.chapter_contract && (
                    <p className="text-gray-400">章节目标: {JSON.parse(runtimeContract.chapter_contract.contract_json).chapter_directive?.goal || 'N/A'}</p>
                  )}
                </div>
              ) : (
                <p className="text-gray-500 text-sm">点击加载查看运行时合同</p>
              )}
            </CardContent>
          </Card>
        </div>
      )}

      {/* Commits Tab */}
      {activeTab === 'commits' && (
        <Card>
          <CardContent className="p-4">
            <h3 className="text-lg font-semibold text-white mb-4">章节提交链</h3>
            {commits.length === 0 ? (
              <p className="text-gray-500 text-sm">暂无提交记录</p>
            ) : (
              <div className="space-y-2">
                {commits.map((commit) => (
                  <div key={commit.id} className="p-3 bg-cinema-800 rounded-lg flex items-center justify-between">
                    <div>
                      <p className="text-white text-sm font-medium">第{commit.chapter_number}章</p>
                      <p className="text-gray-500 text-xs">状态: {commit.status}</p>
                    </div>
                    <ChevronRight className="w-4 h-4 text-gray-600" />
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Reading Power Tab */}
      {activeTab === 'reading' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <CardContent className="p-4">
              <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <TrendingUp className="w-5 h-5 text-green-400" />
                追读力趋势
              </h3>
              {readingTrend.length === 0 ? (
                <p className="text-gray-500 text-sm">暂无数据</p>
              ) : (
                <div className="space-y-3">
                  {readingTrend.map((rp) => (
                    <div key={rp.chapter_number} className="flex items-center gap-3">
                      <span className="text-gray-400 text-sm w-12">Ch{rp.chapter_number}</span>
                      <div className="flex-1 h-6 bg-cinema-800 rounded-full overflow-hidden">
                        <div
                          className="h-full rounded-full transition-all"
                          style={{
                            width: `${rp.score * 100}%`,
                            backgroundColor: rp.score > 0.7 ? '#4ade80' : rp.score > 0.4 ? '#fbbf24' : '#f87171',
                          }}
                        />
                      </div>
                      <span className="text-white text-sm w-10 text-right">{(rp.score * 100).toFixed(0)}</span>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <h3 className="text-lg font-semibold text-white mb-4">追读债务</h3>
              {chaseDebts.length === 0 ? (
                <p className="text-gray-500 text-sm">无活跃债务</p>
              ) : (
                <div className="space-y-2">
                  {chaseDebts.map((debt) => (
                    <div key={debt.id} className="p-3 bg-cinema-800 rounded-lg">
                      <p className="text-white text-sm">{debt.debt_type} — 第{debt.source_chapter}章</p>
                      <p className="text-gray-500 text-xs">金额: {debt.current_amount} / 截止: 第{debt.due_chapter}章</p>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}

      {/* Memory Tab */}
      {activeTab === 'memory' && (
        <Card>
          <CardContent className="p-4">
            <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
              <Brain className="w-5 h-5 text-purple-400" />
              记忆项 ({memoryItems.length})
            </h3>
            {memoryItems.length === 0 ? (
              <p className="text-gray-500 text-sm">暂无记忆项</p>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {memoryItems.slice(0, 30).map((item) => (
                  <div key={item.id} className="p-3 bg-cinema-800 rounded-lg">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-xs px-2 py-0.5 rounded bg-cinema-700 text-gray-300">{item.category}</span>
                      <span className="text-xs text-gray-500">Ch{item.source_chapter || '?'}</span>
                    </div>
                    <p className="text-white text-sm">{item.subject || item.value || '(空)'}</p>
                    <p className="text-gray-500 text-xs mt-1">置信度: {(item.confidence * 100).toFixed(0)}%</p>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Anti-AI Tab */}
      {activeTab === 'anti-ai' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <CardContent className="p-4">
              <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <ShieldAlert className="w-5 h-5 text-red-400" />
                Anti-AI 审查
              </h3>
              <textarea
                value={reviewText}
                onChange={(e) => setReviewText(e.target.value)}
                placeholder="粘贴要审查的文本..."
                className="w-full h-48 bg-cinema-800 border border-cinema-700 rounded-lg p-3 text-white text-sm resize-none focus:outline-none focus:border-cinema-gold"
              />
              <div className="flex justify-end mt-3">
                <Button
                  onClick={handleAntiAiReview}
                  disabled={isReviewing}
                  className="flex items-center gap-2"
                >
                  {isReviewing ? <Loader2 className="w-4 h-4 animate-spin" /> : <Zap className="w-4 h-4" />}
                  开始审查
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <h3 className="text-lg font-semibold text-white mb-4">审查结果</h3>
              {!reviewResult ? (
                <p className="text-gray-500 text-sm">输入文本并点击审查</p>
              ) : (
                <div className="space-y-4">
                  <div className="flex items-center gap-4">
                    <div className="text-3xl font-bold"
                      style={{
                        color: reviewResult.overall_score > 0.7 ? '#4ade80' : reviewResult.overall_score > 0.4 ? '#fbbf24' : '#f87171'
                      }}
                    >
                      {(reviewResult.overall_score * 100).toFixed(0)}
                    </div>
                    <div className="text-gray-400 text-sm">综合评分</div>
                  </div>

                  <div className="space-y-2">
                    {reviewResult.dimensions.map((dim) => (
                      <div key={dim.name} className="flex items-center gap-3">
                        <span className="text-gray-400 text-sm w-12">{dim.name}</span>
                        <div className="flex-1 h-4 bg-cinema-800 rounded-full overflow-hidden">
                          <div
                            className="h-full rounded-full"
                            style={{
                              width: `${dim.score * 100}%`,
                              backgroundColor: dim.score > 0.7 ? '#4ade80' : dim.score > 0.4 ? '#fbbf24' : '#f87171',
                            }}
                          />
                        </div>
                        <span className="text-white text-xs w-8">{(dim.score * 100).toFixed(0)}</span>
                      </div>
                    ))}
                  </div>

                  {reviewResult.issues.length > 0 && (
                    <div className="mt-4">
                      <h4 className="text-white text-sm font-medium mb-2">发现的问题</h4>
                      <div className="space-y-2">
                        {reviewResult.issues.slice(0, 5).map((issue, idx) => (
                          <div key={idx} className="p-2 bg-cinema-800 rounded text-sm">
                            <div className="flex items-center gap-2 mb-1">
                              <span className={`text-xs px-1.5 py-0.5 rounded ${
                                issue.severity === 'high' ? 'bg-red-900/50 text-red-300' :
                                issue.severity === 'medium' ? 'bg-yellow-900/50 text-yellow-300' :
                                'bg-blue-900/50 text-blue-300'
                              }`}>
                                {issue.severity}
                              </span>
                              <span className="text-gray-300">{issue.dimension}</span>
                            </div>
                            <p className="text-gray-400">{issue.description}</p>
                            <p className="text-cinema-gold text-xs mt-1">建议: {issue.suggestion}</p>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}
