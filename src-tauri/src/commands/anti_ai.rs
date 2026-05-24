//! Anti Ai commands

use crate::get_pool;

// ==================== Anti-AI Review Command ====================

#[tauri::command(rename_all = "snake_case")]
pub fn anti_ai_review(
    text: String,
    genre: Option<String>,
) -> Result<crate::anti_ai::AntiAiReview, String> {
    let reviewer = crate::anti_ai::AntiAiReviewer::new();
    Ok(reviewer.review(&text, genre.as_deref()))
}


#[tauri::command(rename_all = "snake_case")]
pub fn evolve_style_from_anti_ai_review(
    app: tauri::AppHandle,
    story_id: String,
    review: crate::anti_ai::AntiAiReview,
) -> Result<crate::creative_engine::style::evolution::StyleDnaDelta, String> {
    let pool = get_pool().ok_or("Database not initialized")?;

    // 1. 获取 story 的 style_dna_id
    let story_repo = crate::db::repositories::StoryRepository::new(pool.clone());
    let style_dna_id = match story_repo.get_by_id(&story_id) {
        Ok(Some(story)) => story.style_dna_id,
        Ok(None) => return Err("故事不存在".to_string()),
        Err(e) => return Err(format!("查询故事失败: {}", e)),
    };

    // 2. 获取或创建 StyleDNA
    let dna_repo = crate::db::repositories_v3::StyleDnaRepository::new(pool.clone());
    let (dna_id, base_dna) = if let Some(id) = style_dna_id {
        let dna = dna_repo.get_by_id(&id).map_err(|e| e.to_string())?
            .ok_or("StyleDNA not found")?;
        let base: crate::creative_engine::style::dna::StyleDNA = serde_json::from_str(&dna.dna_json)
            .map_err(|e| format!("Parse StyleDNA failed: {}", e))?;
        (id, base)
    } else {
        let base = crate::creative_engine::style::dna::StyleDNA::new("evolved");
        let json = serde_json::to_string(&base).map_err(|e| e.to_string())?;
        let dna = dna_repo.create("evolved", None, &json, false).map_err(|e| e.to_string())?;
        let conn = pool.get().map_err(|e| e.to_string())?;
        conn.execute("UPDATE stories SET style_dna_id = ?1 WHERE id = ?2", [&dna.id, &story_id])
            .map_err(|e| e.to_string())?;
        (dna.id, base)
    };

    // 3. 运行风格演化
    let engine = crate::creative_engine::style::evolution::StyleEvolutionEngine::new();
    let delta = engine.evolve_from_reviews(&base_dna, Some(&review), None);

    // 4. 如果有实质调整，保存
    if !delta.is_empty() {
        let evolved = delta.apply(&base_dna);
        let json = serde_json::to_string(&evolved).map_err(|e| e.to_string())?;
        dna_repo.update_dna_json(&dna_id, &json).map_err(|e| e.to_string())?;
        crate::state_sync::service::StateSync::emit_style_dna_updated(&app, &story_id, &dna_id);
    }

    Ok(delta)
}


// ==================== Telemetry Commands ====================

#[tauri::command(rename_all = "snake_case")]
pub fn log_frontend_feature_usage(
    feature_id: String,
    action: String,
    story_id: Option<String>,
) {
    if let Some(pool) = get_pool() {
        crate::telemetry::log_feature_usage(&pool, &feature_id, &action, story_id.as_deref(), None);
    }
}

