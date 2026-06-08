import { loggedInvoke } from './core';
// ==================== v6.0.0: Reading Power ====================

export interface ReadingPowerEvaluation {
  chapter_number: number;
  hook_type: string | null;
  hook_strength: string;
  coolpoint_patterns: string[];
  micropayoffs: string[];
  hard_violations: string[];
  soft_suggestions: string[];
  is_transition: boolean;
  override_count: number;
  debt_balance: number;
  score: number;
}

export interface ChaseDebt {
  id: number;
  story_id: string;
  debt_type: string;
  original_amount: number;
  current_amount: number;
  interest_rate: number;
  source_chapter: number;
  due_chapter: number;
  override_contract_id: number | null;
  status: string;
  created_at: string;
}
// ==================== v6.0.0: Story Audit ====================

export interface StoryAnalysisReport {
  story_id: string;
  overall_score: number;
  dimensions: AuditDimension[];
  findings: AuditFinding[];
  recommendations: string[];
}

export interface AuditDimension {
  name: string;
  score: number;
  weight: number;
  description: string;
  details: string[];
}

export interface AuditFinding {
  severity: 'Critical' | 'Warning' | 'Info';
  category: string;
  message: string;
  suggestion: string;
}

export const auditStory = (storyId: string) =>
  loggedInvoke<StoryAnalysisReport>('audit_story', { story_id: storyId });
export interface OverrideContract {
  id: number;
  story_id: string;
  chapter_number: number;
  constraint_type: string;
  constraint_id: string;
  rationale_type: string;
  rationale_text: string;
  payback_plan: string;
  due_chapter: number;
  status: string;
  fulfilled_at: string | null;
  created_at: string;
}
export const evaluateReadingPower = (storyId: string, chapterNumber: number) =>
  loggedInvoke<ReadingPowerEvaluation>('evaluate_reading_power', {
    story_id: storyId,
    chapter_number: chapterNumber,
  });

export const getReadingPowerTrend = (storyId: string, lastN: number) =>
  loggedInvoke<ReadingPowerEvaluation[]>('get_reading_power_trend', {
    story_id: storyId,
    last_n: lastN,
  });

export const getChaseDebts = (storyId: string) =>
  loggedInvoke<ChaseDebt[]>('get_chase_debts', { story_id: storyId });

export const createOverrideContract = (params: {
  story_id: string;
  chapter_number: number;
  constraint_type: string;
  constraint_id: string;
  rationale_type: string;
  rationale_text: string;
  payback_plan: string;
  due_chapter: number;
}) => loggedInvoke<OverrideContract>('create_override_contract', params);

export interface GenreProfile {
  id: number;
  genre_name: string;
  canonical_name: string;
  aliases: string[];
  core_tone: string;
  pacing_strategy: string;
  anti_patterns: string[];
  reference_tables: string[];
  is_builtin: boolean;
  created_at: string;
}

export const getGenreProfiles = () => loggedInvoke<GenreProfile[]>('get_genre_profiles');

export const getGenreProfile = (genreName: string) =>
  loggedInvoke<GenreProfile | null>('get_genre_profile', { genre_name: genreName });
// ==================== v6.0.0: Anti-AI Review ====================

export interface AntiAiReview {
  overall_score: number;
  dimensions: DimensionScore[];
  issues: ReviewIssue[];
  suggestions: string[];
  flagged_passages: FlaggedPassage[];
}

export interface DimensionScore {
  name: string;
  score: number;
  weight: number;
  description: string;
}

export interface ReviewIssue {
  dimension: string;
  severity: string;
  description: string;
  example: string;
  suggestion: string;
}

export interface FlaggedPassage {
  text: string;
  dimension: string;
  reason: string;
  position: number;
}

export const antiAiReview = (text: string, genre?: string) =>
  loggedInvoke<AntiAiReview>('anti_ai_review', { text, genre });

export interface StyleDnaDelta {
  sentence_length_delta: number;
  dialogue_ratio_delta: number;
  metaphor_density_delta: number;
  interior_monologue_delta: number;
  emotion_density_delta: number;
  rhythm_score_delta: number;
  vocabulary_density_shift: string | null;
  expressiveness_shift: string | null;
  avoided_patterns_add: string[];
  reasons: string[];
}

export const evolveStyleFromAntiAiReview = (storyId: string, review: AntiAiReview) =>
  loggedInvoke<StyleDnaDelta>('evolve_style_from_anti_ai_review', { story_id: storyId, review });
