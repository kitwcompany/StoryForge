//! Model Gateway — 任务分类与分配
//!
//! v0.15.0: 升级为智能任务分类器，按 prompt 长度 + agent 类型将任务分入
//! LightTool / BalancedWork / HeavyCreation 三类，驱动后续模型路由决策。

use super::types::{GatewayRequest, TaskClass};
use crate::router::{Complexity, RoutingRequest, TaskType};

/// 智能任务分类器（v0.15.0 替代旧版 TaskClassifier）
///
/// 决策依据：
/// 1. Agent 类型（Writer/Inspector 等）
/// 2. TaskType + 估计 input/output token 数
/// 3. 升级惩罚：输入 >4000 token 或输出 >1500 token → 升为 HeavyCreation
#[derive(Debug, Clone, Default)]
pub struct TaskClassifier;

impl TaskClassifier {
    pub fn new() -> Self {
        Self
    }

    /// 根据 GatewayRequest 判定任务复杂度类别（v0.15.0 核心）
    pub fn classify_task(req: &GatewayRequest) -> TaskClass {
        let base = Self::classify_by_type(&req.task, &req.agent_id);
        Self::upgrade_by_size(base, req.estimated_input_tokens, req.max_tokens)
    }

    fn classify_by_type(task: &TaskType, agent_id: &str) -> TaskClass {
        match (task, agent_id) {
            // 轻量工具
            (TaskType::Analysis, id)
                if id == "input_hint"
                    || id == "intent_detection"
                    || id == "model_gateway_probe" =>
            {
                TaskClass::LightTool
            }
            (TaskType::Summarization, _) => TaskClass::LightTool,
            (TaskType::WorldBuilding, _) => TaskClass::LightTool,
            // 重型创作
            (TaskType::CreativeWriting, _) => TaskClass::HeavyCreation,
            (TaskType::Editing, _) => TaskClass::HeavyCreation,
            (_, id) if id == "writer" || id == "inspector" || id == "style_mimic" => {
                TaskClass::HeavyCreation
            }
            _ => TaskClass::BalancedWork,
        }
    }

    fn upgrade_by_size(base: TaskClass, input_tokens: u32, max_output: Option<i32>) -> TaskClass {
        let out = max_output.unwrap_or(512) as u32;
        match base {
            TaskClass::LightTool if input_tokens > 4000 || out > 1500 => TaskClass::BalancedWork,
            TaskClass::BalancedWork if input_tokens > 4000 || out > 1500 => {
                TaskClass::HeavyCreation
            }
            other => other,
        }
    }

    /// 兼容旧版：根据请求动态评估复杂度（转为 RoutingRequest）
    pub fn classify(&self, request: &GatewayRequest) -> RoutingRequest {
        let complexity = request
            .complexity
            .unwrap_or_else(|| match Self::classify_task(request) {
                TaskClass::LightTool => Complexity::Low,
                TaskClass::BalancedWork => Complexity::Medium,
                TaskClass::HeavyCreation => Complexity::High,
            });

        RoutingRequest {
            task: request.task.clone(),
            complexity,
            budget_priority: request.budget_priority.clone(),
            speed_priority: request.speed_priority.clone(),
            estimated_input_tokens: request.estimated_input_tokens,
            constraints: Vec::new(),
        }
    }
}
