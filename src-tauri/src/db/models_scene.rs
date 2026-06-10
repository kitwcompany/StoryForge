use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== 通用类型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicTrait {
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub confidence: f32,
}

// ==================== 场景模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub story_id: String,
    pub sequence_number: i32,
    pub title: Option<String>,

    // 戏剧结构
    pub dramatic_goal: Option<String>,
    pub external_pressure: Option<String>,
    pub conflict_type: Option<ConflictType>,

    // 角色参与
    pub characters_present: Vec<String>,
    pub character_conflicts: Vec<CharacterConflict>,

    // 内容
    pub content: Option<String>,

    // 场景设置
    pub setting_location: Option<String>,
    pub setting_time: Option<String>,
    pub setting_atmosphere: Option<String>,

    // 关联
    pub previous_scene_id: Option<String>,
    pub next_scene_id: Option<String>,

    // 结构化大纲字段
    pub execution_stage: Option<String>, // planning | outline | drafting | review | final
    pub outline_content: Option<String>,
    pub draft_content: Option<String>,

    // 元数据
    pub model_used: Option<String>,
    pub cost: Option<f64>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,

    // 置信度评分 (0-1)
    pub confidence_score: Option<f32>,

    // 风格混合覆盖
    pub style_blend_override: Option<String>,

    // 关联伏笔ID列表
    pub foreshadowing_ids: Option<Vec<String>>,

    // 关联的章节ID
    pub chapter_id: Option<String>,

    // LitSeg: 叙事分析字段（从 narrative_events 表合并）
    pub narrative_intensity: Option<f32>,
    pub narrative_sentiment: Option<f32>,
    pub narrative_event_types: Option<String>,
    pub narrative_preceding_scene_id: Option<String>,
    pub narrative_following_scene_id: Option<String>,
    pub act_number: Option<i32>,
    pub position_in_act: Option<i32>,
}

/// 场景分隔节点
///
/// 在 1:N 架构下，Chapter 的 content 是多个 Scene 的聚合视图。
/// SceneDividerNode 标记 Scene 边界，支撑连续编辑表面上的 divider
/// 插入/删除/重排。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDividerNode {
    pub id: String,
    pub chapter_id: String,
    /// 在章节内的顺序位置（从 0 开始）
    pub position: i32,
    /// 该 divider 之后的 Scene ID
    pub scene_id: String,
    /// 可选标签（如 "Scene 2"、"转折"）
    pub label: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictType {
    ManVsMan,          // 人与人
    ManVsSelf,         // 人与自我
    ManVsSociety,      // 人与社会
    ManVsNature,       // 人与自然
    ManVsTechnology,   // 人与科技
    ManVsFate,         // 人与命运
    ManVsSupernatural, // 人与超自然
    ManVsTime,         // 人与时间
    ManVsMorality,     // 人与道德
    ManVsIdentity,     // 人与身份
    FactionVsFaction,  // 群体冲突
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ConflictType::ManVsMan => "人与人",
            ConflictType::ManVsSelf => "人与自我",
            ConflictType::ManVsSociety => "人与社会",
            ConflictType::ManVsNature => "人与自然",
            ConflictType::ManVsTechnology => "人与科技",
            ConflictType::ManVsFate => "人与命运",
            ConflictType::ManVsSupernatural => "人与超自然",
            ConflictType::ManVsTime => "人与时间",
            ConflictType::ManVsMorality => "人与道德",
            ConflictType::ManVsIdentity => "人与身份",
            ConflictType::FactionVsFaction => "群体冲突",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for ConflictType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "人与人" | "ManVsMan" => Ok(ConflictType::ManVsMan),
            "人与自我" | "ManVsSelf" => Ok(ConflictType::ManVsSelf),
            "人与社会" | "ManVsSociety" => Ok(ConflictType::ManVsSociety),
            "人与自然" | "ManVsNature" => Ok(ConflictType::ManVsNature),
            "人与科技" | "ManVsTechnology" => Ok(ConflictType::ManVsTechnology),
            "人与命运" | "ManVsFate" => Ok(ConflictType::ManVsFate),
            "人与超自然" | "ManVsSupernatural" => Ok(ConflictType::ManVsSupernatural),
            "人与时间" | "ManVsTime" => Ok(ConflictType::ManVsTime),
            "人与道德" | "ManVsMorality" => Ok(ConflictType::ManVsMorality),
            "人与身份" | "ManVsIdentity" => Ok(ConflictType::ManVsIdentity),
            "群体冲突" | "FactionVsFaction" => Ok(ConflictType::FactionVsFaction),
            _ => Err(format!("Unknown conflict type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterConflict {
    pub character_a_id: String,
    pub character_b_id: String,
    pub conflict_nature: String,
    pub stakes: String,
}

// ==================== 场景-角色关联模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneCharacter {
    pub id: String,
    pub scene_id: String,
    pub character_id: String,
    pub character_name: Option<String>, // 冗余字段，便于显示
    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneCharacterAction {
    pub id: String,
    pub scene_id: String,
    pub character_id: String,
    pub action_type: String, // dialogue, action, thought, etc.
    pub content: String,
    pub created_at: DateTime<Local>,
}

// ==================== 保留配置模型 (Phase 1.4) ====================

/// 艾宾浩斯遗忘曲线配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// 衰减率 λ (0.01 架构级, 0.05 默认, 0.1 瞬态)
    pub lambda: f64,
    /// 每次访问的强化奖励
    pub reinforcement_bonus: f64,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            lambda: 0.05,
            reinforcement_bonus: 0.1,
        }
    }
}

/// 保留优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPriority {
    Critical,  // 关键记忆（世界观、主要角色）
    High,      // 重要记忆（次要角色、关键事件）
    Medium,    // 普通记忆
    Low,       // 可压缩记忆
    Forgotten, // 已遗忘/可归档
}

// ==================== 场景版本模型 (新增) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneVersion {
    pub id: String,
    pub scene_id: String,
    pub version_number: i32,

    // 版本内容快照
    pub title: Option<String>,
    pub content: Option<String>,
    pub dramatic_goal: Option<String>,
    pub external_pressure: Option<String>,
    pub conflict_type: Option<ConflictType>,
    pub characters_present: Vec<String>,
    pub character_conflicts: Vec<CharacterConflict>,
    pub setting_location: Option<String>,
    pub setting_time: Option<String>,
    pub setting_atmosphere: Option<String>,

    // 版本元数据
    pub word_count: i32,
    pub change_summary: String,
    pub created_by: CreatorType,       // user/ai/system
    pub model_used: Option<String>,    // AI生成时使用的模型
    pub confidence_score: Option<f32>, // AI生成置信度

    // 版本链 (Supersession)
    pub previous_version_id: Option<String>,
    pub superseded_by: Option<String>, // 被哪个版本取代

    pub created_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreatorType {
    User,
    Ai,
    System,
}

impl std::fmt::Display for CreatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CreatorType::User => "user",
            CreatorType::Ai => "ai",
            CreatorType::System => "system",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for CreatorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(CreatorType::User),
            "ai" => Ok(CreatorType::Ai),
            "system" => Ok(CreatorType::System),
            _ => Err(format!("Unknown creator type: {}", s)),
        }
    }
}

// ==================== 场景批注模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAnnotation {
    pub id: String,
    pub scene_id: String,
    pub story_id: String,
    pub content: String,
    pub annotation_type: AnnotationType,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub resolved_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    Note,    // 普通笔记
    Todo,    // 待办事项
    Warning, // 警告/注意
    Idea,    // 灵感/想法
}

impl std::fmt::Display for AnnotationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AnnotationType::Note => "note",
            AnnotationType::Todo => "todo",
            AnnotationType::Warning => "warning",
            AnnotationType::Idea => "idea",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for AnnotationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "note" => Ok(AnnotationType::Note),
            "todo" => Ok(AnnotationType::Todo),
            "warning" => Ok(AnnotationType::Warning),
            "idea" => Ok(AnnotationType::Idea),
            _ => Err(format!("Unknown annotation type: {}", s)),
        }
    }
}

// ==================== 文本内联批注模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnnotation {
    pub id: String,
    pub story_id: String,
    pub scene_id: Option<String>,
    pub chapter_id: Option<String>,
    pub content: String,
    pub annotation_type: AnnotationType,
    pub from_pos: i32,
    pub to_pos: i32,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub resolved_at: Option<DateTime<Local>>,
}
