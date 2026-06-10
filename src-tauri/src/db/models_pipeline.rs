use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== Export Template Models ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub format: String,
    pub template_content: String,
    pub is_builtin: bool,
    pub is_user_created: bool,
    pub created_at: DateTime<Local>,
}

// ==================== AI Operation History Models ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOperation {
    pub id: String,
    pub story_id: String,
    pub scene_id: Option<String>,
    pub chapter_id: Option<String>,
    pub operation_type: String,
    pub operation_name: String,
    pub input_summary: Option<String>,
    pub output_summary: Option<String>,
    pub previous_content: Option<String>,
    pub new_content: Option<String>,
    pub metadata: Option<String>,
    pub status: String,
    pub created_at: DateTime<Local>,
}

// ==================== Story System Models ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryContract {
    pub id: String,
    pub story_id: String,
    pub contract_type: String, // MASTER_SETTING | VOLUME | CHAPTER | REVIEW
    pub contract_json: String,
    pub version: i32,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneCommit {
    pub id: String,
    pub story_id: String,
    pub scene_id: Option<String>,
    pub chapter_id: Option<String>,
    pub chapter_number: i32,
    pub status: String, // pending | accepted | rejected
    pub outline_snapshot_json: Option<String>,
    pub review_result_json: Option<String>,
    pub fulfillment_result_json: Option<String>,
    pub accepted_events_json: Option<String>,
    pub state_deltas_json: Option<String>,
    pub entity_deltas_json: Option<String>,
    pub summary_text: Option<String>,
    pub dominant_strand: Option<String>,
    pub projection_status_json: Option<String>,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisRun {
    pub id: String,
    pub story_id: Option<String>,
    pub session_id: String,
    pub premise: String,
    pub status: String, // pending | running | paused | completed | failed
    pub current_step: Option<String>,
    pub current_step_number: i32,
    pub total_steps: i32,
    pub steps_json: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub story_id: String,
    pub category: String, /* world_rule | character_state | relationship | story_fact |
                           * open_loop | reader_promise | timeline */
    pub subject: Option<String>,
    pub field: Option<String>,
    pub value: Option<String>,
    pub source_chapter: Option<i32>,
    pub confidence: f32,
    pub status: String, // active | archived | conflicting
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterReadingPower {
    pub id: String,
    pub story_id: String,
    pub scene_id: Option<String>,
    pub chapter_number: i32,
    pub hook_type: Option<String>,
    pub hook_strength: String,
    pub coolpoint_patterns_json: Option<String>,
    pub micropayoffs_json: Option<String>,
    pub hard_violations_json: Option<String>,
    pub soft_suggestions_json: Option<String>,
    pub is_transition: bool,
    pub override_count: i32,
    pub debt_balance: f64,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaseDebt {
    pub id: i64,
    pub story_id: String,
    pub debt_type: String,
    pub original_amount: f64,
    pub current_amount: f64,
    pub interest_rate: f64,
    pub source_chapter: i32,
    pub due_chapter: i32,
    pub override_contract_id: Option<i64>,
    pub status: String, // active | paid | overdue | written_off
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideContract {
    pub id: i64,
    pub story_id: String,
    pub chapter_number: i32,
    pub constraint_type: String,
    pub constraint_id: String,
    pub rationale_type: String,
    pub rationale_text: String,
    pub payback_plan: String,
    pub due_chapter: i32,
    pub status: String, // pending | fulfilled | overdue | cancelled
    pub fulfilled_at: Option<DateTime<Local>>,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    pub id: String,
    pub story_id: String,
    pub scene_id: Option<String>,
    pub chapter_number: i32,
    pub severity: String, // critical | high | medium | low
    pub category: String, /* continuity | setting | character | timeline | ai_flavor | logic |
                           * pacing | other */
    pub location: Option<String>,
    pub description: String,
    pub evidence: Option<String>,
    pub fix_hint: Option<String>,
    pub blocking: bool,
    pub resolved: bool,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreProfile {
    pub id: String,
    pub genre_name: String,
    pub canonical_name: String,
    pub aliases_json: Option<String>,
    pub core_tone: Option<String>,
    pub pacing_strategy: Option<String>,
    pub anti_patterns_json: Option<String>,
    pub reference_tables_json: Option<String>,
    pub is_builtin: bool,
    pub created_at: DateTime<Local>,
}

// ==================== Pipeline 模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub id: String,
    pub story_id: String,
    pub chapter_number: i32,
    pub title: Option<String>,
    pub role: Option<String>,
    pub purpose: Option<String>,
    pub key_events: Option<String>,
    pub characters: Option<String>,
    pub suspense_hook: Option<String>,
    pub user_guidance: Option<String>,
    pub notes: Option<String>,
    pub notes_updated_at: Option<DateTime<Local>>,
    pub knowledge_query_hint: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// --- Draft ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draft {
    pub id: String,
    pub story_id: String,
    pub chapter_number: i32,
    pub version: i32,
    pub status: DraftStatus,
    pub source: DraftSource,
    pub content: String,
    pub word_count: i32,
    pub model_used: Option<String>,
    pub cost: Option<f64>,
    pub metadata: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraftStatus {
    Draft,
    Refined,
    Reviewed,
    Finalized,
    Archived,
}

impl std::fmt::Display for DraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DraftStatus::Draft => write!(f, "draft"),
            DraftStatus::Refined => write!(f, "refined"),
            DraftStatus::Reviewed => write!(f, "reviewed"),
            DraftStatus::Finalized => write!(f, "finalized"),
            DraftStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for DraftStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(DraftStatus::Draft),
            "refined" => Ok(DraftStatus::Refined),
            "reviewed" => Ok(DraftStatus::Reviewed),
            "finalized" => Ok(DraftStatus::Finalized),
            "archived" => Ok(DraftStatus::Archived),
            _ => Err(format!("Unknown draft status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraftSource {
    Write,
    Rewrite,
    Refine,
    ReviewFix,
}

impl std::fmt::Display for DraftSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DraftSource::Write => write!(f, "write"),
            DraftSource::Rewrite => write!(f, "rewrite"),
            DraftSource::Refine => write!(f, "refine"),
            DraftSource::ReviewFix => write!(f, "review_fix"),
        }
    }
}

impl std::str::FromStr for DraftSource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "write" => Ok(DraftSource::Write),
            "rewrite" => Ok(DraftSource::Rewrite),
            "refine" => Ok(DraftSource::Refine),
            "review_fix" => Ok(DraftSource::ReviewFix),
            _ => Err(format!("Unknown draft source: {}", s)),
        }
    }
}

// --- Revision ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub id: String,
    pub story_id: String,
    pub draft_id: String,
    pub revision_index: i32,
    pub revision_type: RevisionType,
    pub status: RevisionStatus,
    pub user_prompt: Option<String>,
    pub original_content: String,
    pub revised_content: String,
    pub word_count: i32,
    pub change_summary: Option<String>,
    pub model_used: Option<String>,
    pub cost: Option<f64>,
    pub metadata: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevisionType {
    Refine,
    ReviewFix,
    UserEdit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevisionStatus {
    Pending,
    Merged,
    Discarded,
    Superseded,
}

impl std::fmt::Display for RevisionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RevisionType::Refine => write!(f, "refine"),
            RevisionType::ReviewFix => write!(f, "review_fix"),
            RevisionType::UserEdit => write!(f, "user_edit"),
        }
    }
}

impl std::str::FromStr for RevisionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "refine" => Ok(RevisionType::Refine),
            "review_fix" => Ok(RevisionType::ReviewFix),
            "user_edit" => Ok(RevisionType::UserEdit),
            _ => Err(format!("Unknown revision type: {}", s)),
        }
    }
}

impl std::fmt::Display for RevisionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RevisionStatus::Pending => write!(f, "pending"),
            RevisionStatus::Merged => write!(f, "merged"),
            RevisionStatus::Discarded => write!(f, "discarded"),
            RevisionStatus::Superseded => write!(f, "superseded"),
        }
    }
}

impl std::str::FromStr for RevisionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(RevisionStatus::Pending),
            "merged" => Ok(RevisionStatus::Merged),
            "discarded" => Ok(RevisionStatus::Discarded),
            "superseded" => Ok(RevisionStatus::Superseded),
            _ => Err(format!("Unknown revision status: {}", s)),
        }
    }
}

// --- Pipeline Review (审稿报告) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineReview {
    pub id: String,
    pub story_id: String,
    pub draft_id: String,
    pub review_index: i32,
    pub content: String,
    pub dimensions: Option<String>,
    pub issues: Option<String>,
    pub overall_score: Option<f32>,
    pub review_focus: Option<String>,
    pub model_used: Option<String>,
    pub cost: Option<f64>,
    pub metadata: Option<String>,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDimension {
    pub name: String,
    pub score: f32,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssueItem {
    pub severity: String,
    pub dimension: String,
    pub description: String,
    pub suggestion: String,
}

// --- Post Process ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessRun {
    pub id: String,
    pub story_id: String,
    pub chapter_number: i32,
    pub source_label: String,
    pub scope: Option<String>,
    pub status: PostProcessStatus,
    pub started_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostProcessStep {
    pub id: String,
    pub run_id: String,
    pub step_key: String,
    pub step_label: String,
    pub status: StepStatus,
    pub critical: bool,
    pub log_output: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Local>>,
    pub completed_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostProcessStatus {
    Running,
    Completed,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

impl std::fmt::Display for PostProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostProcessStatus::Running => write!(f, "running"),
            PostProcessStatus::Completed => write!(f, "completed"),
            PostProcessStatus::Failed => write!(f, "failed"),
            PostProcessStatus::Partial => write!(f, "partial"),
        }
    }
}

impl std::str::FromStr for PostProcessStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(PostProcessStatus::Running),
            "completed" => Ok(PostProcessStatus::Completed),
            "failed" => Ok(PostProcessStatus::Failed),
            "partial" => Ok(PostProcessStatus::Partial),
            _ => Err(format!("Unknown post process status: {}", s)),
        }
    }
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Pending => write!(f, "pending"),
            StepStatus::Running => write!(f, "running"),
            StepStatus::Success => write!(f, "success"),
            StepStatus::Failed => write!(f, "failed"),
            StepStatus::Skipped => write!(f, "skipped"),
        }
    }
}

impl std::str::FromStr for StepStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(StepStatus::Pending),
            "running" => Ok(StepStatus::Running),
            "success" => Ok(StepStatus::Success),
            "failed" => Ok(StepStatus::Failed),
            "skipped" => Ok(StepStatus::Skipped),
            _ => Err(format!("Unknown step status: {}", s)),
        }
    }
}

// --- LLM Call ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCall {
    pub id: String,
    pub story_id: Option<String>,
    pub draft_id: Option<String>,
    pub revision_id: Option<String>,
    pub model_id: String,
    pub model_name: Option<String>,
    pub purpose: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub duration_ms: i32,
    pub success: bool,
    pub error_message: Option<String>,
    pub prompt_preview: Option<String>,
    pub metadata: Option<String>,
    pub created_at: DateTime<Local>,
}

// --- Character State ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    pub location: Option<String>,
    pub power_level: Option<String>,
    pub physical_state: Option<String>,
    pub mental_state: Option<String>,
    pub key_items: Option<String>,
    pub recent_events: Option<String>,
    pub updated_at_chapter: Option<i32>,

    // LitSeg: 角色弧光追踪（从 narrative_threads.character_arc 合并）
    pub state_transitions_json: Option<String>,
    pub arc_type: Option<String>,
}

// ==================== LitSeg 深度融合模型 ====================
// 注意: narrative_events / narrative_threads / narrative_structure
// 已合并到现有表:
//   - narrative_events → scenes 表 (narrative_* 字段)
//   - narrative_threads.foreshadow → foreshadowing_tracker (event_id + score
//     字段)
//   - narrative_threads.character_arc → character_states (transitions +
//     arc_type 字段)
//   - narrative_threads.conflict_escalation → conflict_escalations 表
//   - narrative_structure → story_outlines.analyzed_structure_json

/// 叙事结构定位表 — 保留，提供场景级精细定位（新增能力）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeStructurePosition {
    pub id: String,
    pub story_id: String,
    pub event_id: String,
    pub act_number: i32,
    pub act_type: String,
    pub position_in_act: f32,
    pub dramatic_function: String,
    pub is_narrative_boundary: bool,
    pub created_at: String,
}

/// 冲突升级追踪表 — 从 narrative_threads.conflict_escalation 提取为独立结构化表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictEscalation {
    pub id: String,
    pub story_id: String,
    pub conflict_type: String,
    pub party_a_ids: String,
    pub party_b_ids: String,
    pub intensity_timeline_json: String,
    pub current_intensity: f32,
    pub is_escalated: bool,
    pub created_at: String,
}

/// 叙事感知文本块表（SQLite 索引 + LanceDB vector_id）— 保留为物化缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeChunk {
    pub id: String,
    pub story_id: String,
    pub chapter_range_start: i32,
    pub chapter_range_end: i32,
    pub scene_ids: String,
    pub event_ids: String,
    pub text: String,
    pub chunk_type: String,
    pub is_boundary_start: bool,
    pub is_boundary_end: bool,
    pub thread_ids: String,
    pub vector_id: Option<String>,
    pub created_at: String,
}
