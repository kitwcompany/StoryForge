//! 算力档案持久化存储
//!
//! v0.15.0: 网关升级为智能调度器，本模块提供 `model_capability_profile` 表的
//! CRUD 操作，支持跨应用启动保留模型的流式基准实测数据（TTFB、TPS、成功率）。

use rusqlite::params;

use crate::{
    db::DbPool,
    model_gateway::types::{CapabilityProfile, HealthStatus},
};

pub struct CapabilityStore {
    pool: DbPool,
}

impl CapabilityStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 加载所有已存档的算力档案
    pub fn load_all(&self) -> Result<Vec<CapabilityProfile>, rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let mut stmt = conn.prepare(
            "SELECT model_id, short_ttfb_ms_p50, short_ttfb_ms_p95, long_ttfb_ms_p50,
                    long_ttfb_ms_p95, sustained_tps, short_output_tps, success_rate_24h,
                    last_full_benchmark_at, last_health_probe_at, benchmark_sample_count,
                    status, status_reason, capability_score, speed_score, quality_score
             FROM model_capability_profile",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CapabilityProfile {
                model_id: row.get(0)?,
                short_ttfb_ms_p50: row.get::<_, Option<i64>>(1)?.map(|v| v as u64),
                short_ttfb_ms_p95: row.get::<_, Option<i64>>(2)?.map(|v| v as u64),
                long_ttfb_ms_p50: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                long_ttfb_ms_p95: row.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                sustained_tps: row.get(5)?,
                short_output_tps: row.get(6)?,
                success_rate_24h: row.get(7)?,
                last_full_benchmark_at: row.get(8)?,
                last_health_probe_at: row.get(9)?,
                benchmark_sample_count: row.get(10)?,
                status: deserialize_status(row.get::<_, String>(11)?.as_str()),
                status_reason: row.get(12)?,
                capability_score: row.get(13)?,
                speed_score: row.get(14)?,
                quality_score: row.get(15)?,
            })
        })?;
        rows.collect()
    }

    /// 插入或更新一条算力档案
    pub fn upsert(&self, profile: &CapabilityProfile) -> Result<(), rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let status = serialize_status(profile.status);
        conn.execute(
            "INSERT INTO model_capability_profile
                (model_id, short_ttfb_ms_p50, short_ttfb_ms_p95, long_ttfb_ms_p50,
                 long_ttfb_ms_p95, sustained_tps, short_output_tps, success_rate_24h,
                 last_full_benchmark_at, last_health_probe_at, benchmark_sample_count,
                 status, status_reason, capability_score, speed_score, quality_score,
                 updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     strftime('%s', 'now'))
             ON CONFLICT(model_id) DO UPDATE SET
                short_ttfb_ms_p50 = excluded.short_ttfb_ms_p50,
                short_ttfb_ms_p95 = excluded.short_ttfb_ms_p95,
                long_ttfb_ms_p50 = excluded.long_ttfb_ms_p50,
                long_ttfb_ms_p95 = excluded.long_ttfb_ms_p95,
                sustained_tps = excluded.sustained_tps,
                short_output_tps = excluded.short_output_tps,
                success_rate_24h = excluded.success_rate_24h,
                last_full_benchmark_at = excluded.last_full_benchmark_at,
                last_health_probe_at = excluded.last_health_probe_at,
                benchmark_sample_count = excluded.benchmark_sample_count,
                status = excluded.status,
                status_reason = excluded.status_reason,
                capability_score = excluded.capability_score,
                speed_score = excluded.speed_score,
                quality_score = excluded.quality_score,
                updated_at = strftime('%s', 'now')",
            params![
                profile.model_id,
                profile.short_ttfb_ms_p50.map(|v| v as i64),
                profile.short_ttfb_ms_p95.map(|v| v as i64),
                profile.long_ttfb_ms_p50.map(|v| v as i64),
                profile.long_ttfb_ms_p95.map(|v| v as i64),
                profile.sustained_tps,
                profile.short_output_tps,
                profile.success_rate_24h,
                profile.last_full_benchmark_at,
                profile.last_health_probe_at,
                profile.benchmark_sample_count,
                status,
                profile.status_reason,
                profile.capability_score,
                profile.speed_score,
                profile.quality_score,
            ],
        )?;
        Ok(())
    }

    /// 标记模型为 unhealthy
    pub fn mark_unhealthy(&self, model_id: &str, reason: &str) -> Result<(), rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        conn.execute(
            "INSERT INTO model_capability_profile (model_id, status, status_reason, updated_at)
             VALUES (?1, 'unhealthy', ?2, strftime('%s', 'now'))
             ON CONFLICT(model_id) DO UPDATE SET
                status = 'unhealthy', status_reason = ?2,
                updated_at = strftime('%s', 'now')",
            params![model_id, reason],
        )?;
        Ok(())
    }
}

fn serialize_status(s: HealthStatus) -> &'static str {
    match s {
        HealthStatus::Unknown => "unknown",
        HealthStatus::Healthy => "healthy",
        HealthStatus::Degraded => "degraded",
        HealthStatus::Unhealthy => "unhealthy",
    }
}

fn deserialize_status(s: &str) -> HealthStatus {
    match s {
        "healthy" => HealthStatus::Healthy,
        "degraded" => HealthStatus::Degraded,
        "unhealthy" => HealthStatus::Unhealthy,
        _ => HealthStatus::Unknown,
    }
}
