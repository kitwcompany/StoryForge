import { loggedInvoke } from './core';
// ==================== v6.0.0: Memory System ====================

export interface MemoryPack {
  working_memory: MemoryEntry[];
  episodic_memory: MemoryEntry[];
  semantic_memory: MemoryItemDto[];
  long_term_facts: MemoryItemDto[];
  active_constraints: string[];
  recent_changes: string[];
  warnings: MemoryWarning[];
  stats: MemoryStats;
}

export interface MemoryEntry {
  subject: string;
  field: string;
  value: string;
  source_chapter: number;
}

export interface MemoryItemDto {
  id: string;
  category: string;
  subject: string | null;
  field: string | null;
  value: string | null;
  source_chapter: number | null;
  confidence: number;
}

export interface MemoryWarning {
  category: string;
  subject: string;
  count: number;
}

export interface MemoryStats {
  total: number;
  working_total: number;
  episodic_total: number;
  semantic_total: number;
  injected: number;
  layered_total_injected: number;
  filtered: number;
  conflicts: number;
}

export interface MemoryItem {
  id: string;
  story_id: string;
  category: string;
  subject: string | null;
  field: string | null;
  value: string | null;
  source_chapter: number | null;
  confidence: number;
  status: string;
  updated_at: string;
}

export const buildMemoryPack = (
  storyId: string,
  chapterNumber: number,
  taskType: string,
  outline?: string
) =>
  loggedInvoke<MemoryPack>('build_memory_pack', {
    story_id: storyId,
    chapter_number: chapterNumber,
    task_type: taskType,
    outline,
  });

export const getMemoryItems = (storyId: string) =>
  loggedInvoke<MemoryItem[]>('get_memory_items', { story_id: storyId });

export const createMemoryItem = (params: {
  story_id: string;
  category: string;
  subject?: string;
  field?: string;
  value?: string;
  source_chapter?: number;
  confidence: number;
}) => loggedInvoke<MemoryItem>('create_memory_item', params);
