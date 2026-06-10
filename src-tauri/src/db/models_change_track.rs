use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== 变更追踪模型 (修订模式) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTrack {
    pub id: String,
    pub scene_id: Option<String>,
    pub chapter_id: Option<String>,
    pub version_id: Option<String>,
    pub author_id: String,
    pub author_name: Option<String>,
    pub change_type: ChangeType,
    pub from_pos: i32,
    pub to_pos: i32,
    pub content: Option<String>,
    pub status: ChangeStatus,
    pub created_at: DateTime<Local>,
    pub resolved_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Insert,
    Delete,
    Format,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeStatus {
    Pending,
    Accepted,
    Rejected,
}

impl ChangeTrack {
    pub fn new(
        scene_id: Option<String>,
        chapter_id: Option<String>,
        author_id: String,
        change_type: ChangeType,
        from_pos: i32,
        to_pos: i32,
        content: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            scene_id,
            chapter_id,
            version_id: None,
            author_id,
            author_name: None,
            change_type,
            from_pos,
            to_pos,
            content,
            status: ChangeStatus::Pending,
            created_at: Local::now(),
            resolved_at: None,
        }
    }
}

// ==================== 评论线程模型 (修订模式) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentThread {
    pub id: String,
    pub scene_id: Option<String>,
    pub chapter_id: Option<String>,
    pub version_id: Option<String>,
    pub anchor_type: AnchorType,
    pub from_pos: Option<i32>,
    pub to_pos: Option<i32>,
    pub selected_text: Option<String>,
    pub status: ThreadStatus,
    pub created_at: DateTime<Local>,
    pub resolved_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentMessage {
    pub id: String,
    pub thread_id: String,
    pub author_id: String,
    pub author_name: Option<String>,
    pub content: String,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnchorType {
    TextRange,
    SceneLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreadStatus {
    Open,
    Resolved,
}

impl CommentThread {
    pub fn new(
        version_id: Option<String>,
        anchor_type: AnchorType,
        scene_id: Option<String>,
        chapter_id: Option<String>,
        from_pos: Option<i32>,
        to_pos: Option<i32>,
        selected_text: Option<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            version_id,
            anchor_type,
            scene_id,
            chapter_id,
            from_pos,
            to_pos,
            selected_text,
            status: ThreadStatus::Open,
            created_at: Local::now(),
            resolved_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentThreadWithMessages {
    pub thread: CommentThread,
    pub messages: Vec<CommentMessage>,
}

// ==================== StyleDNA 模型 (深度风格系统) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleDNA {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub dna_json: String,
    pub is_builtin: bool,
    pub is_user_created: bool,
    pub created_at: DateTime<Local>,
}

// ==================== StyleDNA 六维向量快照 (W3-B7) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSnapshot {
    pub id: String,
    pub story_id: String,
    pub chapter_number: Option<i32>,
    pub scene_number: Option<i32>,
    pub sentence_length: f64,
    pub dialogue_ratio: f64,
    pub metaphor_density: f64,
    pub inner_monologue_ratio: f64,
    pub emotion_density: f64,
    pub rhythm_score: f64,
    pub computed_at: DateTime<Local>,
}

// ==================== 风格混合配置模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryStyleConfig {
    pub id: String,
    pub story_id: String,
    pub name: String,
    pub blend_json: String, // JSON serialized Vec<BlendComponent>
    pub is_active: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}
