#![allow(dead_code)]
//! 中性搜索类型
//!
//! 被 memory / narrative 等模块共享，避免 narrative 直接依赖 memory 模块。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 混合搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub id: String,
    pub content: String,
    pub bm25_score: f32,
    pub vector_score: f32,
    pub hybrid_score: f32,
    pub source_type: SourceType,
    pub metadata: HashMap<String, String>,
}

/// 搜索结果来源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    Scene,
    Entity,
    Memory,
    Note,
}
