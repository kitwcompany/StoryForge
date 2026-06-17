//! Model Gateway — 健康探测调度器
//!
//! v0.14.0: 负责在应用启动时执行全量探测，并按计划对 healthy/degraded
//! 模型进行轻量 ping。

use std::time::Duration;

use tauri::AppHandle;
use tokio::time::interval;

use super::executor::GatewayExecutor;

/// 启动后台健康探测任务
///
/// - 启动时立即对所有启用模型执行一次探测
/// - 每 5 分钟对 healthy 模型 ping 一次
/// - 每 1 分钟对 degraded/unhealthy 模型重试一次
pub fn spawn_health_probe_scheduler(app_handle: AppHandle, executor: GatewayExecutor) {
    tauri::async_runtime::spawn(async move {
        // 启动时全量探测
        run_full_probe(&executor).await;

        let mut healthy_interval = interval(Duration::from_secs(300));
        let mut retry_interval = interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                _ = healthy_interval.tick() => {
                    run_healthy_probe(&executor).await;
                }
                _ = retry_interval.tick() => {
                    run_retry_probe(&executor).await;
                }
            }
        }
    });
}

async fn run_full_probe(executor: &GatewayExecutor) {
    let models: Vec<String> = executor
        .registry
        .enabled_generative_models()
        .into_iter()
        .map(|m| m.id.clone())
        .collect();

    log::info!("[GatewayScheduler] 启动时全量探测 {} 个模型", models.len());
    for model_id in models {
        if let Err(e) = executor.probe_model(&model_id).await {
            log::warn!("[GatewayScheduler] 探测 {} 失败: {}", model_id, e);
        }
    }
}

async fn run_healthy_probe(executor: &GatewayExecutor) {
    let registry = executor.health_registry();
    let health = match registry.lock() {
        Ok(g) => g.all(),
        Err(_) => return,
    };

    for snapshot in health {
        if snapshot.status == super::types::HealthStatus::Healthy {
            if let Err(e) = executor.probe_model(&snapshot.model_id).await {
                log::warn!(
                    "[GatewayScheduler] healthy 模型 {} ping 失败: {}",
                    snapshot.model_id,
                    e
                );
            }
        }
    }
}

async fn run_retry_probe(executor: &GatewayExecutor) {
    let registry = executor.health_registry();
    let health = match registry.lock() {
        Ok(g) => g.all(),
        Err(_) => return,
    };

    for snapshot in health {
        if matches!(
            snapshot.status,
            super::types::HealthStatus::Degraded | super::types::HealthStatus::Unhealthy
        ) {
            if let Err(e) = executor.probe_model(&snapshot.model_id).await {
                log::warn!(
                    "[GatewayScheduler] 重试模型 {} 失败: {}",
                    snapshot.model_id,
                    e
                );
            }
        }
    }
}
