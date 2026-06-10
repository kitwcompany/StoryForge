import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import useFrontstageEditor from '../useFrontstageEditor';

describe('useFrontstageEditor', () => {
  it('should have correct default state', () => {
    const { result } = renderHook(() => useFrontstageEditor());

    expect(result.current.content).toBe('');
    expect(result.current.isSaved).toBe(true);
    expect(result.current.fontSize).toBe(16);
    expect(result.current.isZenMode).toBe(false);
    expect(result.current.isRevisionMode).toBe(false);
  });

  it('should handle content change and set isSaved to false', () => {
    const { result } = renderHook(() => useFrontstageEditor());

    act(() => result.current.handleContentChange('new content'));
    expect(result.current.content).toBe('new content');
    expect(result.current.isSaved).toBe(false);
  });

  it('should increase font size and clamp at 32', () => {
    const { result } = renderHook(() => useFrontstageEditor());

    act(() => result.current.setFontSize(30));
    act(() => result.current.increaseFontSize());
    expect(result.current.fontSize).toBe(32);

    act(() => result.current.increaseFontSize());
    expect(result.current.fontSize).toBe(32);
  });

  it('should decrease font size and clamp at 12', () => {
    const { result } = renderHook(() => useFrontstageEditor());

    act(() => result.current.setFontSize(14));
    act(() => result.current.decreaseFontSize());
    expect(result.current.fontSize).toBe(12);

    act(() => result.current.decreaseFontSize());
    expect(result.current.fontSize).toBe(12);
  });

  it('should toggle zen mode', () => {
    const { result } = renderHook(() => useFrontstageEditor());

    act(() => result.current.toggleZenMode());
    expect(result.current.isZenMode).toBe(true);

    act(() => result.current.toggleZenMode());
    expect(result.current.isZenMode).toBe(false);
  });

  it('should toggle revision mode', () => {
    const { result } = renderHook(() => useFrontstageEditor());

    act(() => result.current.toggleRevisionMode());
    expect(result.current.isRevisionMode).toBe(true);

    act(() => result.current.toggleRevisionMode());
    expect(result.current.isRevisionMode).toBe(false);
  });

  it('should mark saved', () => {
    const { result } = renderHook(() => useFrontstageEditor());

    act(() => result.current.handleContentChange('content'));
    expect(result.current.isSaved).toBe(false);

    act(() => result.current.markSaved());
    expect(result.current.isSaved).toBe(true);
  });
});
