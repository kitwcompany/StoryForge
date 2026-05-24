//! Reading Power commands

use crate::get_pool;

// ==================== Reading Power Commands ====================

#[tauri::command(rename_all = "snake_case")]
pub fn evaluate_reading_power(
    story_id: String,
    chapter_number: i32,
) -> Result<crate::reading_power::ReadingPowerEvaluation, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let evaluator = crate::reading_power::ReadingPowerEvaluator::new(pool);
    evaluator.evaluate(&story_id, chapter_number)
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_reading_power_trend(
    story_id: String,
    last_n: i64,
) -> Result<Vec<crate::reading_power::ReadingPowerEvaluation>, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let evaluator = crate::reading_power::ReadingPowerEvaluator::new(pool);
    evaluator.get_trend(&story_id, last_n)
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_chase_debts(story_id: String) -> Result<Vec<crate::db::ChaseDebt>, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let repo = crate::db::ChaseDebtRepository::new(pool);
    repo.get_active_by_story(&story_id).map_err(|e| crate::error::AppError::from(e).to_string())
}


#[tauri::command(rename_all = "snake_case")]
pub fn create_override_contract(
    story_id: String,
    chapter_number: i32,
    constraint_type: String,
    constraint_id: String,
    rationale_type: String,
    rationale_text: String,
    payback_plan: String,
    due_chapter: i32,
) -> Result<crate::db::OverrideContract, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let manager = crate::reading_power::DebtManager::new(pool);
    manager.create_override_contract(
        &story_id, chapter_number, &constraint_type, &constraint_id,
        &rationale_type, &rationale_text, &payback_plan, due_chapter
    )
}

