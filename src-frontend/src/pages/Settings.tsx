/**
 * Settings Page - 工作室配置
 * 
 * 支持多类型LLM配置管理：
 * - Chat/Completion 模型（文本生成）
 * - Embedding 模型（向量嵌入）
 * - Multimodal 模型（多模态）
 * - Image 模型（图像生成）
 * 
 * 功能：
 * - 添加/编辑/删除模型配置
 * - 设置默认模型
 * - Agent模型映射
 * - 设置导出/导入
 */

import { useState, useEffect } from 'react';
import {
  Settings2, Database,
  Download, Upload,
  Bot, Sparkles, MessageSquare,
  Compass,
  User,
  GitBranch,
  BarChart3,
} from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { useSettings, useModels, useExportSettings, useImportSettings, useSetActiveModel } from '@/hooks/useSettings';
import { cn } from '@/utils/cn';
import type { ModelConfig, ModelType } from '@/types/llm';
import { testModelConnection } from '@/services/settings';
import { logFeatureUsage } from '@/services/tauri';

import { ModelList } from './settings/ModelList';
import { ModelModal } from './settings/ModelModal';
import { StatsSettings } from './settings/StatsSettings';
import { MethodologySettings } from './settings/MethodologySettings';
import { WorkflowSettings } from './settings/WorkflowSettings';
import { GeneralSettings } from './settings/GeneralSettings';
import { AccountSettings } from './settings/AccountSettings';

type TabType = 'chat' | 'embedding' | 'multimodal' | 'image' | 'agents' | 'methodology' | 'workflows' | 'general' | 'account' | 'stats';

export function Settings() {
  const [activeTab, setActiveTab] = useState<TabType>('chat');
  const [showAddModal, setShowAddModal] = useState(false);
  const [editingModel, setEditingModel] = useState<ModelConfig | null>(null);
  
  const { data: settings, isLoading: settingsLoading } = useSettings();
  const { data: models = [], isLoading: modelsLoading } = useModels();
  const [activeModelIds, setActiveModelIds] = useState<Record<string, string>>({});
  
  // 同步活跃模型状态
  useEffect(() => {
    if (settings?.active_models) {
      setActiveModelIds(settings.active_models);
    }
  }, [settings]);
  const exportSettings = useExportSettings();
  const importSettings = useImportSettings();
  const setActiveModelMutation = useSetActiveModel();
  
  const isLoading = settingsLoading || modelsLoading;

  // Feature usage telemetry
  useEffect(() => {
    if (activeTab === 'stats') {
      logFeatureUsage('feature_stats', 'opened');
    }
  }, [activeTab]);

  // 模型连接状态
  const [connectionStatus, setConnectionStatus] = useState<Record<string, { loading: boolean; success?: boolean; latency?: number; error?: string }>>({});

  // 对当前 tab 的模型进行连接测试
  useEffect(() => {
    if (!models.length) return;

    const currentTabModels = models.filter(m => m.type === activeTab);
    if (!currentTabModels.length) return;

    const testConnections = async () => {
      // 将当前 tab 的模型设为加载中
      setConnectionStatus(prev => {
        const next = { ...prev };
        currentTabModels.forEach(m => { next[m.id] = { loading: true }; });
        return next;
      });

      for (const model of currentTabModels) {
        try {
          const result = await testModelConnection(model.id);
          setConnectionStatus(prev => ({
            ...prev,
            [model.id]: { loading: false, success: result.success, latency: result.latency, error: result.error }
          }));
        } catch (e) {
          setConnectionStatus(prev => ({
            ...prev,
            [model.id]: { loading: false, success: false, error: '测试失败' }
          }));
        }
      }
    };

    testConnections();
  }, [models, activeTab]);
  
  // 按类型过滤模型
  const filteredModels = models.filter(m => m.type === activeTab);
  
  // 处理设置导入
  const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      importSettings.mutate(file);
    }
  };
  
  return (
    <div className="p-8 space-y-6 animate-fade-in">
      {/* 头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="font-display text-3xl font-bold text-white">工作室配置</h1>
          <p className="text-gray-400">配置LLM模型和全局设置</p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="ghost" onClick={() => exportSettings.mutate()} isLoading={exportSettings.isPending}>
            <Download className="w-4 h-4 mr-2" />
            导出设置
          </Button>
          <label className="cursor-pointer inline-flex items-center gap-2 px-4 py-2 text-gray-400 hover:text-white hover:bg-cinema-800/50 rounded-xl transition-all">
            <input type="file" accept=".json" className="hidden" onChange={handleImport} />
            {importSettings.isPending ? (
              <span className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
            ) : (
              <Upload className="w-4 h-4" />
            )}
            导入设置
          </label>
        </div>
      </div>
      
      {/* 标签页 */}
      <div className="flex items-center gap-2 border-b border-cinema-800 pb-4 overflow-x-auto">
        <TabButton 
          active={activeTab === 'chat'} 
          onClick={() => setActiveTab('chat')}
          icon={<MessageSquare className="w-4 h-4" />}
          label="聊天模型"
        />
        <TabButton 
          active={activeTab === 'embedding'} 
          onClick={() => setActiveTab('embedding')}
          icon={<Database className="w-4 h-4" />}
          label="嵌入模型"
        />
        <TabButton 
          active={activeTab === 'multimodal'} 
          onClick={() => setActiveTab('multimodal')}
          icon={<Sparkles className="w-4 h-4" />}
          label="多模态"
        />
        {/* v5.6.4 修复: 图像生成功能暂未实现，隐藏该 Tab */}
        {/* <TabButton
          active={activeTab === 'image'}
          onClick={() => setActiveTab('image')}
          icon={<Image className="w-4 h-4" />}
          label="图像生成"
        /> */}
        <TabButton 
          active={activeTab === 'agents'} 
          onClick={() => setActiveTab('agents')}
          icon={<Bot className="w-4 h-4" />}
          label="Agent配置"
        />
        <TabButton 
          active={activeTab === 'methodology'} 
          onClick={() => setActiveTab('methodology')}
          icon={<Compass className="w-4 h-4" />}
          label="创作方法论"
        />
        <TabButton 
          active={activeTab === 'workflows'} 
          onClick={() => setActiveTab('workflows')}
          icon={<GitBranch className="w-4 h-4" />}
          label="工作流"
        />
        <TabButton
          active={activeTab === 'general'}
          onClick={() => setActiveTab('general')}
          icon={<Settings2 className="w-4 h-4" />}
          label="通用设置"
        />
        <TabButton
          active={activeTab === 'stats'}
          onClick={() => setActiveTab('stats')}
          icon={<BarChart3 className="w-4 h-4" />}
          label="数据统计"
        />
        <TabButton
          active={activeTab === 'account'}
          onClick={() => setActiveTab('account')}
          icon={<User className="w-4 h-4" />}
          label="账号与登录"
        />
      </div>
      
      {/* 内容区域 */}
      {isLoading ? (
        <div className="text-center py-12 text-gray-500">加载中...</div>
      ) : (
        <>
          {activeTab === 'chat' && (
            <ModelList 
              type="chat" 
              models={filteredModels}
              activeModelId={activeModelIds.chat}
              connectionStatus={connectionStatus}
              onAdd={() => setShowAddModal(true)}
              onEdit={setEditingModel}
              onSetActive={(modelId) => {
                setActiveModelMutation.mutate({ type: 'chat', modelId });
              }}
            />
          )}
          {activeTab === 'embedding' && (
            <ModelList 
              type="embedding" 
              models={filteredModels}
              activeModelId={activeModelIds.embedding}
              connectionStatus={connectionStatus}
              onAdd={() => setShowAddModal(true)}
              onEdit={setEditingModel}
              onSetActive={(modelId) => {
                setActiveModelMutation.mutate({ type: 'embedding', modelId });
              }}
            />
          )}
          {activeTab === 'multimodal' && (
            <ModelList 
              type="multimodal" 
              models={filteredModels}
              activeModelId={activeModelIds.multimodal}
              connectionStatus={connectionStatus}
              onAdd={() => setShowAddModal(true)}
              onEdit={setEditingModel}
              onSetActive={(modelId) => {
                setActiveModelMutation.mutate({ type: 'multimodal', modelId });
              }}
            />
          )}
          {activeTab === 'image' && (
            <ModelList 
              type="image" 
              models={filteredModels}
              activeModelId={activeModelIds.image}
              connectionStatus={connectionStatus}
              onAdd={() => setShowAddModal(true)}
              onEdit={setEditingModel}
              onSetActive={(modelId) => {
                setActiveModelMutation.mutate({ type: 'image', modelId });
              }}
            />
          )}
          {activeTab === 'agents' && <AgentConfig />}
          {activeTab === 'methodology' && <MethodologySettings />}
          {activeTab === 'workflows' && <WorkflowSettings />}
          {activeTab === 'general' && <GeneralSettings />}
          {activeTab === 'stats' && <StatsSettings />}
          {activeTab === 'account' && <AccountSettings />}
        </>
      )}

      {/* 添加/编辑模态框 */}
      {(showAddModal || editingModel) && (
        <ModelModal
          type={activeTab as ModelType}
          model={editingModel}
          onClose={() => {
            setShowAddModal(false);
            setEditingModel(null);
          }}
        />
      )}
    </div>
  );
}

function TabButton({ active, onClick, icon, label }: {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors whitespace-nowrap',
        active
          ? 'bg-cinema-gold text-black'
          : 'text-gray-400 hover:text-white hover:bg-cinema-800'
      )}
    >
      {icon}
      {label}
    </button>
  );
}

// Agent配置组件
function AgentConfig() {
  return (
    <Card>
      <CardContent className="p-8 text-center">
        <Bot className="w-16 h-16 text-gray-600 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-white mb-2">Agent模型映射</h3>
        <p className="text-gray-500">为不同的Agent配置专用的LLM模型</p>
        <p className="text-sm text-gray-600 mt-4">功能开发中...</p>
      </CardContent>
    </Card>
  );
}
