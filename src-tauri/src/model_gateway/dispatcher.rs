//! Model Gateway — 任务分配与复杂度评估
//!
//! v0.14.0: 将调用方请求转换为 `RoutingRequest`，支持动态复杂度评估。

use crate::{
    config::settings::LlmProfile,
    memory::tokenizer::count_tokens,
    router::{Complexity, RoutingConstraint, RoutingRequest, TaskType},
};

use super::types::GatewayRequest;

/// 任务分类器
#[derive(Debug, Clone, Default)]
pub struct TaskClassifier;

impl TaskClassifier {
    pub fn new() -> Self {
        Self
    }

    /// 根据请求动态评估复杂度
    pub fn classify(&self, request: &GatewayRequest) -> RoutingRequest {
        let complexity = request.complexity.unwrap_or_else(|| {
            Self::estimate_complexity(request.prompt.as_str(), request.task, &request.agent_id)
        });

        let mut constraints = Vec::new();

        // 视觉/图像任务必须带对应能力
        match request.task {
            TaskType::Vision => constraints.push(RoutingConstraint::Requires(
                crate::config::settings::ModelCapability::Vision,
            )),
            TaskType::ImageGeneration => {
                // ImageGeneration 在 registry 中通过 kind 过滤，这里不需要额外约束
            }
            _ => {}
        }

        // 长上下文要求
        if request.estimated_input_tokens > 0 {
            constraints.push(RoutingConstraint::MinContext(
                request.estimated_input_tokens,
            ));
        }

        RoutingRequest {
            task: request.task,
            complexity,
            budget_priority: request.budget_priority,
            speed_priority: request.speed_priority,
            estimated_input_tokens: request.estimated_input_tokens,
            constraints,
        }
    }

    /// 估算任务复杂度
    pub fn estimate_complexity(prompt: &str, task: TaskType, agent_id: &str) -> Complexity {
        // 1. 基于 agent_id 的默认值
        let base = match agent_id {
            "commentator" | "memory_compressor" | "knowledge_distiller" => Complexity::Medium,
            "inspector" | "style_mimic" => Complexity::Medium,
            "writer" | "outline_planner" | "plot_analyzer" => Complexity::High,
            _ => match task {
                TaskType::Summarization | TaskType::Brainstorming => Complexity::Medium,
                TaskType::CreativeWriting | TaskType::WorldBuilding => Complexity::High,
                TaskType::Analysis | TaskType::Editing => Complexity::High,
                _ => Complexity::Medium,
            },
        };

        let mut level = base;

        // 2. 输入 token 数 > 4k 升一级
        let token_count = count_tokens(prompt, "gpt-4");
        if token_count > 4000 {
            level = bump_complexity(level);
        }

        // 3. 用户明确关键词升一级
        let lower = prompt.to_lowercase();
        let complex_signals = [
            "深度分析",
            "复杂",
            "多线",
            "伏笔",
            " intricate",
            "complex",
            "deep analysis",
            "multi-thread",
        ];
        if complex_signals.iter().any(|s| lower.contains(s)) {
            level = bump_complexity(level);
        }

        level
    }
}

fn bump_complexity(c: Complexity) -> Complexity {
    match c {
        Complexity::Low => Complexity::Medium,
        Complexity::Medium => Complexity::High,
        Complexity::High | Complexity::Critical => Complexity::Critical,
    }
}

/// 根据模型配置评估其是否适合给定任务（额外考虑 reasoning 等能力偏好）
pub fn evaluate_model_fit(model: &LlmProfile, request: &RoutingRequest) -> f64 {
    let mut score = 0.0;

    // 复杂写作任务偏好 reasoning
    if request.task == TaskType::CreativeWriting && request.complexity >= Complexity::High {
        if model
            .capabilities
            .contains(&crate::config::settings::ModelCapability::Reasoning)
        {
            score += 25.0;
        } else {
            score -= 10.0; // 不强制，但降权
        }
    }

    // 摘要/头脑风暴偏好 fast
    if matches!(request.task, TaskType::Summarization | TaskType::Brainstorming) {
        if model
            .capabilities
            .contains(&crate::config::settings::ModelCapability::Fast)
        {
            score += 15.0;
        }
    }

    score
}
