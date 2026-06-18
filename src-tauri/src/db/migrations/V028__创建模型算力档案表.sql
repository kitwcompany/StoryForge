-- v0.15.0: ModelGateway 算力档案持久化
-- 网关从"健康监控展示器"升级为"智能调度器"，本表存储每个模型的实测算力档案，
-- 跨应用启动持久化，避免重启后丢失基准数据。
--
-- 字段说明：
--   short_*：短任务基准（200 token 输入 / 150 token 输出，模拟摘要/意图识别）
--   long_*：长任务基准（600 token 输入 / 1200 token 输出，模拟 800 字续写）
--   sustained_tps：长输出持续 token/s（v0.14 旧 probe 中的魔法数被替换为真实值）
--   capability_score：综合调度得分 0-100，由 score_for_task 计算
--   status：枚举 healthy/degraded/unhealthy/unknown，由 ProbeEngine 维护
--   last_full_benchmark_at：上次完整基准时间戳（unix 秒）
CREATE TABLE IF NOT EXISTS model_capability_profile (
    model_id              TEXT PRIMARY KEY,
    short_ttfb_ms_p50     INTEGER,
    short_ttfb_ms_p95     INTEGER,
    long_ttfb_ms_p50      INTEGER,
    long_ttfb_ms_p95      INTEGER,
    sustained_tps         REAL,
    short_output_tps      REAL,
    success_rate_24h      REAL,
    last_full_benchmark_at INTEGER,
    last_health_probe_at  INTEGER,
    benchmark_sample_count INTEGER NOT NULL DEFAULT 0,
    status                TEXT NOT NULL DEFAULT 'unknown',
    status_reason         TEXT,
    capability_score      REAL,
    speed_score           REAL,
    quality_score         REAL,
    created_at            INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at            INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_capability_status ON model_capability_profile(status);
CREATE INDEX IF NOT EXISTS idx_capability_score ON model_capability_profile(capability_score);
