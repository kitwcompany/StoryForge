//! Story commands

use crate::db::{StoryRepository, CreateStoryRequest};
use tauri::AppHandle;
use crate::get_pool;

#[tauri::command(rename_all = "snake_case")]
pub fn list_stories() -> Result<Vec<crate::db::Story>, String> {
    StoryRepository::new(get_pool().ok_or("DB not initialized")?).get_all().map_err(|e| crate::error::AppError::from(e).to_string())
}


#[tauri::command(rename_all = "snake_case")]
pub fn create_story(title: String, description: Option<String>, genre: Option<String>, app: AppHandle, automation_service: tauri::State<crate::automation::service::AutomationService>) -> Result<crate::db::Story, String> {
    let story = StoryRepository::new(get_pool().ok_or("DB not initialized")?).create(CreateStoryRequest { title, description, genre, style_dna_id: None }).map_err(|e| crate::error::AppError::from(e).to_string())?;
    let _ = crate::state_sync::StateSync::emit_story_created(&app, &story.id, &story.title);
    let automation_service_clone = automation_service.inner().clone();
    let story_id_clone = story.id.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = automation_service_clone.trigger_event(crate::automation::triggers::TriggerEvent::StoryCreated {
            story_id: story_id_clone,
        }).await {
            log::warn!("[create_story] Failed to trigger story created automation: {}", e);
        }
    });
    Ok(story)
}


#[tauri::command(rename_all = "snake_case")]
pub fn update_story(
    id: String,
    title: Option<String>,
    description: Option<String>,
    genre: Option<String>,
    tone: Option<String>,
    pacing: Option<String>,
    style_dna_id: Option<String>,
    methodology_id: Option<String>,
    methodology_step: Option<i32>,
    app: AppHandle,
) -> Result<(), String> {
    let req = crate::db::UpdateStoryRequest { title: title.clone(), description: description.clone(), genre, tone, pacing, style_dna_id, methodology_id, methodology_step };
    StoryRepository::new(get_pool().ok_or("DB not initialized")?).update(&id, &req).map_err(|e| crate::error::AppError::from(e).to_string())?;
    let _ = crate::state_sync::StateSync::emit_story_updated(&app, &id, title.as_deref());
    Ok(())
}


#[tauri::command(rename_all = "snake_case")]
pub fn delete_story(id: String, app: AppHandle) -> Result<(), String> {
    StoryRepository::new(get_pool().ok_or("DB not initialized")?).delete(&id).map_err(|e| crate::error::AppError::from(e).to_string())?;
    let _ = crate::state_sync::StateSync::emit_story_deleted(&app, &id);
    Ok(())
}

