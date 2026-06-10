use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== 工作室配置模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioConfig {
    pub id: String,
    pub story_id: String,
    pub pen_name: Option<String>,
    pub llm_config: LlmStudioConfig,
    pub ui_config: UiStudioConfig,
    pub agent_bots: Vec<AgentBotConfig>,
    pub frontstage_theme: Option<String>,
    pub backstage_theme: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmStudioConfig {
    pub default_provider: String,
    pub default_model: String,
    pub generation_temperature: f32,
    pub max_tokens: i32,
    pub profiles: Vec<LlmProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProfile {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiStudioConfig {
    pub frontstage_font_size: i32,
    pub frontstage_font_family: String,
    pub frontstage_line_height: f32,
    pub frontstage_paper_color: String,
    pub frontstage_text_color: String,
    pub backstage_theme: String,
    pub backstage_accent_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBotConfig {
    pub id: String,
    pub name: String,
    pub agent_type: AgentBotType,
    pub enabled: bool,
    pub llm_profile_id: String,
    pub system_prompt: String,
    pub custom_settings: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentBotType {
    WorldBuilding, // 世界观助手
    Character,     // 人物助手
    WritingStyle,  // 文风助手
    Plot,          // 情节助手
    Scene,         // 场景助手
    Memory,        // 记忆助手
}
