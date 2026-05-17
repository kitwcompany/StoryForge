import { Star, Edit2, Key, Globe } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { cn } from '@/utils/cn';
import type { ModelConfig } from '@/types/llm';
import { getModelProviders } from '@/services/settings';

export function ModelCard({ model, isActive, connectionStatus, onEdit, onSetActive }: {
  model: ModelConfig;
  isActive?: boolean;
  connectionStatus?: { loading: boolean; success?: boolean; latency?: number; error?: string };
  onEdit: () => void;
  onSetActive: () => void;
}) {
  const isDefault = model.is_default;
  const providerMeta = getModelProviders().find(p => p.id === model.provider);
  const requiresApiKey = providerMeta?.requiresApiKey ?? true;
  // 后端 api_key 返回 '***' 表示有密钥，null/undefined/'' 表示无密钥
  const hasApiKey = model.api_key === '***' || (!!model.api_key && model.api_key !== '');

  return (
    <Card className={cn(isActive && 'border-cinema-gold ring-1 ring-cinema-gold/30')}>
      <CardContent className="p-5">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-4">
            {/* 提供商图标 */}
            <div className={cn(
              'w-12 h-12 rounded-xl flex items-center justify-center',
              isActive ? 'bg-cinema-gold/20' : 'bg-cinema-800'
            )}>
              {model.provider === 'openai' && <span className={cn('font-bold text-lg', isActive ? 'text-cinema-gold' : 'text-green-400')}>O</span>}
              {model.provider === 'anthropic' && <span className={cn('font-bold text-lg', isActive ? 'text-cinema-gold' : 'text-orange-400')}>A</span>}
              {model.provider === 'ollama' && <span className={cn('font-bold text-lg', isActive ? 'text-cinema-gold' : 'text-blue-400')}>L</span>}
              {model.provider === 'azure' && <span className={cn('font-bold text-lg', isActive ? 'text-cinema-gold' : 'text-blue-500')}>Az</span>}
              {!['openai', 'anthropic', 'ollama', 'azure'].includes(model.provider) && (
                <Globe className={cn('w-6 h-6', isActive ? 'text-cinema-gold' : 'text-gray-400')} />
              )}
            </div>

            <div>
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-white text-lg">{model.name}</h3>
                {isActive && (
                  <span className="px-2 py-0.5 bg-cinema-gold text-black text-xs rounded-full font-medium flex items-center gap-1">
                    <Star className="w-3 h-3" />
                    当前使用
                  </span>
                )}
                {!isActive && isDefault && (
                  <span className="px-2 py-0.5 bg-cinema-gold/20 text-cinema-gold text-xs rounded-full">
                    默认
                  </span>
                )}
                {!model.enabled && (
                  <span className="px-2 py-0.5 bg-red-500/20 text-red-400 text-xs rounded-full">
                    禁用
                  </span>
                )}
              </div>
              <p className="text-sm text-gray-500">
                {model.provider} · {model.model}
              </p>
              {model.description && (
                <p className="text-sm text-gray-400 mt-1">{model.description}</p>
              )}
            </div>
          </div>

          <div className="flex items-center gap-3">
            {connectionStatus && connectionStatus.loading && (
              <span className="flex items-center gap-1.5 text-gray-400 text-sm">
                <span className="w-3.5 h-3.5 rounded-full border-2 border-current border-t-transparent animate-spin" />
                检测中
              </span>
            )}
            {connectionStatus && !connectionStatus.loading && connectionStatus.success && (
              <span className="flex items-center gap-1.5 text-green-400 text-sm" title={`延迟 ${connectionStatus.latency}ms`}>
                <span className="w-2.5 h-2.5 rounded-full bg-green-400" />
                已连接 ({connectionStatus.latency}ms)
              </span>
            )}
            {connectionStatus && !connectionStatus.loading && !connectionStatus.success && (
              <span className="flex items-center gap-1.5 text-red-400 text-sm" title={connectionStatus.error}>
                <span className="w-2.5 h-2.5 rounded-full bg-red-400" />
                连接失败
              </span>
            )}
            {requiresApiKey && !hasApiKey && (
              <span className="flex items-center gap-1 text-amber-400 text-sm">
                <Key className="w-4 h-4" />
                需配置API Key
              </span>
            )}
            {!isActive && (
              <button
                onClick={onSetActive}
                className="px-3 py-1.5 text-xs font-medium text-cinema-gold bg-cinema-gold/10 hover:bg-cinema-gold/20 rounded-lg transition-colors flex items-center gap-1"
                title="设为当前使用"
              >
                <Star className="w-3.5 h-3.5" />
                设为当前
              </button>
            )}
            <Button variant="ghost" size="sm" onClick={onEdit}>
              <Edit2 className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {/* 能力标签 */}
        {'capabilities' in model && model.capabilities && (
          <div className="flex flex-wrap gap-2 mt-4">
            {model.capabilities.map(cap => (
              <span
                key={cap}
                className="px-2 py-1 bg-cinema-800 text-gray-400 text-xs rounded-lg"
              >
                {cap}
              </span>
            ))}
          </div>
        )}

        {'dimensions' in model && (
          <div className="mt-4 text-sm text-gray-500">
            维度: {model.dimensions} · 最大输入: {model.max_input_tokens} tokens
          </div>
        )}
      </CardContent>
    </Card>
  );
}
