//! 图传播评分引擎
//!
//! 实现 Personalized PageRank (PPR) 在异构意图-资产图上的传播。
//! 同时支持语义相似度评分和协同过滤信号。

use std::collections::{HashMap, VecDeque};

/// PPR 配置
#[derive(Debug, Clone)]
pub struct PprConfig {
    /// 随机游走重启概率（1 - alpha）
    pub alpha: f64,
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 收敛阈值
    pub convergence_threshold: f64,
    /// 每步最大邻居数（采样）
    pub max_neighbors_per_step: usize,
}

impl Default for PprConfig {
    fn default() -> Self {
        Self {
            alpha: 0.85,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            max_neighbors_per_step: 50,
        }
    }
}

/// 图传播评分器
pub struct GraphScorer {
    config: PprConfig,
}

impl GraphScorer {
    pub fn new(config: PprConfig) -> Self {
        Self { config }
    }

    /// 在异构图上执行 PPR 传播
    /// 从种子节点出发，计算图中所有节点的访问概率
    pub fn ppr_propagate(
        &self,
        seed_nodes: &[String],
        edges: &HashMap<String, Vec<(String, f64)>>, // node_id -> [(neighbor_id, weight)]
    ) -> HashMap<String, f64> {
        let mut scores: HashMap<String, f64> = HashMap::new();
        let mut new_scores: HashMap<String, f64> = HashMap::new();

        // 初始化：种子节点均分概率质量
        let seed_prob = 1.0 / seed_nodes.len() as f64;
        for seed in seed_nodes {
            scores.insert(seed.clone(), seed_prob);
        }

        let alpha = self.config.alpha;
        let restart_prob = 1.0 - alpha;

        for _ in 0..self.config.max_iterations {
            new_scores.clear();

            // 对每个当前有概率质量的节点，传播到邻居
            for (node_id, score) in &scores {
                if *score < 1e-10 {
                    continue;
                }

                // 重启概率质量回到种子节点
                let restart_mass = score * restart_prob;
                for seed in seed_nodes {
                    new_scores
                        .entry(seed.clone())
                        .and_modify(|s| *s += restart_mass / seed_nodes.len() as f64)
                        .or_insert(restart_mass / seed_nodes.len() as f64);
                }

                // 传播概率到邻居
                let neighbors = edges.get(node_id).cloned().unwrap_or_default();
                let total_weight: f64 = neighbors.iter().map(|(_, w)| w).sum();

                if total_weight > 0.0 {
                    let propagation_mass = score * alpha;
                    for (neighbor_id, weight) in &neighbors {
                        let prob = propagation_mass * (weight / total_weight);
                        new_scores
                            .entry(neighbor_id.clone())
                            .and_modify(|s| *s += prob)
                            .or_insert(prob);
                    }
                }
            }

            // 检查收敛
            let max_diff: f64 = scores
                .keys()
                .chain(new_scores.keys())
                .map(|k| {
                    let old = scores.get(k).unwrap_or(&0.0);
                    let new = new_scores.get(k).unwrap_or(&0.0);
                    (old - new).abs()
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);

            scores = new_scores.clone();

            if max_diff < self.config.convergence_threshold {
                break;
            }
        }

        scores
    }

    /// 基于 BFS 的短路径评分（用于快速近似）
    pub fn bfs_relevance(
        &self,
        seed_nodes: &[String],
        edges: &HashMap<String, Vec<(String, f64)>>,
        max_depth: usize,
    ) -> HashMap<String, f64> {
        let mut scores: HashMap<String, f64> = HashMap::new();
        let mut visited: HashMap<String, usize> = HashMap::new(); // node -> depth
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();

        for seed in seed_nodes {
            scores.insert(seed.clone(), 1.0);
            visited.insert(seed.clone(), 0);
            queue.push_back((seed.clone(), 0));
        }

        while let Some((node_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            let neighbors = edges.get(&node_id).cloned().unwrap_or_default();
            for (neighbor_id, weight) in neighbors {
                if visited.contains_key(&neighbor_id) {
                    continue;
                }

                let decay = 0.5_f64.powi(depth as i32 + 1);
                let score = weight * decay;

                scores
                    .entry(neighbor_id.clone())
                    .and_modify(|s| *s = s.max(score))
                    .or_insert(score);

                visited.insert(neighbor_id.clone(), depth + 1);
                queue.push_back((neighbor_id, depth + 1));
            }
        }

        scores
    }

    /// 协同过滤评分：基于共现历史
    pub fn collaborative_filtering(
        &self,
        target_asset_id: &str,
        cooccurrence_counts: &HashMap<String, i32>, // asset_id -> count
    ) -> f64 {
        let count = cooccurrence_counts.get(target_asset_id).copied().unwrap_or(0);
        let max_count = cooccurrence_counts.values().copied().max().unwrap_or(1);

        if max_count == 0 {
            0.0
        } else {
            count as f64 / max_count as f64
        }
    }
}

impl Default for GraphScorer {
    fn default() -> Self {
        Self::new(PprConfig::default())
    }
}
