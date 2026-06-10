import { useState, useCallback } from 'react';

type WensiMode = 'off' | 'rhythm' | 'flow' | 'burst';
type WensiTab = 'suggestions' | 'analysis' | 'history';

export default function useFrontstageWensi() {
  const [wensiMode, setWensiMode] = useState<WensiMode>('off');
  const [showWenSiPanel, setShowWenSiPanel] = useState(false);
  const [wenSiTab, setWenSiTab] = useState<WensiTab>('suggestions');
  const [smartGhostText, setSmartGhostText] = useState('');

  const cycleWensiMode = useCallback(() => {
    setWensiMode(prev => {
      const modes: WensiMode[] = ['off', 'rhythm', 'flow', 'burst'];
      const idx = modes.indexOf(prev);
      return modes[(idx + 1) % modes.length];
    });
  }, []);

  const openWensiPanel = useCallback((tab?: WensiTab) => {
    if (tab) setWenSiTab(tab);
    setShowWenSiPanel(true);
  }, []);

  const closeWensiPanel = useCallback(() => {
    setShowWenSiPanel(false);
  }, []);

  return {
    wensiMode,
    setWensiMode,
    cycleWensiMode,
    showWenSiPanel,
    setShowWenSiPanel,
    openWensiPanel,
    closeWensiPanel,
    wenSiTab,
    setWenSiTab,
    smartGhostText,
    setSmartGhostText,
  };
}
