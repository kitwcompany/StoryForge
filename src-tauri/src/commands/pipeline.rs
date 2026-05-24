//! Pipeline commands

use tauri::AppHandle;
use crate::get_pool;

#[tauri::command(rename_all = "snake_case")]
pub async fn auto_create_missing_contracts(
    story_id: String,
    chapter_number: i32,
    scene_id: Option<String>,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let builder = crate::story_system::auto_contract::AutoContractBuilder::new(pool, app_handle);
    let (created_master, created_chapter, created_outline) = builder.auto_fill(&story_id, chapter_number, scene_id.as_deref()).await?;
    Ok(serde_json::json!({
        "created_master_setting": created_master,
        "created_chapter_contract": created_chapter,
        "created_outline": created_outline,
        "message": if created_master || created_chapter || created_outline {
            "补齐完成"
        } else {
            "所有合同和大纲已存在，无需补齐"
        }
    }))
}

