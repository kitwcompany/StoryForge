use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== 知识图谱模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub story_id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub attributes: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub first_seen: DateTime<Local>,
    pub last_updated: DateTime<Local>,

    // 置信度评分 (0-1)
    pub confidence_score: Option<f32>,
    // 访问计数（用于遗忘曲线）
    pub access_count: i32,
    // 最后访问时间
    pub last_accessed: Option<DateTime<Local>>,

    // 归档状态
    pub is_archived: bool,
    pub archived_at: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Character,
    Location,
    Item,
    Organization,
    Concept,
    Event,
    PlotDevice,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EntityType::Character => "Character",
            EntityType::Location => "Location",
            EntityType::Item => "Item",
            EntityType::Organization => "Organization",
            EntityType::Concept => "Concept",
            EntityType::Event => "Event",
            EntityType::PlotDevice => "PlotDevice",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for EntityType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Character" | "角色" => Ok(EntityType::Character),
            "Location" | "地点" => Ok(EntityType::Location),
            "Item" | "物品" => Ok(EntityType::Item),
            "Organization" | "组织" => Ok(EntityType::Organization),
            "Concept" | "概念" => Ok(EntityType::Concept),
            "Event" | "事件" => Ok(EntityType::Event),
            "PlotDevice" => Ok(EntityType::PlotDevice),
            _ => Err(format!("Unknown entity type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub story_id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: RelationType,
    pub strength: f32,
    pub evidence: Vec<String>,
    pub first_seen: DateTime<Local>,

    // 置信度评分
    pub confidence_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    // 人际关系
    Friend,
    Enemy,
    Family,
    Lover,
    Mentor,
    Rival,
    Ally,

    // 物品关系
    LocatedAt,
    BelongsTo,
    Uses,
    Owns,
    Created,
    Destroyed,

    // 组织关系
    PartOf,
    Leads,
    MemberOf,
    FounderOf,

    // 因果关系
    Causes,
    Enables,
    Prevents,
    ResultsIn,

    // 语义关系
    SimilarTo,
    OppositeOf,
    RelatedTo,
    EvolvesInto,

    // 动态关系
    Supersedes,  // 取代
    Contradicts, // 矛盾

    // 创世引擎关系
    ParticipatesIn, // 角色参与场景
    SetUpIn,        // 伏笔在场景中埋设
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RelationType::Friend => "Friend",
            RelationType::Enemy => "Enemy",
            RelationType::Family => "Family",
            RelationType::Lover => "Lover",
            RelationType::Mentor => "Mentor",
            RelationType::Rival => "Rival",
            RelationType::Ally => "Ally",
            RelationType::LocatedAt => "LocatedAt",
            RelationType::BelongsTo => "BelongsTo",
            RelationType::Uses => "Uses",
            RelationType::Owns => "Owns",
            RelationType::Created => "Created",
            RelationType::Destroyed => "Destroyed",
            RelationType::PartOf => "PartOf",
            RelationType::Leads => "Leads",
            RelationType::MemberOf => "MemberOf",
            RelationType::FounderOf => "FounderOf",
            RelationType::Causes => "Causes",
            RelationType::Enables => "Enables",
            RelationType::Prevents => "Prevents",
            RelationType::ResultsIn => "ResultsIn",
            RelationType::SimilarTo => "SimilarTo",
            RelationType::OppositeOf => "OppositeOf",
            RelationType::RelatedTo => "RelatedTo",
            RelationType::EvolvesInto => "EvolvesInto",
            RelationType::Supersedes => "Supersedes",
            RelationType::Contradicts => "Contradicts",
            RelationType::ParticipatesIn => "ParticipatesIn",
            RelationType::SetUpIn => "SetUpIn",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for RelationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Friend" => Ok(RelationType::Friend),
            "Enemy" => Ok(RelationType::Enemy),
            "Family" => Ok(RelationType::Family),
            "Lover" => Ok(RelationType::Lover),
            "Mentor" => Ok(RelationType::Mentor),
            "Rival" => Ok(RelationType::Rival),
            "Ally" => Ok(RelationType::Ally),
            "LocatedAt" => Ok(RelationType::LocatedAt),
            "BelongsTo" => Ok(RelationType::BelongsTo),
            "Uses" => Ok(RelationType::Uses),
            "Owns" => Ok(RelationType::Owns),
            "Created" => Ok(RelationType::Created),
            "Destroyed" => Ok(RelationType::Destroyed),
            "PartOf" => Ok(RelationType::PartOf),
            "Leads" => Ok(RelationType::Leads),
            "MemberOf" => Ok(RelationType::MemberOf),
            "FounderOf" => Ok(RelationType::FounderOf),
            "Causes" => Ok(RelationType::Causes),
            "Enables" => Ok(RelationType::Enables),
            "Prevents" => Ok(RelationType::Prevents),
            "ResultsIn" => Ok(RelationType::ResultsIn),
            "SimilarTo" => Ok(RelationType::SimilarTo),
            "OppositeOf" => Ok(RelationType::OppositeOf),
            "RelatedTo" => Ok(RelationType::RelatedTo),
            "EvolvesInto" => Ok(RelationType::EvolvesInto),
            "Supersedes" => Ok(RelationType::Supersedes),
            "Contradicts" => Ok(RelationType::Contradicts),
            "ParticipatesIn" => Ok(RelationType::ParticipatesIn),
            "SetUpIn" => Ok(RelationType::SetUpIn),
            _ => Err(format!("Unknown relation type: {}", s)),
        }
    }
}
