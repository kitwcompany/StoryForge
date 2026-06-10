use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== 故事摘要模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySummary {
    pub id: String,
    pub story_id: String,
    pub summary_type: String,
    pub content: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// ==================== 故事大纲模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryOutline {
    pub id: String,
    pub story_id: String,
    pub content: String,
    pub structure_json: Option<String>,
    pub act_count: i32,
    pub total_scenes_estimate: Option<i32>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,

    // LitSeg: 分析后的实际幕结构（从 narrative_structure 表合并）
    pub analyzed_structure_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryOutlineAct {
    pub act_number: i32,
    pub title: String,
    pub summary: String,
    pub key_plot_points: Vec<String>,
    pub estimated_scenes: i32,
}

// ==================== 角色关系模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRelationship {
    pub id: String,
    pub story_id: String,
    pub source_character_id: String,
    pub target_character_id: String,
    pub target_character_name: Option<String>, // 填充时查询
    pub relationship_type: String,
    pub description: Option<String>,
    pub dynamic: Option<String>,
    pub created_at: DateTime<Local>,
}
