import { X } from 'lucide-react';

interface HelpPanelProps {
  onClose: () => void;
}

export default function HelpPanel({ onClose }: HelpPanelProps) {
  return (
    <div className="fixed top-16 left-1/2 -translate-x-1/2 z-50">
      <div className="frontstage-help-panel">
        <div className="frontstage-help-header">
          <span className="text-sm font-medium">快捷键指南</span>
          <button
            onClick={onClose}
            className="text-[var(--stone-gray)] hover:text-[var(--charcoal)] transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
        <div className="frontstage-help-body">
          <div className="frontstage-help-section">
            <h4>写作</h4>
            <div className="frontstage-help-row">
              <kbd>Ctrl</kbd>+<kbd>Enter</kbd>
              <span>AI 续写</span>
            </div>
            <div className="frontstage-help-row">
              <kbd>/</kbd>
              <span>输入任意指令</span>
            </div>
            <div className="frontstage-help-row">
              <kbd>Tab</kbd>
              <span>接受 AI 建议</span>
            </div>
            <div className="frontstage-help-row">
              <kbd>Esc</kbd>
              <span>拒绝 AI 建议</span>
            </div>
          </div>
          <div className="frontstage-help-section">
            <h4>模式</h4>
            <div className="frontstage-help-row">
              <kbd>Ctrl</kbd>+<kbd>Space</kbd>
              <span>循环文思模式</span>
            </div>
            <div className="frontstage-help-row">
              <kbd>F11</kbd>
              <span>禅模式</span>
            </div>
            <div className="frontstage-help-row">
              <kbd>F1</kbd>
              <span>本帮助面板</span>
            </div>
          </div>
          <div className="frontstage-help-section">
            <h4>操作</h4>
            <div className="frontstage-help-row">
              <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>B</kbd>
              <span>回幕后工作室</span>
            </div>
            <div className="frontstage-help-row">
              <span className="no-kbd">点击标题</span>
              <span>回幕后工作室</span>
            </div>
            <div className="frontstage-help-row">
              <span className="no-kbd">修 / 批 / 幕</span>
              <span>侧边栏快捷按钮</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
