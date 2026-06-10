import { useState, useCallback } from 'react';

export default function useFrontstageEditor() {
  const [content, setContent] = useState('');
  const [isSaved, setIsSaved] = useState(true);
  const [fontSize, setFontSize] = useState(16);
  const [isZenMode, setIsZenMode] = useState(false);
  const [isRevisionMode, setIsRevisionMode] = useState(false);

  const increaseFontSize = useCallback(() => setFontSize(prev => Math.min(prev + 2, 32)), []);
  const decreaseFontSize = useCallback(() => setFontSize(prev => Math.max(prev - 2, 12)), []);

  const handleContentChange = useCallback((newContent: string) => {
    setContent(newContent);
    setIsSaved(false);
  }, []);

  const markSaved = useCallback(() => setIsSaved(true), []);

  const toggleZenMode = useCallback(() => setIsZenMode(prev => !prev), []);
  const toggleRevisionMode = useCallback(() => setIsRevisionMode(prev => !prev), []);

  return {
    content,
    setContent,
    isSaved,
    setIsSaved,
    fontSize,
    setFontSize,
    increaseFontSize,
    decreaseFontSize,
    isZenMode,
    setIsZenMode,
    toggleZenMode,
    isRevisionMode,
    setIsRevisionMode,
    toggleRevisionMode,
    handleContentChange,
    markSaved,
  };
}
