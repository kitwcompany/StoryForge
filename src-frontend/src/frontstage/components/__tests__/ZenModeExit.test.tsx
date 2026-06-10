import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ZenModeExit from '../ZenModeExit';

describe('ZenModeExit', () => {
  it('should render button with text "退出禅模式 (F11)"', () => {
    render(<ZenModeExit onExit={vi.fn()} />);

    expect(screen.getByText('退出禅模式 (F11)')).toBeInTheDocument();
  });

  it('should call onExit when clicked', async () => {
    const onExit = vi.fn();
    render(<ZenModeExit onExit={onExit} />);

    const button = screen.getByRole('button');
    await userEvent.click(button);
    expect(onExit).toHaveBeenCalledTimes(1);
  });
});
