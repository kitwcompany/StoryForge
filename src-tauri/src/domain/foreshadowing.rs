#![allow(dead_code)]
//! 中性伏笔类型
//!
//! 被 creative_engine / narrative 等模块共享，避免 narrative 直接依赖
//! creative_engine。

/// 伏笔状态
#[derive(Debug, Clone)]
pub enum ForeshadowingStatus {
    /// 已设置，未回收
    Setup,
    /// 已回收
    Payoff,
    /// 已放弃
    Abandoned,
}

impl std::fmt::Display for ForeshadowingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForeshadowingStatus::Setup => write!(f, "setup"),
            ForeshadowingStatus::Payoff => write!(f, "payoff"),
            ForeshadowingStatus::Abandoned => write!(f, "abandoned"),
        }
    }
}

/// 伏笔记录
#[derive(Debug, Clone)]
pub struct ForeshadowingRecord {
    pub id: String,
    pub story_id: String,
    pub content: String,
    pub setup_scene_id: Option<String>,
    pub payoff_scene_id: Option<String>,
    // LitSeg: 叙事事件关联（从 narrative_threads.foreshadow 合并）
    pub setup_event_id: Option<String>,
    pub payoff_event_id: Option<String>,
    pub risk_signals_score: Option<f32>,
    pub status: ForeshadowingStatus,
    pub importance: i32, // 1-10
    pub created_at: String,
    pub resolved_at: Option<String>,
}

/// 伏笔查询端口，供 narrative 等模块在不依赖 creative_engine 的情况下读取伏笔。
pub trait ForeshadowingProvider: Send + Sync {
    fn get_all(
        &self,
        story_id: &str,
    ) -> Result<Vec<ForeshadowingRecord>, Box<dyn std::error::Error + Send + Sync>>;
}
