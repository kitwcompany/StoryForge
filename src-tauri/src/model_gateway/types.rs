//! Model Gateway — 通用类型定义
//!
//! v0.14.0: 为模型网关提供任务、健康快照、路由决策等共享类型。

use serde::{Deserialize, Serialize};

use crate::router::{Complexity, Priority, RoutingDecision, TaskType};

/// 模型健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// 尚未探测
    Unknown,
    /// 探测成功且近期成功率达标
    Healthy,
    /// 可用但指标下降（如 TTFB 过高、成功率偏低）
    Degraded,
    /// 探测失败或近期成功率过低
    Unhealthy,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// 单个模型的健康快照
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelHealthSnapshot {
    pub model_id: String,
    pub model_name: String,
    pub status: HealthStatus,
    /// 首 token 时间（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttfb_ms: Option<u64>,
    /// 生成速度（tokens / second）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tps: Option<f64>,
    /// 最近 24 小时成功率（0.0–1.0）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_rate_24h: Option<f64>,
    /// 平均延迟（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_latency_ms: Option<u64>,
    /// 最近一次错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// 最近一次探测时间（ISO 8601）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checked_at: Option<String>,
    /// 是否被用户启用
    pub enabled: bool,
    /// 是否被当前任务选为主模型
    #[serde(default)]
    pub is_primary: bool,
    /// 若为 fallback 目标，显示将 fallback 到该模型
    #[serde(default)]
    pub is_fallback: bool,
}

/// 轻量探测结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProbeResult {
    pub success: bool,
    /// 首 token 时间（毫秒）
    pub ttft_ms: u64,
    /// 总输出 token 数
    pub total_tokens: i32,
    /// 总耗时（毫秒）
    pub duration_ms: u64,
    /// tokens per second（不包含 TTFT）
    pub tps: f64,
    pub error: Option<String>,
}

/// 网关生成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    /// 原始 prompt
    pub prompt: String,
    /// 发起调用的 Agent 或模块标识
    pub agent_id: String,
    /// 任务类型
    pub task: TaskType,
    /// 可选的复杂度覆盖；None 时由 dispatcher 动态评估
    pub complexity: Option<Complexity>,
    /// 成本优先级
    pub budget_priority: Priority,
    /// 速度优先级
    pub speed_priority: Priority,
    /// 估计输入 token 数
    pub estimated_input_tokens: u32,
    /// 期望最大输出 token 数
    pub max_tokens: Option<i32>,
    /// 温度
    pub temperature: Option<f32>,
    /// 是否流式输出
    pub stream: bool,
    /// 请求 ID（用于日志和取消）
    pub request_id: String,
    /// 上下文标签（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_label: Option<String>,
    /// 超时覆盖（秒，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds_override: Option<u64>,
    /// 最大重试覆盖（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries_override: Option<u32>,
}

/// 网关层对路由决策的扩展：RoutingDecision 已包含候选链
pub type GatewayRoutingDecision = RoutingDecision;

/// 网关整体状态（供前端展示）
#[derive(Debug, Clone, Serialize)]
pub struct GatewayStatus {
    /// 最近一次全量探测时间
    pub last_probe_at: Option<String>,
    /// 当前被选中的主模型
    pub primary_model_id: Option<String>,
    /// 所有启用模型的健康快照
    pub models: Vec<ModelHealthSnapshot>,
    /// 是否正在探测中
    pub is_probing: bool,
}
