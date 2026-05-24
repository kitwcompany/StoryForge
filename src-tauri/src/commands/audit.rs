//! Audit commands

use crate::get_pool;

// ==================== Story Structure Audit Commands ====================

#[tauri::command(rename_all = "snake_case")]
pub async fn audit_story(
    story_id: String,
    app_handle: tauri::AppHandle,
) -> Result<crate::narrative::audit::StoryAnalysisReport, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let llm_service = crate::llm::LlmService::new(app_handle);
    let auditor = crate::narrative::audit::StoryStructureAuditor::new(pool, llm_service);
    auditor.analyze(&story_id).await.map_err(|e| e.to_string())
}

