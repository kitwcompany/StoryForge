import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import useFrontstagePanels from '../useFrontstagePanels';

describe('useFrontstagePanels', () => {
  it('should have correct default state', () => {
    const { result } = renderHook(() => useFrontstagePanels());

    expect(result.current.showHelpPanel).toBe(false);
    expect(result.current.showUpgradePanel).toBe(false);
    expect(result.current.upgradeTrigger).toBe('generation');
    expect(result.current.learnings).toEqual([]);
  });

  it('should toggle help panel', () => {
    const { result } = renderHook(() => useFrontstagePanels());

    act(() => result.current.toggleHelpPanel());
    expect(result.current.showHelpPanel).toBe(true);

    act(() => result.current.toggleHelpPanel());
    expect(result.current.showHelpPanel).toBe(false);
  });

  it('should open upgrade panel with trigger', () => {
    const { result } = renderHook(() => useFrontstagePanels());

    act(() => result.current.openUpgradePanel('wensi'));
    expect(result.current.showUpgradePanel).toBe(true);
    expect(result.current.upgradeTrigger).toBe('wensi');

    act(() => result.current.openUpgradePanel('analysis'));
    expect(result.current.showUpgradePanel).toBe(true);
    expect(result.current.upgradeTrigger).toBe('analysis');
  });

  it('should open upgrade panel with default trigger', () => {
    const { result } = renderHook(() => useFrontstagePanels());

    act(() => result.current.openUpgradePanel());
    expect(result.current.showUpgradePanel).toBe(true);
    expect(result.current.upgradeTrigger).toBe('generation');
  });

  it('should add learning', () => {
    const { result } = renderHook(() => useFrontstagePanels());

    const learning = { category: 'style', insight: 'insight1', confidence: 0.9 };
    act(() => result.current.addLearning(learning));
    expect(result.current.learnings).toHaveLength(1);
    expect(result.current.learnings[0]).toEqual(learning);
  });

  it('should remove learning by index', () => {
    const { result } = renderHook(() => useFrontstagePanels());

    act(() => result.current.addLearning({ category: 'a', insight: 'i1', confidence: 0.5 }));
    act(() => result.current.addLearning({ category: 'b', insight: 'i2', confidence: 0.8 }));
    act(() => result.current.removeLearning(0));

    expect(result.current.learnings).toHaveLength(1);
    expect(result.current.learnings[0].category).toBe('b');
  });

  it('should dismiss all learnings', () => {
    const { result } = renderHook(() => useFrontstagePanels());

    act(() => result.current.addLearning({ category: 'a', insight: 'i1', confidence: 0.5 }));
    act(() => result.current.addLearning({ category: 'b', insight: 'i2', confidence: 0.8 }));
    act(() => result.current.dismissLearnings());

    expect(result.current.learnings).toEqual([]);
  });

  it('should close upgrade panel', () => {
    const { result } = renderHook(() => useFrontstagePanels());

    act(() => result.current.openUpgradePanel());
    expect(result.current.showUpgradePanel).toBe(true);

    act(() => result.current.closeUpgradePanel());
    expect(result.current.showUpgradePanel).toBe(false);
  });
});
