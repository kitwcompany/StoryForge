//! Ai Op commands

use crate::db::ChapterRepository;
use tauri::{AppHandle, Emitter};
use crate::get_pool;

#[tauri::command(rename_all = "snake_case")]
pub async fn list_ai_operations(story_id: String) -> Result<Vec<crate::db::AiOperation>, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let repo = crate::db::AiOperationRepository::new(pool);
    repo.get_by_story(&story_id).map_err(|e| crate::error::AppError::from(e).to_string())
}


#[tauri::command(rename_all = "snake_case")]
pub async fn rollback_ai_operation(operation_id: String, app: AppHandle) -> Result<(), String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let op_repo = crate::db::AiOperationRepository::new(pool.clone());
    let chapter_repo = ChapterRepository::new(pool.clone());

    let operation = op_repo.get_by_id(&operation_id)
        .map_err(|e| crate::error::AppError::from(e).to_string())?
        .ok_or("Operation not found")?;

    // Only support rollback for chapter content operations that have previous_content
    let prev_content = operation.previous_content
        .ok_or("此操作不支持回滚")?;

    let chapter_id = operation.chapter_id
        .ok_or("此操作没有关联章节")?;

    // Restore previous content
    chapter_repo.update(&chapter_id, None, None, Some(prev_content), None)
        .map_err(|e| crate::error::AppError::from(e).to_string())?;

    // Mark operation as rolled back
    op_repo.update_status(&operation_id, "rolled_back")
        .map_err(|e| crate::error::AppError::from(e).to_string())?;

    // Emit sync event
    let _ = app.emit("sync-event", serde_json::json!({
        "event": "chapterUpdated",
        "chapter_id": chapter_id,
        "story_id": operation.story_id,
    }));

    Ok(())
}

