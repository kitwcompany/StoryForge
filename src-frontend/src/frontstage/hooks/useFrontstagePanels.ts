import { useState, useCallback } from 'react';

export interface AiLearning {
  category: string;
  insight: string;
  confidence: number;
}

export type UpgradeTrigger = 'wensi' | 'generation' | 'analysis' | 'batch' | 'custom';

export default function useFrontstagePanels() {
  const [showHelpPanel, setShowHelpPanel] = useState(false);
  const [showUpgradePanel, setShowUpgradePanel] = useState(false);
  const [upgradeTrigger, setUpgradeTrigger] = useState<UpgradeTrigger>('generation');
  const [learnings, setLearnings] = useState<AiLearning[]>([]);

  const toggleHelpPanel = useCallback(() => {
    setShowHelpPanel(prev => !prev);
  }, []);

  const openUpgradePanel = useCallback((trigger: UpgradeTrigger = 'generation') => {
    setUpgradeTrigger(trigger);
    setShowUpgradePanel(true);
  }, []);

  const closeUpgradePanel = useCallback(() => {
    setShowUpgradePanel(false);
  }, []);

  const dismissLearnings = useCallback(() => {
    setLearnings([]);
  }, []);

  const addLearning = useCallback((learning: AiLearning) => {
    setLearnings(prev => [...prev, learning]);
  }, []);

  const removeLearning = useCallback((index: number) => {
    setLearnings(prev => prev.filter((_, i) => i !== index));
  }, []);

  return {
    showHelpPanel,
    setShowHelpPanel,
    toggleHelpPanel,
    showUpgradePanel,
    setShowUpgradePanel,
    openUpgradePanel,
    closeUpgradePanel,
    upgradeTrigger,
    setUpgradeTrigger,
    learnings,
    setLearnings,
    dismissLearnings,
    addLearning,
    removeLearning,
  };
}
