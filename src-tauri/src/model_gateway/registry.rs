//! Model Gateway — 网关视角的模型注册表
//!
//! v0.14.0: 包装 `router::UnifiedModelRegistry`，附加健康快照与能力索引。

use crate::{
    config::settings::{LlmProfile, ModelCapability},
    router::UnifiedModelRegistry,
};

use super::types::ModelHealthSnapshot;

/// 网关模型注册表
#[derive(Debug, Clone)]
pub struct GatewayRegistry {
    pub inner: UnifiedModelRegistry,
}

impl GatewayRegistry {
    pub fn new(inner: UnifiedModelRegistry) -> Self {
        Self { inner }
    }

    /// 所有启用的生成模型
    pub fn enabled_generative_models(&self) -> Vec<&LlmProfile> {
        self.inner
            .generative_models()
            .into_iter()
            .filter(|m| m.enabled)
            .collect()
    }

    /// 按能力过滤
    pub fn models_with_capability(&self, cap: ModelCapability) -> Vec<&LlmProfile> {
        self.enabled_generative_models()
            .into_iter()
            .filter(|m| m.capabilities.contains(&cap))
            .collect()
    }

    /// 获取指定模型
    pub fn get(&self, model_id: &str) -> Option<&LlmProfile> {
        self.inner.get(model_id).and_then(|m| match m {
            crate::router::UnifiedModel::Generative(p) => Some(p),
            _ => None,
        })
    }

    /// 合并健康快照，生成前端展示用的模型状态列表
    pub fn models_with_health(
        &self,
        health_snapshots: &[ModelHealthSnapshot],
    ) -> Vec<ModelHealthSnapshot> {
        let enabled: Vec<&LlmProfile> = self.enabled_generative_models();
        enabled
            .into_iter()
            .map(|m| {
                health_snapshots
                    .iter()
                    .find(|s| s.model_id == m.id)
                    .cloned()
                    .unwrap_or_else(|| ModelHealthSnapshot {
                        model_id: m.id.clone(),
                        model_name: m.name.clone(),
                        status: super::types::HealthStatus::Unknown,
                        ttfb_ms: None,
                        tps: None,
                        success_rate_24h: None,
                        avg_latency_ms: None,
                        last_error: None,
                        last_checked_at: None,
                        enabled: m.enabled,
                        is_primary: false,
                        is_fallback: false,
                    })
            })
            .collect()
    }
}

impl Default for GatewayRegistry {
    fn default() -> Self {
        Self::new(UnifiedModelRegistry::default())
    }
}
