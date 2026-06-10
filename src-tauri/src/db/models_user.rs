use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== 用户反馈与偏好模型 (自适应学习) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedbackLog {
    pub id: String,
    pub story_id: String,
    pub scene_id: Option<String>,
    pub chapter_id: Option<String>,
    pub feedback_type: FeedbackType,
    pub agent_type: Option<String>,
    pub original_ai_text: String,
    pub final_text: String,
    pub ai_score: Option<f32>,
    pub user_satisfaction: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    Accept, // 用户直接接受 AI 建议
    Reject, // 用户拒绝 AI 建议
    Modify, // 用户修改后接受
}

impl std::fmt::Display for FeedbackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackType::Accept => write!(f, "accept"),
            FeedbackType::Reject => write!(f, "reject"),
            FeedbackType::Modify => write!(f, "modify"),
        }
    }
}

impl std::str::FromStr for FeedbackType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "accept" => Ok(FeedbackType::Accept),
            "reject" => Ok(FeedbackType::Reject),
            "modify" => Ok(FeedbackType::Modify),
            _ => Err(format!("Unknown feedback type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    pub id: String,
    pub story_id: String,
    pub preference_type: PreferenceType,
    pub preference_key: String,
    pub preference_value: String,
    pub confidence: f32,
    pub evidence_count: i32,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceType {
    Style,     // 风格偏好
    Content,   // 内容偏好
    Structure, // 结构偏好
    Dialogue,  // 对话偏好
    Pacing,    // 节奏偏好
}

impl std::fmt::Display for PreferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreferenceType::Style => write!(f, "style"),
            PreferenceType::Content => write!(f, "content"),
            PreferenceType::Structure => write!(f, "structure"),
            PreferenceType::Dialogue => write!(f, "dialogue"),
            PreferenceType::Pacing => write!(f, "pacing"),
        }
    }
}

impl std::str::FromStr for PreferenceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "style" => Ok(PreferenceType::Style),
            "content" => Ok(PreferenceType::Content),
            "structure" => Ok(PreferenceType::Structure),
            "dialogue" => Ok(PreferenceType::Dialogue),
            "pacing" => Ok(PreferenceType::Pacing),
            _ => Err(format!("Unknown preference type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub tone: Option<String>,
    pub pacing: Option<String>,
    pub style_dna_id: Option<String>,
    pub methodology_id: Option<String>,
    pub methodology_step: Option<i32>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub story_id: String,
    pub name: String,
    pub background: Option<String>,
    pub personality: Option<String>,
    pub goals: Option<String>,
    pub appearance: Option<String>,
    pub gender: Option<String>,
    pub age: Option<i32>,
    pub dynamic_traits: Vec<DynamicTrait>,
    // --- 动态状态字段 ---
    pub cs_location: Option<String>,
    pub cs_power_level: Option<String>,
    pub cs_physical_state: Option<String>,
    pub cs_mental_state: Option<String>,
    pub cs_key_items: Option<String>,
    pub cs_recent_events: Option<String>,
    pub cs_updated_at_chapter: Option<i32>,
    pub cs_json: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub story_id: String,
    pub chapter_number: i32,
    pub title: Option<String>,
    pub outline: Option<String>,
    pub content: Option<String>,
    pub word_count: Option<i32>,
    pub model_used: Option<String>,
    pub cost: Option<f64>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// ==================== Auth Models ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_local_user: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthAccount {
    pub id: String,
    pub user_id: String,
    pub provider: String,
    pub provider_account_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Local>>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Local>,
    pub created_at: DateTime<Local>,
}
