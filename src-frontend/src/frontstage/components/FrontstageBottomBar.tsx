import React, { useState } from 'react';
import { Send, X } from 'lucide-react';
import { StatusIcon } from './StatusIcon';

interface FrontstageBottomBarProps {
  isZenMode: boolean;
  isGenerating: boolean;
  generationStatus: string;
  inputValue: string;
  ghostHint: string;
  hintSource: 'llm' | 'history';
  modelStatus: 'connected' | 'disconnected' | 'connecting';
  modelName: string;
  onInputChange: (value: string) => void;
  onInputSubmit: () => void;
  onCancelGeneration: () => void;
  onInputFocus: () => void;
  onInputKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
}

const FrontstageBottomBar: React.FC<FrontstageBottomBarProps> = ({
  isZenMode,
  isGenerating,
  generationStatus,
  inputValue,
  ghostHint,
  hintSource,
  modelStatus,
  modelName,
  onInputChange,
  onInputSubmit,
  onCancelGeneration,
  onInputFocus,
  onInputKeyDown,
}) => {
  const [showModelTooltip, setShowModelTooltip] = useState(false);

  if (isZenMode) return null;

  return (
    <div className="frontstage-bottom-bar">
      <div className="frontstage-bottom-bar-inner">
        {/* 输入框 */}
        <div className="frontstage-input-pill">
          {/* 模型状态指示器 */}
          <div
            className="model-status-wrapper"
            onMouseEnter={() => setShowModelTooltip(true)}
            onMouseLeave={() => setShowModelTooltip(false)}
          >
            <div className={`model-status-dot status-${modelStatus}`} />
            {showModelTooltip && (
              <div className="model-tooltip">
                <div className="model-tooltip-header">
                  <span className="model-name">{modelName || '未配置'}</span>
                  <span className={`model-status-text status-${modelStatus}`}>
                    {modelStatus === 'connected' ? '已连接' : modelStatus === 'connecting' ? '检测中' : '未连接'}
                  </span>
                </div>
                <div className="model-id">{modelStatus === 'connected' ? '模型就绪，可直接输入指令' : '请检查模型配置'}</div>
              </div>
            )}
          </div>

          {/* 输入框 + Ghost Hint */}
          <div className="frontstage-input-middle">
            <div className="frontstage-input-ghost-wrapper">
              {ghostHint && !inputValue && (
                <span className="frontstage-input-ghost">
                  {ghostHint}
                  <span className="frontstage-input-ghost-hint">
                    {hintSource === 'llm' ? ' · →确认' : ' · ↑↓切换 · →确认'}
                  </span>
                </span>
              )}
              <textarea
                className="frontstage-input-textarea"
                placeholder={ghostHint ? '' : '输入任意指令…'}
                value={inputValue}
                onChange={(e) => onInputChange(e.target.value)}
                onKeyDown={onInputKeyDown}
                onFocus={onInputFocus}
                disabled={isGenerating}
                rows={1}
              />
            </div>
          </div>

          {isGenerating ? (
            <button
              className="frontstage-input-cancel"
              onClick={onCancelGeneration}
              title="取消生成"
            >
              <X className="w-4 h-4" />
            </button>
          ) : (
            <button
              className="frontstage-input-send"
              onClick={onInputSubmit}
              disabled={!inputValue.trim()}
              title="发送"
            >
              <Send className="w-4 h-4" />
            </button>
          )}
        </div>

        {/* 生成状态行 */}
        {isGenerating && generationStatus && (
          <div className="generation-status-row" title={generationStatus}>
            <StatusIcon text={generationStatus} />
          </div>
        )}
      </div>
    </div>
  );
};

export default React.memo(FrontstageBottomBar);
