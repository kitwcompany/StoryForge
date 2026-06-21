import { useRef, useCallback } from 'react';

/**
 * WebView 重绘修复 Hook
 *
 * Tauri/WebKit 在窗口恢复/切换时可能出现渲染不更新的问题。
 * 此 hook 提供强制重绘机制，封装了 opacity 抖动 hack。
 *
 * 背景: v5.2.0/W2-F3 引入，用于解决 WebView2 compositor 在窗口切换后不刷新的问题。
 * FIXME: 仍是临时 hack，需在真实窗口恢复/切换场景验证无白屏后再移除。
 */
export function useWebViewRedrawFix() {
  const mainRef = useRef<HTMLElement>(null);

  const trigger = useCallback((el: HTMLElement | null) => {
    if (!el) return;
    // 极短的 opacity 抖动强制 WebView compositor 刷新
    el.style.opacity = '0.99';
    requestAnimationFrame(() => {
      el.style.opacity = '';
    });
  }, []);

  const forceRedraw = useCallback(() => {
    window.dispatchEvent(new Event('resize'));
    window.dispatchEvent(new Event('scroll'));
    trigger(mainRef.current);
    setTimeout(() => {
      window.dispatchEvent(new Event('resize'));
      trigger(mainRef.current);
    }, 300);
  }, [trigger]);

  return { mainRef, forceRedraw };
}

export default useWebViewRedrawFix;
