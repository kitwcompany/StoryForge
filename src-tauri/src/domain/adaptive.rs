//! Adaptive generation domain types.
//!
//! Pure data definitions shared across the creative engine and agents.
//! Behavior remains in `crate::creative_engine::adaptive`.

/// 生成策略
#[derive(Debug, Clone)]
pub struct GenerationStrategy {
    /// 温度（创造性 vs 确定性）
    pub temperature: f32,
    /// top-p（核采样）
    pub top_p: f32,
    /// 最大 token 数
    pub max_tokens: i32,
    /// 系统提示词权重增强
    pub prompt_weight_adjustments: Vec<PromptWeightAdjustment>,
    /// 风格偏好注入
    pub style_injections: Vec<String>,
    /// 内容约束
    pub content_constraints: Vec<String>,
}

impl Default for GenerationStrategy {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            top_p: 0.95,
            max_tokens: 2000,
            prompt_weight_adjustments: vec![],
            style_injections: vec![],
            content_constraints: vec![],
        }
    }
}

/// Prompt 权重调整
#[derive(Debug, Clone)]
pub struct PromptWeightAdjustment {
    pub target: String,    // 调整目标（如"对话""描写"）
    pub direction: String, // increase / decrease / maintain
    pub strength: f32,     // 0.0-1.0
    pub reason: String,
}
