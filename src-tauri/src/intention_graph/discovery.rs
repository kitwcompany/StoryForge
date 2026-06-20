//! 分层发现机制
//!
//! Server-level: PPR 图传播发现相关意图/资产
//! Tool-level: 描述匹配 + 意图匹配 + 图信号 + 协同过滤 融合评分

use std::collections::HashMap;

use crate::error::AppError;

use super::graph::IntentionGraphRepository;
use super::models::*;
use super::scorer::GraphScorer;

/// 分层发现引擎
pub struct LayeredDiscovery {
    scorer: GraphScorer,
}

impl LayeredDiscovery {
    pub fn new(scorer: GraphScorer) -> Self {
        Self { scorer }
    }

    /// Server-level 发现：基于 PPR 图传播
    /// 从根意图出发，在异构图上传播，发现相关意图和资产
    pub fn discover_server_level(
        &self,
        root_intention: &IntentionNode,
        graph_repo: &IntentionGraphRepository,
        max_results: usize,
    ) -> Result<ServerLevelResult, AppError> {
        // 1. 获取根意图直接关联的资产
        let direct_edges = graph_repo.get_intention_edges(
            &root_intention.id,
            Some(IntentionAssetEdgeType::TriggeredBy),
        )?;

        let mut asset_scores: HashMap<String, f64> = HashMap::new();

        // 2. 直接关联的权重最高
        for edge in &direct_edges {
            let score = edge.weight * edge.cooccurrence_count as f64;
            asset_scores.insert(edge.asset_id.clone(), score);
        }

        // 3. PPR 传播：通过 asset-asset 边扩展
        for edge in &direct_edges {
            let asset_edges = graph_repo.get_asset_edges(
                &edge.asset_id,
                Some(AssetAssetEdgeType::ToolCooccur),
            )?;
            for ae in &asset_edges {
                let propagated_score = edge.weight * ae.weight * 0.5; // 衰减因子
                asset_scores
                    .entry(ae.target_asset_id.clone())
                    .and_modify(|s| *s += propagated_score)
                    .or_insert(propagated_score);
            }
        }

        // 4. 排序取 Top-K
        let mut scored: Vec<(String, f64)> = asset_scores.into_iter().collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(max_results);

        let mut assets = Vec::new();
        for (asset_id, score) in scored {
            if let Some(asset) = graph_repo.get_asset(&asset_id)? {
                assets.push((asset, score));
            }
        }

        Ok(ServerLevelResult {
            root_intention: root_intention.clone(),
            discovered_assets: assets,
            propagation_depth: 2,
        })
    }

    /// Tool-level 发现：多信号融合评分
    /// score = 0.3 * desc_match + 0.4 * intent_match + 0.2 * ppr_graph + 0.1 * collab_bonus
    pub fn discover_tool_level(
        &self,
        intention: &IntentionNode,
        candidates: &[AssetNode],
        _graph_repo: &IntentionGraphRepository,
    ) -> Vec<AssetDiscoveryResult> {
        let mut results = Vec::new();

        for candidate in candidates {
            // 1. 描述匹配（语义相似度）
            let desc_score = if let (Some(a_emb), Some(i_emb)) =
                (candidate.embedding.as_ref(), intention.embedding.as_ref())
            {
                cosine_similarity(a_emb, i_emb)
            } else {
                // 回退到文本匹配
                Self::text_similarity(&candidate.description, &intention.description)
            };

            // 2. 意图匹配（名称/能力 ID 匹配）
            let intent_score = Self::intent_match_score(intention, candidate);

            // 3. PPR 图分数（由上层传入）
            let ppr_score = 0.5; // 占位，实际由 discover_server_level 计算

            // 4. 协同过滤分数（基于共现频率）
            let collab_score = (candidate.frequency as f64).min(10.0) / 10.0;

            // 加权融合
            let total_score = 0.3 * desc_score + 0.4 * intent_score + 0.2 * ppr_score + 0.1 * collab_score;

            results.push(AssetDiscoveryResult {
                asset: candidate.clone(),
                score: total_score,
                semantic_score: desc_score,
                intent_score,
                ppr_score,
                collab_score,
                reason: format!(
                    "desc_match={:.2}, intent_match={:.2}, ppr={:.2}, collab={:.2}",
                    desc_score, intent_score, ppr_score, collab_score
                ),
            });
        }

        // 按总分排序
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// 完整分层发现：Server-level + Tool-level
    pub fn discover(
        &self,
        root_intention: &IntentionNode,
        graph_repo: &IntentionGraphRepository,
        max_results: usize,
    ) -> Result<Vec<AssetDiscoveryResult>, AppError> {
        // Server-level: PPR 传播发现候选资产
        let server_result = self.discover_server_level(root_intention, graph_repo, max_results * 3)?;

        let candidates: Vec<AssetNode> = server_result
            .discovered_assets
            .iter()
            .map(|(a, _)| a.clone())
            .collect();

        // Tool-level: 多信号融合重排序
        let mut results = self.discover_tool_level(root_intention, &candidates, graph_repo);
        results.truncate(max_results);

        Ok(results)
    }

    // ------------------------------------------------------------------
    // 辅助函数
    // ------------------------------------------------------------------

    fn text_similarity(a: &str, b: &str) -> f64 {
        let a_words: HashMap<String, i32> = a
            .to_lowercase()
            .split_whitespace()
            .map(|w| (w.to_string(), 1))
            .collect();
        let b_words: HashMap<String, i32> = b
            .to_lowercase()
            .split_whitespace()
            .map(|w| (w.to_string(), 1))
            .collect();

        let mut intersection = 0;
        let mut union = a_words.len() + b_words.len();

        for (word, _) in &a_words {
            if b_words.contains_key(word) {
                intersection += 1;
                union -= 1;
            }
        }

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    fn intent_match_score(intention: &IntentionNode, asset: &AssetNode) -> f64 {
        let mut score = 0.0;

        // 动词匹配
        if asset.description.to_lowercase().contains(&intention.verb)
            || asset.name.to_lowercase().contains(&intention.verb)
        {
            score += 0.5;
        }

        // 宾语匹配
        if asset.description.to_lowercase().contains(&intention.object)
            || asset.name.to_lowercase().contains(&intention.object)
        {
            score += 0.5;
        }

        score
    }
}

/// Server-level 发现结果
#[derive(Debug, Clone)]
pub struct ServerLevelResult {
    pub root_intention: IntentionNode,
    pub discovered_assets: Vec<(AssetNode, f64)>,
    pub propagation_depth: usize,
}
