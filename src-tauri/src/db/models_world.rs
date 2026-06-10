use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== 世界观模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBuilding {
    pub id: String,
    pub story_id: String,
    pub concept: String,
    pub rules: Vec<WorldRule>,
    pub history: Option<String>,
    pub cultures: Vec<Culture>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: RuleType,
    pub importance: i32, // 1-10
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    Magic,      // 魔法规则
    Technology, // 科技规则
    Social,     // 社会规则
    Physical,   // 物理规则
    Biological, // 生物规则
    Historical, // 历史规则
    Cultural,   // 文化规则
    Custom,     // 自定义
}

impl std::fmt::Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RuleType::Magic => "魔法",
            RuleType::Technology => "科技",
            RuleType::Social => "社会",
            RuleType::Physical => "物理",
            RuleType::Biological => "生物",
            RuleType::Historical => "历史",
            RuleType::Cultural => "文化",
            RuleType::Custom => "自定义",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Culture {
    pub name: String,
    pub description: String,
    pub customs: Vec<String>,
    pub values: Vec<String>,
}

// ==================== 场景设置模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub id: String,
    pub story_id: String,
    pub name: String,
    pub description: Option<String>,
    pub location_type: LocationType,
    pub sensory_details: SensoryDetails,
    pub significance: Option<String>,
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationType {
    City,
    Building,
    Nature,
    Underground,
    Underwater,
    Space,
    Dream,
    Virtual,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SensoryDetails {
    pub visual: Vec<String>,
    pub auditory: Vec<String>,
    pub olfactory: Vec<String>,
    pub tactile: Vec<String>,
    pub gustatory: Vec<String>,
}

// ==================== 文字风格模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingStyle {
    pub id: String,
    pub story_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tone: Option<String>,
    pub pacing: Option<String>,
    pub vocabulary_level: Option<String>,
    pub sentence_structure: Option<String>,
    pub custom_rules: Vec<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}
