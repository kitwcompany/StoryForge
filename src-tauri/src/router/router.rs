//! Unified model router — v0.11.0
//!
//! 根据任务类型、复杂度、成本/速度偏好，从 UnifiedModelRegistry
//! 中选择最优模型。 不再硬编码默认模型；无可用模型时返回明确错误。

use serde::{Deserialize, Serialize};

use super::registry::UnifiedModelRegistry;
use crate::{
    config::settings::{LlmProfile, ModelCapability, ModelKind, QualityTier, SpeedTier},
    error::AppError,
};

/// 任务类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// 创意写作：需要高质量
    CreativeWriting,
    /// 编辑/改写：需要精确
    Editing,
    /// 分析/推理：需要上下文与逻辑
    Analysis,
    /// 对话/角色声音
    Dialogue,
    /// 摘要：可用更快/更便宜模型
    Summarization,
    /// 头脑风暴：可用更快/更便宜模型
    Brainstorming,
    /// 校对：需要准确性
    Proofreading,
    /// 世界观构建：需要创意与一致性
    WorldBuilding,
    /// 多模态理解（图文）
    Vision,
    /// 图像生成
    ImageGeneration,
}

/// 任务复杂度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Complexity {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// 优先级（成本、速度等维度）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    #[default]
    Low,
    Medium,
    High,
}

/// 路由约束：调用方可以叠加额外过滤条件
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum RoutingConstraint {
    /// 最低质量等级
    MinQuality(QualityTier),
    /// 最低上下文长度
    MinContext(u32),
    /// 必须具备的能力
    Requires(ModelCapability),
    /// 仅使用本地模型
    LocalOnly,
    /// 仅使用平台模型
    PlatformOnly,
}

/// 路由请求
#[derive(Debug, Clone)]
pub struct RoutingRequest {
    pub task: TaskType,
    pub complexity: Complexity,
    pub budget_priority: Priority,
    pub speed_priority: Priority,
    pub estimated_input_tokens: u32,
    pub constraints: Vec<RoutingConstraint>,
}

impl Default for RoutingRequest {
    fn default() -> Self {
        Self {
            task: TaskType::CreativeWriting,
            complexity: Complexity::Medium,
            budget_priority: Priority::Low,
            speed_priority: Priority::Low,
            estimated_input_tokens: 0,
            constraints: Vec::new(),
        }
    }
}

/// 排序后的候选模型
#[derive(Debug, Clone, Serialize)]
pub struct RankedCandidate {
    pub model_id: String,
    pub model_name: String,
    pub score: f64,
    pub reason: String,
}

/// 路由决策结果
#[derive(Debug, Clone, Serialize)]
pub struct RoutingDecision {
    pub model_id: String,
    pub model_name: String,
    pub reason: String,
    pub estimated_cost: f64,
    pub estimated_time_ms: u64,
    /// v0.14.0: 有序候选链，第一个是 primary
    pub candidates: Vec<RankedCandidate>,
}

/// 统一模型路由器
#[derive(Debug, Clone, Default)]
pub struct UnifiedModelRouter {
    registry: UnifiedModelRegistry,
}

impl UnifiedModelRouter {
    pub fn new(registry: UnifiedModelRegistry) -> Self {
        Self { registry }
    }

    /// 执行路由，选择最合适的生成模型
    pub fn route(&self, request: &RoutingRequest) -> Result<RoutingDecision, AppError> {
        let suitable_models: Vec<&LlmProfile> = self
            .registry
            .generative_models()
            .into_iter()
            .filter(|m| self.is_suitable(m, request))
            .collect();

        if suitable_models.is_empty() {
            return Err(AppError::Internal {
                message: format!(
                    "没有满足任务 {:?} 与约束 {:?} 的可用模型，请在模型管理中启用并配置模型",
                    request.task, request.constraints
                ),
            });
        }

        let mut scored: Vec<(f64, &LlmProfile)> = suitable_models
            .into_iter()
            .map(|m| (score_model(m, request), m))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let (best_score, model) = scored.first().copied().unwrap();
        let candidates: Vec<RankedCandidate> = scored
            .into_iter()
            .take(3)
            .map(|(score, m)| RankedCandidate {
                model_id: m.id.clone(),
                model_name: m.name.clone(),
                score,
                reason: format!(
                    "任务 {:?} 复杂度 {:?} 得分 {:.1}",
                    request.task, request.complexity, score
                ),
            })
            .collect();

        Ok(RoutingDecision {
            model_id: model.id.clone(),
            model_name: model.name.clone(),
            reason: format!(
                "基于 {:?} 任务、{:?} 复杂度、成本优先级 {:?}、速度优先级 {:?} 选择，得分 {:.1}",
                request.task,
                request.complexity,
                request.budget_priority,
                request.speed_priority,
                best_score
            ),
            estimated_cost: model.cost_per_1k_output.unwrap_or(0.0),
            estimated_time_ms: estimated_time_ms(model.speed_tier),
            candidates,
        })
    }

    /// 为 embedding 任务选择可用模型；若指定了 model_id 则优先匹配
    pub fn route_embedding(&self, preferred_id: Option<&str>) -> Result<RoutingDecision, AppError> {
        let candidates = self.registry.embedding_models();
        if candidates.is_empty() {
            return Err(AppError::Internal {
                message: "没有可用的 Embedding 模型，请在模型管理中配置".to_string(),
            });
        }

        let model = preferred_id
            .and_then(|id| candidates.iter().find(|m| m.id == id).copied())
            .or_else(|| candidates.iter().find(|m| m.is_default).copied())
            .or_else(|| candidates.first().copied())
            .unwrap();

        Ok(RoutingDecision {
            model_id: model.id.clone(),
            model_name: model.name.clone(),
            reason: "选择 Embedding 模型".to_string(),
            estimated_cost: 0.0,
            estimated_time_ms: 2000,
            candidates: vec![RankedCandidate {
                model_id: model.id.clone(),
                model_name: model.name.clone(),
                score: 0.0,
                reason: "embedding 唯一候选".to_string(),
            }],
        })
    }

    /// 获取指定模型
    pub fn get_model(&self, model_id: &str) -> Option<&crate::config::settings::LlmProfile> {
        self.registry.get(model_id).and_then(|m| match m {
            super::registry::UnifiedModel::Generative(p) => Some(p),
            _ => None,
        })
    }

    /// 获取全部生成模型
    pub fn all_generative_models(&self) -> Vec<&LlmProfile> {
        self.registry.generative_models()
    }

    pub fn is_suitable(&self, model: &LlmProfile, request: &RoutingRequest) -> bool {
        // 任务类型硬性过滤
        match request.task {
            TaskType::Vision => {
                if model.kind != ModelKind::Multimodal
                    && !model.capabilities.contains(&ModelCapability::Vision)
                {
                    return false;
                }
            }
            TaskType::ImageGeneration => {
                if model.kind != ModelKind::Image {
                    return false;
                }
            }
            _ => {}
        }

        // 上下文长度
        if request.estimated_input_tokens > 0
            && model.max_context_length < request.estimated_input_tokens
        {
            return false;
        }

        // 用户约束
        for constraint in &request.constraints {
            match constraint {
                RoutingConstraint::MinQuality(q) => {
                    if quality_rank(model.quality_tier) < quality_rank(*q) {
                        return false;
                    }
                }
                RoutingConstraint::MinContext(ctx) => {
                    if model.max_context_length < *ctx {
                        return false;
                    }
                }
                RoutingConstraint::Requires(cap) => {
                    if !model.capabilities.contains(cap) {
                        return false;
                    }
                }
                RoutingConstraint::LocalOnly => {
                    if model.model_source != crate::config::settings::ModelSource::Local {
                        return false;
                    }
                }
                RoutingConstraint::PlatformOnly => {
                    if model.model_source == crate::config::settings::ModelSource::Local {
                        return false;
                    }
                }
            }
        }

        true
    }
}

pub fn score_model(model: &LlmProfile, request: &RoutingRequest) -> f64 {
    let mut score = 0.0;

    // 1. 任务质量匹配
    score += match (request.task, model.quality_tier) {
        (TaskType::CreativeWriting, QualityTier::Ultra) => 100.0,
        (TaskType::CreativeWriting, QualityTier::High) => 80.0,
        (TaskType::Editing, QualityTier::High) => 90.0,
        (TaskType::Editing, QualityTier::Ultra) => 100.0,
        (TaskType::Analysis, QualityTier::Ultra) => 90.0,
        (TaskType::Analysis, QualityTier::High) => 80.0,
        (TaskType::Proofreading, QualityTier::High) => 85.0,
        (TaskType::Proofreading, QualityTier::Ultra) => 95.0,
        (TaskType::Summarization, _) => 50.0,
        (TaskType::Brainstorming, _) => 50.0,
        _ => 70.0,
    };

    // 2. 复杂度加成：复杂任务倾向高质量模型
    score += match request.complexity {
        Complexity::Low => 0.0,
        Complexity::Medium => 10.0,
        Complexity::High => 25.0,
        Complexity::Critical => 50.0,
    };

    // 3. 上下文余量：上下文越大越安全
    let ctx_margin = if request.estimated_input_tokens > 0 {
        (model.max_context_length as f64 / request.estimated_input_tokens.max(1) as f64).min(3.0)
            * 10.0
    } else {
        10.0
    };
    score += ctx_margin;

    // 4. 成本惩罚
    let output_cost = model.cost_per_1k_output.unwrap_or(0.0);
    score -= match request.budget_priority {
        Priority::Low => 0.0,
        Priority::Medium => output_cost * 10.0,
        Priority::High => output_cost * 30.0,
    };

    // 5. 速度惩罚
    score -= match request.speed_priority {
        Priority::Low => 0.0,
        Priority::Medium => speed_penalty(model.speed_tier, 1.0),
        Priority::High => speed_penalty(model.speed_tier, 3.0),
    };

    // 6. 多模态/图像任务鼓励对应 kind
    score += match (request.task, model.kind) {
        (TaskType::Vision, ModelKind::Multimodal) => 20.0,
        (TaskType::ImageGeneration, ModelKind::Image) => 20.0,
        _ => 0.0,
    };

    score
}

fn quality_rank(tier: QualityTier) -> u8 {
    match tier {
        QualityTier::Low => 1,
        QualityTier::Medium => 2,
        QualityTier::High => 3,
        QualityTier::Ultra => 4,
    }
}

fn speed_penalty(tier: SpeedTier, multiplier: f64) -> f64 {
    match tier {
        SpeedTier::Fast => 0.0,
        SpeedTier::Normal => 10.0 * multiplier,
        SpeedTier::Slow => 30.0 * multiplier,
        SpeedTier::VerySlow => 80.0 * multiplier,
    }
}

fn estimated_time_ms(tier: SpeedTier) -> u64 {
    match tier {
        SpeedTier::Fast => 1000,
        SpeedTier::Normal => 5000,
        SpeedTier::Slow => 15000,
        SpeedTier::VerySlow => 45000,
    }
}
