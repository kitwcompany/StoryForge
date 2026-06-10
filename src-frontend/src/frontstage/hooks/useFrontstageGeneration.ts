import { useState, useCallback } from 'react';

type GenerationStatus = 'idle' | 'generating' | 'success' | 'error';
type OrchestratorStatus = 'idle' | 'planning' | 'executing' | 'reviewing' | 'error';

export default function useFrontstageGeneration() {
  const [isGenerating, setIsGenerating] = useState(false);
  const [generationStatus, setGenerationStatus] = useState<GenerationStatus>('idle');
  const [orchestratorStatus, setOrchestratorStatus] = useState<OrchestratorStatus>('idle');
  const [bootstrapProgress, setBootstrapProgress] = useState(0);

  const startGeneration = useCallback(() => {
    setIsGenerating(true);
    setGenerationStatus('generating');
    setOrchestratorStatus('planning');
  }, []);

  const finishGeneration = useCallback((status: GenerationStatus) => {
    setIsGenerating(false);
    setGenerationStatus(status);
    if (status === 'success') {
      setOrchestratorStatus('reviewing');
    } else {
      setOrchestratorStatus(status === 'error' ? 'error' : 'idle');
    }
  }, []);

  const setProgress = useCallback((progress: number) => {
    setBootstrapProgress(progress);
  }, []);

  const resetGeneration = useCallback(() => {
    setIsGenerating(false);
    setGenerationStatus('idle');
    setOrchestratorStatus('idle');
    setBootstrapProgress(0);
  }, []);

  return {
    isGenerating,
    generationStatus,
    orchestratorStatus,
    bootstrapProgress,
    startGeneration,
    finishGeneration,
    setProgress,
    resetGeneration,
  };
}
