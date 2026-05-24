//! Genre commands

use crate::get_pool;

// ==================== Genre Profile Commands ====================

#[tauri::command(rename_all = "snake_case")]
pub fn get_genre_profiles() -> Result<Vec<crate::db::GenreProfile>, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let repo = crate::db::GenreProfileRepository::new(pool);
    repo.get_all().map_err(|e| crate::error::AppError::from(e).to_string())
}


#[tauri::command(rename_all = "snake_case")]
pub fn get_genre_profile(genre_name: String) -> Result<Option<crate::db::GenreProfile>, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let repo = crate::db::GenreProfileRepository::new(pool);
    repo.get_by_name(&genre_name).map_err(|e| crate::error::AppError::from(e).to_string())
}


#[tauri::command(rename_all = "snake_case")]
pub fn save_genre_profile(
    id: Option<String>,
    genre_name: String,
    canonical_name: String,
    aliases_json: Option<String>,
    core_tone: Option<String>,
    pacing_strategy: Option<String>,
    anti_patterns_json: Option<String>,
    reference_tables_json: Option<String>,
) -> Result<crate::db::GenreProfile, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let repo = crate::db::GenreProfileRepository::new(pool);

    if let Some(existing_id) = id {
        // 更新现有记录
        repo.update(
            &existing_id,
            &genre_name,
            &canonical_name,
            aliases_json.as_deref(),
            core_tone.as_deref(),
            pacing_strategy.as_deref(),
            anti_patterns_json.as_deref(),
            reference_tables_json.as_deref(),
        ).map_err(|e| crate::error::AppError::from(e).to_string())?;
        repo.get_by_id(&existing_id)
            .map_err(|e| crate::error::AppError::from(e).to_string())?
            .ok_or_else(|| "更新后未找到记录".to_string())
    } else {
        // 创建新记录（is_builtin = 0）
        repo.create(
            &genre_name,
            &canonical_name,
            aliases_json.as_deref(),
            core_tone.as_deref(),
            pacing_strategy.as_deref(),
            anti_patterns_json.as_deref(),
            reference_tables_json.as_deref(),
        ).map_err(|e| crate::error::AppError::from(e).to_string())
    }
}


#[tauri::command(rename_all = "snake_case")]
pub fn delete_genre_profile(id: String) -> Result<usize, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let repo = crate::db::GenreProfileRepository::new(pool);
    // 只允许删除非内置体裁
    if let Ok(Some(profile)) = repo.get_by_id(&id) {
        if profile.is_builtin {
            return Err("内置体裁不可删除".to_string());
        }
    }
    repo.delete(&id).map_err(|e| crate::error::AppError::from(e).to_string())
}

