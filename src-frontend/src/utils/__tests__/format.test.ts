import { describe, it, expect } from 'vitest';
import { countWords, autoFormatText, formatDate, formatNumber, truncateText } from '../format';

describe('countWords', () => {
  it('should count Chinese characters', () => {
    expect(countWords('今天天气很好')).toBe(6);
  });

  it('should count English words', () => {
    expect(countWords('Hello world test')).toBe(3);
  });

  it('should count mixed Chinese and English text', () => {
    expect(countWords('Hello 世界 this 是 a 测试')).toBe(8);
  });

  it('should return 0 for empty string', () => {
    expect(countWords('')).toBe(0);
  });

  it('should count punctuation correctly', () => {
    expect(countWords('你好，世界！Hello world.')).toBe(6);
  });
});

describe('autoFormatText', () => {
  it('should return empty string for empty input', () => {
    expect(autoFormatText('')).toBe('');
    expect(autoFormatText('   ')).toBe('');
  });

  it('should format text with double newlines into paragraphs', () => {
    const input = '第一段内容。\n\n第二段内容。';
    const result = autoFormatText(input);
    expect(result).toContain('<p>');
    expect(result).toContain('</p>');
  });

  it('should normalize quotes in text', () => {
    const input = '"你好"';
    const result = autoFormatText(input);
    expect(result).toContain('「');
    expect(result).toContain('」');
  });

  it('should return empty string for whitespace-only input', () => {
    expect(autoFormatText('   \n\n   ')).toBe('');
  });
});

describe('formatDate', () => {
  it('should format date string to zh-CN locale', () => {
    const result = formatDate('2024-01-15');
    expect(result).toContain('2024');
    expect(result).toContain('15');
  });
});

describe('formatNumber', () => {
  it('should return number as string when below 1000', () => {
    expect(formatNumber(500)).toBe('500');
  });

  it('should format number with k when >= 1000', () => {
    expect(formatNumber(1500)).toBe('1.5k');
  });
});

describe('truncateText', () => {
  it('should return original text if within max length', () => {
    expect(truncateText('short', 10)).toBe('short');
  });

  it('should truncate text and append ellipsis', () => {
    expect(truncateText('hello world', 5)).toBe('hello...');
  });
});
