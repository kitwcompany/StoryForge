import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import HelpPanel from '../HelpPanel';

describe('HelpPanel', () => {
  it('should render shortcut sections', () => {
    render(<HelpPanel onClose={vi.fn()} />);

    expect(screen.getByText('写作')).toBeInTheDocument();
    expect(screen.getByText('模式')).toBeInTheDocument();
    expect(screen.getByText('操作')).toBeInTheDocument();
  });

  it('should render shortcut items', () => {
    render(<HelpPanel onClose={vi.fn()} />);

    expect(screen.getByText('AI 续写')).toBeInTheDocument();
    expect(screen.getByText('输入任意指令')).toBeInTheDocument();
    expect(screen.getByText('接受 AI 建议')).toBeInTheDocument();
    expect(screen.getByText('拒绝 AI 建议')).toBeInTheDocument();
    expect(screen.getByText('循环文思模式')).toBeInTheDocument();
    expect(screen.getByText('禅模式')).toBeInTheDocument();
    expect(screen.getByText('本帮助面板')).toBeInTheDocument();
    expect(screen.getAllByText('回幕后工作室')).toHaveLength(2);
    expect(screen.getByText('侧边栏快捷按钮')).toBeInTheDocument();
  });

  it('should call onClose when X button is clicked', async () => {
    const onClose = vi.fn();
    render(<HelpPanel onClose={onClose} />);

    const closeButton = screen.getByRole('button');
    await userEvent.click(closeButton);
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
