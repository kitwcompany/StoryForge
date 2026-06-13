//! Router commands exposed to the frontend — v0.11.0

use tauri::{command, AppHandle, Manager};

use super::{
    Complexity, Priority, RoutingConstraint, RoutingDecision, RoutingRequest, TaskType,
    UnifiedModelRegistry, UnifiedModelRouter,
};
use crate::{config::settings::AppConfig, error::AppError};

/// Frontend payload for route simulation
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SimulateRoutePayload {
    pub task: TaskType,
    #[serde(default)]
    pub complexity: Complexity,
    #[serde(default)]
    pub budget_priority: Priority,
    #[serde(default)]
    pub speed_priority: Priority,
    #[serde(default)]
    pub estimated_input_tokens: u32,
    #[serde(default)]
    pub constraints: Vec<RoutingConstraint>,
}

/// Simulate which model the router would select for a given task.
#[command]
pub fn simulate_route(
    payload: SimulateRoutePayload,
    app_handle: AppHandle,
) -> Result<RoutingDecision, AppError> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;

    let config = AppConfig::load(&app_dir).map_err(AppError::from)?;
    let registry = UnifiedModelRegistry::from_app_config(&config);
    let router = UnifiedModelRouter::new(registry);

    let request = RoutingRequest {
        task: payload.task,
        complexity: payload.complexity,
        budget_priority: payload.budget_priority,
        speed_priority: payload.speed_priority,
        estimated_input_tokens: payload.estimated_input_tokens,
        constraints: payload.constraints,
    };

    router.route(&request)
}
