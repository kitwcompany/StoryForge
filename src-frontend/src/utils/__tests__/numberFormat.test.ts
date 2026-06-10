import { describe, it, expect } from 'vitest';
import {
  normalizeFloat,
  normalizeInt,
  formatDisplayFloat,
  clampNumber,
  formatLatencyWithQuality,
} from '../numberFormat';

describe('normalizeFloat', () => {
  it('should normalize float with normal input', () => {
    expect(normalizeFloat(3.14159, 2)).toBe(3.14);
    expect(normalizeFloat(2.5, 1)).toBe(2.5);
    expect(normalizeFloat(1.999, 2)).toBe(2.0);
  });

  it('should return 0 for NaN', () => {
    expect(normalizeFloat(NaN)).toBe(0);
  });

  it('should return 0 for Infinity', () => {
    expect(normalizeFloat(Infinity)).toBe(0);
    expect(normalizeFloat(-Infinity)).toBe(0);
  });

  it('should use default decimals of 2', () => {
    expect(normalizeFloat(1.234)).toBe(1.23);
  });
});

describe('clampNumber', () => {
  it('should return value within bounds', () => {
    expect(clampNumber(5, 0, 10)).toBe(5);
  });

  it('should clamp to min boundary', () => {
    expect(clampNumber(-5, 0, 10)).toBe(0);
  });

  it('should clamp to max boundary', () => {
    expect(clampNumber(15, 0, 10)).toBe(10);
  });

  it('should return min for NaN', () => {
    expect(clampNumber(NaN, 0, 10)).toBe(0);
  });

  it('should return min for non-finite values', () => {
    expect(clampNumber(Infinity, 0, 10)).toBe(0);
    expect(clampNumber(-Infinity, 0, 10)).toBe(0);
  });
});

describe('formatDisplayFloat', () => {
  it('should format normal float and strip trailing zeros', () => {
    expect(formatDisplayFloat(3.14, 2)).toBe('3.14');
    expect(formatDisplayFloat(2.0, 2)).toBe('2');
  });

  it('should return "0" for NaN', () => {
    expect(formatDisplayFloat(NaN)).toBe('0');
  });

  it('should return "0" for Infinity', () => {
    expect(formatDisplayFloat(Infinity)).toBe('0');
  });
});

describe('normalizeInt', () => {
  it('should round to nearest integer', () => {
    expect(normalizeInt(3.7)).toBe(4);
    expect(normalizeInt(3.2)).toBe(3);
  });

  it('should clamp to min and max', () => {
    expect(normalizeInt(5, 10, 20)).toBe(10);
    expect(normalizeInt(25, 10, 20)).toBe(20);
  });

  it('should return 0 for NaN', () => {
    expect(normalizeInt(NaN)).toBe(0);
  });
});

describe('formatLatencyWithQuality', () => {
  it('should return quality for excellent latency', () => {
    expect(formatLatencyWithQuality(50)).toBe('50ms · 优秀');
  });

  it('should return quality for good latency', () => {
    expect(formatLatencyWithQuality(200)).toBe('200ms · 良好');
  });

  it('should return quality for average latency', () => {
    expect(formatLatencyWithQuality(500)).toBe('500ms · 一般');
  });

  it('should return unknown for zero or negative', () => {
    expect(formatLatencyWithQuality(0)).toBe('未知');
    expect(formatLatencyWithQuality(-1)).toBe('未知');
  });
});
