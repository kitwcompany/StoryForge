import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import useFrontstageGeneration from '../useFrontstageGeneration';

describe('useFrontstageGeneration', () => {
  it('should have correct default state', () => {
    const { result } = renderHook(() => useFrontstageGeneration());

    expect(result.current.isGenerating).toBe(false);
    expect(result.current.generationStatus).toBe('idle');
    expect(result.current.orchestratorStatus).toBe('idle');
    expect(result.current.bootstrapProgress).toBe(0);
  });

  it('should start generation', () => {
    const { result } = renderHook(() => useFrontstageGeneration());

    act(() => result.current.startGeneration());
    expect(result.current.isGenerating).toBe(true);
    expect(result.current.generationStatus).toBe('generating');
    expect(result.current.orchestratorStatus).toBe('planning');
  });

  it('should finish generation with success', () => {
    const { result } = renderHook(() => useFrontstageGeneration());

    act(() => result.current.startGeneration());
    act(() => result.current.finishGeneration('success'));
    expect(result.current.isGenerating).toBe(false);
    expect(result.current.generationStatus).toBe('success');
    expect(result.current.orchestratorStatus).toBe('reviewing');
  });

  it('should finish generation with error', () => {
    const { result } = renderHook(() => useFrontstageGeneration());

    act(() => result.current.startGeneration());
    act(() => result.current.finishGeneration('error'));
    expect(result.current.isGenerating).toBe(false);
    expect(result.current.generationStatus).toBe('error');
    expect(result.current.orchestratorStatus).toBe('error');
  });

  it('should set progress', () => {
    const { result } = renderHook(() => useFrontstageGeneration());

    act(() => result.current.setProgress(42));
    expect(result.current.bootstrapProgress).toBe(42);
  });

  it('should reset generation clearing all state', () => {
    const { result } = renderHook(() => useFrontstageGeneration());

    act(() => result.current.startGeneration());
    act(() => result.current.setProgress(50));
    act(() => result.current.resetGeneration());

    expect(result.current.isGenerating).toBe(false);
    expect(result.current.generationStatus).toBe('idle');
    expect(result.current.orchestratorStatus).toBe('idle');
    expect(result.current.bootstrapProgress).toBe(0);
  });
});
