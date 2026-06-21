//! Strategy selection commands

use tauri::{AppHandle, State};

use crate::{
    db::DbPool,
    domain::strategy::StrategyOverrides,
    error::AppError,
    llm::LlmService,
    skills::SkillManager,
    strategy::{load_all_assets, SelectionContext, StrategySelector},
};

/// 预览模型为当前创作场景推荐的策略组合
#[tauri::command(rename_all = "snake_case")]
pub async fn select_creation_strategy(
    app_handle: AppHandle,
    pool: State<'_, DbPool>,
    llm_service: State<'_, LlmService>,
    user_input: String,
    genre_hint: Option<String>,
    methodology_hint: Option<String>,
    word_count_target: Option<i32>,
    story_id: Option<String>,
    overrides: Option<StrategyOverrides>,
) -> Result<crate::domain::strategy::SelectedStrategy, AppError> {
    let repo = crate::db::GenreProfileRepository::new(pool.inner().clone());
    let skills = SkillManager::from_app_handle(&app_handle).get_all_skills();

    let assets = load_all_assets(&repo, &skills).map_err(AppError::from)?;

    let context = SelectionContext {
        user_input,
        genre_hint,
        methodology_hint,
        word_count_target,
        story_id,
        ..Default::default()
    };

    let selector = StrategySelector::new(llm_service.inner().clone(), pool.inner().clone());
    selector
        .select_strategy(&context, &assets, Some(&repo), overrides.as_ref())
        .await
}
