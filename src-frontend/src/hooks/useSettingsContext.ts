import { useContext } from 'react';
import { SettingsContext, type SettingsContextValue } from '@/contexts/settingsContextBase';

export function useSettingsContext(): SettingsContextValue {
  const ctx = useContext(SettingsContext);
  if (!ctx) {
    throw new Error('useSettingsContext must be used within a SettingsProvider');
  }
  return ctx;
}
