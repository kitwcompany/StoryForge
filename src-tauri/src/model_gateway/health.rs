//! Model Gateway — 模型健康探测与注册表
//!
//! v0.14.0: 负责维护每个启用模型的实时健康快照，包括 TTFB、TPS、成功率。

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::types::{HealthStatus, ModelHealthSnapshot, ProbeResult};
use crate::config::settings::LlmProfile;

/// 单个模型的历史健康记录
#[derive(Debug, Clone, Default)]
pub struct HealthRecord {
    pub snapshot: ModelHealthSnapshot,
    /// 最近若干次探测结果（用于计算平均 TTFB/TPS）
    pub recent_probes: Vec<ProbeResult>,
}

/// 健康注册表
#[derive(Debug, Clone, Default)]
pub struct HealthRegistry {
    records: HashMap<String, HealthRecord>,
}

impl HealthRegistry {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// 获取单个模型健康快照
    pub fn get(&self, model_id: &str) -> Option<&ModelHealthSnapshot> {
        self.records.get(model_id).map(|r| &r.snapshot)
    }

    /// 获取所有模型健康快照
    pub fn all(&self) -> Vec<ModelHealthSnapshot> {
        self.records.values().map(|r| r.snapshot.clone()).collect()
    }

    /// 更新或插入模型健康快照
    pub fn update(&mut self, snapshot: ModelHealthSnapshot) {
        let record = self.records.entry(snapshot.model_id.clone()).or_default();
        record.snapshot = snapshot;
    }

    /// 获取所有 healthy / degraded 模型的 ID（用于路由候选过滤）
    pub fn usable_model_ids(&self) -> Vec<String> {
        self.records
            .values()
            .filter(|r| {
                matches!(
                    r.snapshot.status,
                    HealthStatus::Healthy | HealthStatus::Degraded | HealthStatus::Unknown
                )
            })
            .map(|r| r.snapshot.model_id.clone())
            .collect()
    }

    /// 从历史 llm_calls 聚合最近 24 小时成功率
    pub fn aggregate_success_rate_from_history(&mut self, model_id: &str) -> Option<f64> {
        let pool = crate::get_pool()?;
        let conn = pool.get().ok()?;
        let since = chrono::Local::now() - chrono::Duration::hours(24);
        let since_str = since.to_rfc3339();

        let result = conn.query_row(
            "SELECT COUNT(*), SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) \
             FROM llm_calls WHERE model_id = ?1 AND created_at > ?2",
            rusqlite::params![model_id, since_str],
            |row| {
                let total: i64 = row.get(0)?;
                let success: i64 = row.get(1)?;
                Ok((total, success))
            },
        );

        match result {
            Ok((total, success)) if total > 0 => {
                let rate = success as f64 / total as f64;
                if let Some(record) = self.records.get_mut(model_id) {
                    record.snapshot.success_rate_24h = Some(rate);
                }
                Some(rate)
            }
            _ => None,
        }
    }

    /// 根据探测结果更新模型健康状态
    pub fn apply_probe_result(&mut self, profile: &LlmProfile, result: &ProbeResult) {
        let mut status = if result.success {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        // 如果历史成功率过低，标记为 degraded
        let success_rate = self
            .aggregate_success_rate_from_history(&profile.id)
            .or(Some(1.0));
        if result.success {
            if let Some(rate) = success_rate {
                if rate < 0.5 {
                    status = HealthStatus::Unhealthy;
                } else if rate < 0.8 {
                    status = HealthStatus::Degraded;
                }
            }
        }

        let record = self.records.entry(profile.id.clone()).or_default();
        record.recent_probes.push(result.clone());
        // 保留最近 20 次
        if record.recent_probes.len() > 20 {
            record.recent_probes.remove(0);
        }

        let ttfb_values: Vec<u64> = record
            .recent_probes
            .iter()
            .filter(|p| p.success)
            .map(|p| p.ttft_ms)
            .collect();
        let tps_values: Vec<f64> = record
            .recent_probes
            .iter()
            .filter(|p| p.success && p.tps > 0.0)
            .map(|p| p.tps)
            .collect();

        let avg_ttfb = if !ttfb_values.is_empty() {
            Some(ttfb_values.iter().sum::<u64>() / ttfb_values.len() as u64)
        } else {
            None
        };
        let avg_tps = if !tps_values.is_empty() {
            Some(tps_values.iter().sum::<f64>() / tps_values.len() as f64)
        } else {
            None
        };

        record.snapshot = ModelHealthSnapshot {
            model_id: profile.id.clone(),
            model_name: profile.name.clone(),
            status,
            ttfb_ms: avg_ttfb,
            tps: avg_tps,
            success_rate_24h: None, // TODO: 从 llm_calls 聚合
            avg_latency_ms: None,
            last_error: result.error.clone(),
            last_checked_at: Some(chrono::Local::now().to_rfc3339()),
            enabled: profile.enabled,
            is_primary: false,
            is_fallback: false,
        };
    }
}

/// 探测引擎
#[derive(Clone)]
pub struct ProbeEngine {
    registry: Arc<Mutex<HealthRegistry>>,
}

impl ProbeEngine {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(HealthRegistry::new())),
        }
    }

    pub fn registry(&self) -> Arc<Mutex<HealthRegistry>> {
        self.registry.clone()
    }

    /// 对单个模型执行轻量探测
    ///
    /// 实际调用由 executor 层注入，health 模块只负责记录与聚合。
    pub fn record_probe(&self, profile: &LlmProfile, result: &ProbeResult) {
        if let Ok(mut reg) = self.registry.lock() {
            reg.apply_probe_result(profile, result);
        }
    }
}

impl Default for ProbeEngine {
    fn default() -> Self {
        Self::new()
    }
}
