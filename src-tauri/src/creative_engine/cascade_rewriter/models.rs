use serde::{Deserialize, Serialize};

/// 实体变更类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    AttributeModified,
    RelationModified,
    Created,
    Deleted,
}

/// 实体变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityChangeEvent {
    pub story_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub entity_name: String,
    pub change_type: ChangeType,
    pub before_json: String,
    pub after_json: String,
    pub changed_fields: Vec<String>,
    pub timestamp: String,
}

/// 实体引用索引（持久化到 SQLite）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMention {
    pub id: String,
    pub story_id: String,
    pub scene_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub start_pos: i32,
    pub end_pos: i32,
    pub mention_text: String,
    pub confidence: f64,
    pub created_at: String,
    pub updated_at: String,
}

/// 场景影响分析结果
#[derive(Debug, Clone)]
pub struct SceneImpact {
    pub scene_id: String,
    pub mention_count: usize,
    pub confidence_sum: f64,
}

impl SceneImpact {
    pub fn new(scene_id: &str) -> Self {
        Self {
            scene_id: scene_id.to_string(),
            mention_count: 0,
            confidence_sum: 0.0,
        }
    }

    pub fn score(&self) -> f64 {
        if self.mention_count == 0 {
            0.0
        } else {
            self.confidence_sum * (self.mention_count as f64).sqrt()
        }
    }
}

/// 用户改写决策
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserDecision {
    Pending,
    Accepted,
    Rejected,
}

/// 改写片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteSegment {
    pub scene_id: String,
    pub paragraph_index: i32,
    pub original_text: String,
    pub rewritten_text: String,
    pub change_reason: String,
    pub user_decision: UserDecision,
}

/// 改写状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RewriteStatus {
    Ok,
    NeedsReview,
    Failed,
}

/// 级联改写任务 Payload（序列化后存入 Task.payload）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeTaskPayload {
    pub story_id: String,
    pub change_events: Vec<EntityChangeEvent>,
}

/// 级联改写任务结果（序列化后存入 Task.result）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeTaskResult {
    pub status: RewriteStatus,
    pub segments: Vec<RewriteSegment>,
    pub warnings: Vec<String>,
}
