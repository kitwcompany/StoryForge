import { useCallback, useRef } from 'react';

/**
 * 返回一个防抖函数。
 *
 * @param fn 要防抖的函数
 * @param delay 防抖延迟（毫秒）
 */
export function useDebounceCallback<T extends (...args: Parameters<T>) => void>(
  fn: T,
  delay: number
): T {
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  return useCallback(
    ((...args: Parameters<T>) => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
      timerRef.current = setTimeout(() => {
        fn(...args);
      }, delay);
    }) as T,
    [fn, delay]
  );
}
