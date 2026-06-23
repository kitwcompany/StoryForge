//! Model Gateway — Tauri 命令
//!
//! v0.14.0: 向前端暴露网关状态查询、重新探测、模拟路由等命令。

use tauri::{command, AppHandle, State};

use super::{
    executor::GatewayExecutor,
    types::{GatewayRoutingDecision, GatewayStatus, ModelHealthSnapshot, ProbeResult},
};
use crate::error::AppError;

/// 获取网关整体状态（供前端底部状态栏展示）
#[command]
pub async fn get_gateway_status(
    _app_handle: AppHandle,
    executor: State<'_, GatewayExecutor>,
) -> Result<GatewayStatus, AppError> {
    let health_registry = executor.health_registry();
    let health = {
        let guard = health_registry.lock().map_err(|_| AppError::Internal {
            message: "健康注册表锁定失败".to_string(),
        })?;
        guard.all()
    };

    let models: Vec<_> = {
        let guard = executor.registry.lock().map_err(|_| AppError::Internal {
            message: "网关注册表锁定失败".to_string(),
        })?;
        guard.models_with_health(&health)
    }
    .into_iter()
    .filter(|m| {
        m.status == super::types::HealthStatus::Healthy
            || m.status == super::types::HealthStatus::Degraded
    })
    .collect();

    Ok(GatewayStatus {
        last_probe_at: models
            .iter()
            .filter_map(|m| m.last_checked_at.clone())
            .max(),
        primary_model_id: None, // TODO: 从当前活跃任务获取
        models,
        is_probing: false,
    })
}

/// 重新探测单个模型
#[command]
pub async fn refresh_model_health(
    model_id: String,
    executor: State<'_, GatewayExecutor>,
) -> Result<ModelHealthSnapshot, AppError> {
    let _ = executor.probe_model(&model_id).await?;
    let health_registry = executor.health_registry();
    let guard = health_registry.lock().map_err(|_| AppError::Internal {
        message: "健康注册表锁定失败".to_string(),
    })?;
    guard
        .get(&model_id)
        .cloned()
        .ok_or_else(|| AppError::Internal {
            message: format!("模型 {} 健康记录不存在", model_id),
        })
}

/// 模拟路由决策
#[command]
pub async fn simulate_gateway_route(
    request: super::types::GatewayRequest,
    executor: State<'_, GatewayExecutor>,
) -> Result<GatewayRoutingDecision, AppError> {
    executor.select_candidates(&request, None)
}

/// 获取探测结果
#[command]
pub async fn probe_model_gateway(
    model_id: String,
    executor: State<'_, GatewayExecutor>,
) -> Result<ProbeResult, AppError> {
    executor.probe_model(&model_id).await
}
