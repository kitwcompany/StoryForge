use super::types::*;
use super::{PipelineOrchestrator, PostProcessRunWithSteps};
use crate::db::DbPool;
use crate::llm::LlmService;
use tauri::{command, AppHandle, State};

/// 执行 AI 修稿
#[command(rename_all = "snake_case")]
pub async fn run_refine(
    story_id: String,
    draft_id: String,
    user_prompt: Option<String>,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<RefineResult, String> {
    let config = PipelineConfig::default();
    let llm_service = LlmService::new(app_handle);
    let callbacks = super::types::SilentCallbacks;

    super::refine_draft(
        &story_id,
        &draft_id,
        user_prompt.as_deref(),
        &config,
        pool.inner(),
        &llm_service,
        &callbacks,
    ).await.map_err(|e| e.to_string())
}

/// 执行 AI 审稿
#[command(rename_all = "snake_case")]
pub async fn run_review(
    story_id: String,
    draft_id: String,
    review_focus: Option<String>,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<ReviewResult, String> {
    let config = PipelineConfig::default();
    let llm_service = LlmService::new(app_handle);
    let callbacks = super::types::SilentCallbacks;

    super::review_draft(
        &story_id,
        &draft_id,
        review_focus.as_deref(),
        &config,
        pool.inner(),
        &llm_service,
        &callbacks,
    ).await.map_err(|e| e.to_string())
}

/// 执行定稿与后处理
#[command(rename_all = "snake_case")]
pub async fn run_finalize(
    story_id: String,
    draft_id: String,
    chapter_number: i32,
    chapter_title: Option<String>,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<PipelineResult, String> {
    let config = PipelineConfig::default();
    let callbacks = super::types::SilentCallbacks;
    let chapter_info = ChapterInfo { chapter_number, title: chapter_title };

    let post_process_run_id = super::finalize_draft(
        &story_id,
        &draft_id,
        &chapter_info,
        &config,
        pool.inner(),
        &app_handle,
        &callbacks,
    ).await.map_err(|e| e.to_string())?;

    Ok(PipelineResult {
        draft_id: draft_id.clone(),
        chapter_number,
        refined_draft_id: None,
        review_id: None,
        finalized_draft_id: Some(draft_id),
        post_process_run_id: if post_process_run_id.is_empty() { None } else { Some(post_process_run_id) },
        success: true,
        message: "定稿完成".to_string(),
    })
}

/// 修复定稿后处理 — 当后处理失败时重跑
#[command(rename_all = "snake_case")]
pub async fn repair_finalize(
    story_id: String,
    chapter_number: i32,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<PipelineResult, String> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());

    // 获取已定稿的草稿
    let draft = orchestrator.get_finalized_draft(&story_id, chapter_number)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "未找到已定稿的草稿".to_string())?;

    let config = PipelineConfig::default();
    let callbacks = super::types::SilentCallbacks;
    let chapter_info = ChapterInfo { chapter_number, title: None };

    let post_process_run_id = super::finalize_draft(
        &story_id,
        &draft.id,
        &chapter_info,
        &config,
        pool.inner(),
        &app_handle,
        &callbacks,
    ).await.map_err(|e| e.to_string())?;

    Ok(PipelineResult {
        draft_id: draft.id.clone(),
        chapter_number,
        refined_draft_id: None,
        review_id: None,
        finalized_draft_id: Some(draft.id),
        post_process_run_id: if post_process_run_id.is_empty() { None } else { Some(post_process_run_id) },
        success: true,
        message: "后处理修复完成".to_string(),
    })
}

/// 获取后处理运行状态（含步骤详情）
#[command(rename_all = "snake_case")]
pub async fn get_post_process_status(
    run_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<PostProcessRunWithSteps>, String> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    orchestrator.get_post_process_status(&run_id).map_err(|e| e.to_string())
}

/// 获取管线编排器状态 — 指定章节当前活跃草稿
#[command(rename_all = "snake_case")]
pub async fn get_pipeline_active_draft(
    story_id: String,
    chapter_number: i32,
    pool: State<'_, DbPool>,
) -> Result<Option<crate::db::Draft>, String> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    orchestrator.get_active_draft(&story_id, chapter_number).map_err(|e| e.to_string())
}

/// 合并修稿（用户接受修稿结果）
#[command(rename_all = "snake_case")]
pub async fn merge_revision(
    revision_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    orchestrator.merge_revision(&revision_id).map_err(|e| e.to_string())
}

/// 获取草稿的修稿历史
#[command(rename_all = "snake_case")]
pub async fn get_draft_revision_history(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::Revision>, String> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    orchestrator.get_draft_revision_history(&draft_id).map_err(|e| e.to_string())
}

/// 获取草稿的审稿历史
#[command(rename_all = "snake_case")]
pub async fn get_draft_review_history(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::PipelineReview>, String> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    orchestrator.get_draft_review_history(&draft_id).map_err(|e| e.to_string())
}
