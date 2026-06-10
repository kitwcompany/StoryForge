import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import useFrontstageWensi from '../useFrontstageWensi';

describe('useFrontstageWensi', () => {
  it('should have correct default state', () => {
    const { result } = renderHook(() => useFrontstageWensi());

    expect(result.current.wensiMode).toBe('off');
    expect(result.current.showWenSiPanel).toBe(false);
    expect(result.current.wenSiTab).toBe('suggestions');
    expect(result.current.smartGhostText).toBe('');
  });

  it('should cycle wensi mode: off → rhythm → flow → burst → off', () => {
    const { result } = renderHook(() => useFrontstageWensi());

    act(() => result.current.cycleWensiMode());
    expect(result.current.wensiMode).toBe('rhythm');

    act(() => result.current.cycleWensiMode());
    expect(result.current.wensiMode).toBe('flow');

    act(() => result.current.cycleWensiMode());
    expect(result.current.wensiMode).toBe('burst');

    act(() => result.current.cycleWensiMode());
    expect(result.current.wensiMode).toBe('off');
  });

  it('should open wensi panel with specified tab', () => {
    const { result } = renderHook(() => useFrontstageWensi());

    act(() => result.current.openWensiPanel('analysis'));
    expect(result.current.showWenSiPanel).toBe(true);
    expect(result.current.wenSiTab).toBe('analysis');

    act(() => result.current.openWensiPanel('history'));
    expect(result.current.showWenSiPanel).toBe(true);
    expect(result.current.wenSiTab).toBe('history');
  });

  it('should open wensi panel without changing tab when no argument', () => {
    const { result } = renderHook(() => useFrontstageWensi());

    act(() => result.current.setWenSiTab('analysis'));
    act(() => result.current.openWensiPanel());
    expect(result.current.showWenSiPanel).toBe(true);
    expect(result.current.wenSiTab).toBe('analysis');
  });

  it('should set smart ghost text', () => {
    const { result } = renderHook(() => useFrontstageWensi());

    act(() => result.current.setSmartGhostText('ghost hint'));
    expect(result.current.smartGhostText).toBe('ghost hint');
  });

  it('should close wensi panel', () => {
    const { result } = renderHook(() => useFrontstageWensi());

    act(() => result.current.openWensiPanel());
    expect(result.current.showWenSiPanel).toBe(true);

    act(() => result.current.closeWensiPanel());
    expect(result.current.showWenSiPanel).toBe(false);
  });
});
