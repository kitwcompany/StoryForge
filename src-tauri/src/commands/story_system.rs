//! Story System commands

use crate::error::AppError;
use crate::get_pool;

// ==================== Story System Commands ====================

#[tauri::command(rename_all = "snake_case")]
pub fn create_master_setting(
    story_id: String,
    genre: String,
    core_tone: String,
    pacing_strategy: String,
    anti_patterns: Vec<String>,
    world_rules: Vec<String>,
) -> Result<crate::db::StoryContract, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let engine = crate::story_system::StorySystemEngine::new(pool);
    engine.create_master_setting(
        &story_id, &genre, &core_tone, &pacing_strategy, &anti_patterns, &world_rules
    )
}


#[tauri::command(rename_all = "snake_case")]
pub fn create_chapter_contract(
    story_id: String,
    chapter_number: i32,
    goal: String,
    must_cover_nodes: Vec<String>,
    forbidden_zones: Vec<String>,
    time_anchor: Option<String>,
    chapter_span: Option<String>,
) -> Result<crate::db::StoryContract, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let engine = crate::story_system::StorySystemEngine::new(pool);
    engine.create_chapter_contract(
        &story_id, chapter_number, &goal, &must_cover_nodes, &forbidden_zones,
        time_anchor.as_deref(), chapter_span.as_deref()
    )
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_contract_tree(story_id: String) -> Result<crate::story_system::ContractTree, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let engine = crate::story_system::StorySystemEngine::new(pool);
    engine.get_contract_tree(&story_id)
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_runtime_contract(
    story_id: String,
    chapter_number: i32,
) -> Result<crate::story_system::RuntimeContract, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let engine = crate::story_system::StorySystemEngine::new(pool);
    engine.get_runtime_contract(&story_id, chapter_number)
}


#[tauri::command(rename_all = "snake_case")]
pub fn init_chapter_commit(
    story_id: String,
    scene_id: Option<String>,
    chapter_id: Option<String>,
    chapter_number: i32,
) -> Result<crate::db::SceneCommit, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let service = crate::story_system::SceneCommitService::new(pool);
    service.init_commit(&story_id, scene_id.as_deref(), chapter_id.as_deref(), chapter_number)
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_chapter_commits(story_id: String) -> Result<Vec<crate::db::SceneCommit>, AppError> {
    let pool = get_pool().ok_or_else(|| AppError::internal("Database not initialized"))?;
    let repo = crate::db::SceneCommitRepository::new(pool);
    repo.get_by_story(&story_id).map_err(AppError::from)
}


#[tauri::command(rename_all = "snake_case")]
pub fn list_genesis_runs(limit: Option<i64>) -> Result<Vec<crate::db::GenesisRun>, AppError> {
    let pool = get_pool().ok_or_else(|| AppError::internal("Database not initialized"))?;
    let repo = crate::db::GenesisRunRepository::new(pool);
    repo.list_all(limit.unwrap_or(100)).map_err(AppError::from)
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_genesis_run(id: String) -> Result<Option<crate::db::GenesisRun>, AppError> {
    let pool = get_pool().ok_or_else(|| AppError::internal("Database not initialized"))?;
    let repo = crate::db::GenesisRunRepository::new(pool);
    repo.get_by_id(&id).map_err(AppError::from)
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_latest_style_snapshot(story_id: String) -> Result<Option<crate::db::models_v3::StyleSnapshot>, AppError> {
    let pool = get_pool().ok_or_else(|| AppError::internal("Database not initialized"))?;
    let repo = crate::db::StyleSnapshotRepository::new(pool);
    repo.get_latest_by_story(&story_id).map_err(AppError::from)
}

