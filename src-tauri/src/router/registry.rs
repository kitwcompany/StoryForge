//! Unified model registry — v0.11.0
//!
//! 统一注册表聚合 AppConfig 中所有启用的生成模型与嵌入模型，
//! 为 router 提供单一事实来源，避免硬编码默认模型。

use std::collections::HashMap;

use serde::Serialize;

use crate::config::settings::{
    AppConfig, EmbeddingProfile, LlmProfile, ModelCapability, ModelKind, ModelSource, QualityTier,
    SpeedTier,
};

/// 统一模型视图：生成模型或嵌入模型均可通过该视图参与路由/展示
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UnifiedModel {
    Generative(LlmProfile),
    Embedding(EmbeddingProfile),
}

impl UnifiedModel {
    pub fn id(&self) -> &str {
        match self {
            UnifiedModel::Generative(p) => &p.id,
            UnifiedModel::Embedding(p) => &p.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            UnifiedModel::Generative(p) => &p.name,
            UnifiedModel::Embedding(p) => &p.name,
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            UnifiedModel::Generative(p) => p.enabled,
            UnifiedModel::Embedding(_) => true,
        }
    }

    pub fn model_source(&self) -> ModelSource {
        match self {
            UnifiedModel::Generative(p) => p.model_source,
            UnifiedModel::Embedding(_) => ModelSource::Platform,
        }
    }

    /// 是否支持指定能力（仅生成模型）
    pub fn supports_capability(&self, cap: ModelCapability) -> bool {
        match self {
            UnifiedModel::Generative(p) => p.capabilities.contains(&cap),
            UnifiedModel::Embedding(_) => false,
        }
    }

    /// 最大上下文长度（仅生成模型）
    pub fn max_context_length(&self) -> Option<u32> {
        match self {
            UnifiedModel::Generative(p) => Some(p.max_context_length),
            UnifiedModel::Embedding(_) => None,
        }
    }

    /// 质量等级（生成模型直接返回；嵌入模型默认 Medium）
    pub fn quality_tier(&self) -> QualityTier {
        match self {
            UnifiedModel::Generative(p) => p.quality_tier,
            UnifiedModel::Embedding(_) => QualityTier::Medium,
        }
    }

    /// 速度等级（生成模型直接返回；嵌入模型默认 Normal）
    pub fn speed_tier(&self) -> SpeedTier {
        match self {
            UnifiedModel::Generative(p) => p.speed_tier,
            UnifiedModel::Embedding(_) => SpeedTier::Normal,
        }
    }

    /// 每 1K 输出 token 成本（生成模型；嵌入模型输出成本为 0）
    pub fn cost_per_1k_output(&self) -> f64 {
        match self {
            UnifiedModel::Generative(p) => p.cost_per_1k_output.unwrap_or(0.0),
            UnifiedModel::Embedding(_) => 0.0,
        }
    }

    /// 每 1K 输入 token 成本
    pub fn cost_per_1k_input(&self) -> f64 {
        match self {
            UnifiedModel::Generative(p) => p.cost_per_1k_input.unwrap_or(0.0),
            UnifiedModel::Embedding(_) => 0.0,
        }
    }
}

/// 统一模型注册表
#[derive(Debug, Clone, Default)]
pub struct UnifiedModelRegistry {
    models: HashMap<String, UnifiedModel>,
}

impl UnifiedModelRegistry {
    /// 从 AppConfig 构建注册表，仅保留 enabled 模型
    pub fn from_app_config(config: &AppConfig) -> Self {
        let mut models = HashMap::new();

        for profile in config.llm_profiles.values() {
            if profile.enabled {
                models.insert(
                    profile.id.clone(),
                    UnifiedModel::Generative(profile.clone()),
                );
            }
        }

        // 嵌入模型默认启用，未来可补充 enabled 字段
        for profile in config.embedding_profiles.values() {
            models.insert(profile.id.clone(), UnifiedModel::Embedding(profile.clone()));
        }

        Self { models }
    }

    pub fn register(&mut self, model: UnifiedModel) {
        self.models.insert(model.id().to_string(), model);
    }

    pub fn get(&self, id: &str) -> Option<&UnifiedModel> {
        self.models.get(id)
    }

    pub fn all(&self) -> Vec<&UnifiedModel> {
        self.models.values().collect()
    }

    pub fn generative_models(&self) -> Vec<&LlmProfile> {
        self.models
            .values()
            .filter_map(|m| match m {
                UnifiedModel::Generative(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    pub fn embedding_models(&self) -> Vec<&EmbeddingProfile> {
        self.models
            .values()
            .filter_map(|m| match m {
                UnifiedModel::Embedding(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    pub fn chat_models(&self) -> Vec<&LlmProfile> {
        self.generative_models()
            .into_iter()
            .filter(|p| p.kind == ModelKind::Chat)
            .collect()
    }

    pub fn multimodal_models(&self) -> Vec<&LlmProfile> {
        self.generative_models()
            .into_iter()
            .filter(|p| p.kind == ModelKind::Multimodal)
            .collect()
    }

    pub fn image_models(&self) -> Vec<&LlmProfile> {
        self.generative_models()
            .into_iter()
            .filter(|p| p.kind == ModelKind::Image)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    pub fn len(&self) -> usize {
        self.models.len()
    }
}
