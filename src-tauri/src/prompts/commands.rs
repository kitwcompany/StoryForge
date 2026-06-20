//! PromptRegistry IPC 命令

use tauri::State;

use crate::{db::DbPool, error::AppError};

use super::registry;

/// 列出所有提示词条目
#[tauri::command(rename_all = "snake_case")]
pub fn list_prompt_entries(pool: State<'_, DbPool>) -> Result<Vec<registry::PromptEntry>, AppError> {
    registry::list_prompt_entries(&pool)
}

/// 保存提示词覆盖
#[tauri::command(rename_all = "snake_case")]
pub fn save_prompt_override(
    pool: State<'_, DbPool>,
    prompt_id: String,
    content: String,
) -> Result<(), AppError> {
    registry::save_override(&pool, &prompt_id, &content)
}

/// 重置提示词为默认
#[tauri::command(rename_all = "snake_case")]
pub fn reset_prompt_override(pool: State<'_, DbPool>, prompt_id: String) -> Result<(), AppError> {
    registry::reset_override(&pool, &prompt_id)
}

/// 批量重置所有提示词覆盖
#[tauri::command(rename_all = "snake_case")]
pub fn reset_all_prompt_overrides(pool: State<'_, DbPool>) -> Result<usize, AppError> {
    registry::reset_all_overrides(&pool)
}

/// 解析提示词内容（用于调试/预览）
#[tauri::command(rename_all = "snake_case")]
pub fn resolve_prompt_content(pool: State<'_, DbPool>, prompt_id: String) -> Result<String, AppError> {
    registry::resolve_prompt(&pool, &prompt_id)
}
