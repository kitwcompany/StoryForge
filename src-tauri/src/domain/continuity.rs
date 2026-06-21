//! Continuity domain types.
//!
//! Shared data structures for story continuity checking.
//! The engine implementation remains in `crate::creative_engine::continuity`.

/// 角色当前状态
#[derive(Debug, Clone)]
pub struct CharacterState {
    pub character_id: String,
    pub name: String,
    pub current_location: Option<String>,
    pub current_emotion: Option<String>,
    pub active_goal: Option<String>,
    pub secrets_known: Vec<String>,
    pub secrets_unknown: Vec<String>,
    pub arc_progress: f32, // 0.0 - 1.0
}

/// 连续性检查结果
#[derive(Debug, Clone)]
pub struct ConsistencyCheck {
    pub is_valid: bool,
    pub issues: Vec<ConsistencyIssue>,
}

/// 一致性问题
#[derive(Debug, Clone)]
pub struct ConsistencyIssue {
    pub issue_type: IssueType,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IssueType {
    CharacterLocation,
    CharacterEmotion,
    TimelineConflict,
    WorldRuleViolation,
    RelationshipInconsistency,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}
