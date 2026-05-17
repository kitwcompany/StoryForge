//! Pipeline 管线体系 Tauri 命令 (v7.0.0)

use crate::db::*;
use crate::error::AppError;
use tauri::{command, State};

// ==================== Blueprint Commands ====================

#[command(rename_all = "snake_case")]
pub async fn create_blueprint(
    req: CreateBlueprintRequest,
    pool: State<'_, DbPool>,
) -> Result<Blueprint, AppError> {
    let repo = BlueprintRepository::new(pool.inner().clone());
    repo.create(req).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_story_blueprints(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<Blueprint>, AppError> {
    let repo = BlueprintRepository::new(pool.inner().clone());
    repo.get_by_story(&story_id).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_chapter_blueprint(
    story_id: String,
    chapter_number: i32,
    pool: State<'_, DbPool>,
) -> Result<Option<Blueprint>, AppError> {
    let repo = BlueprintRepository::new(pool.inner().clone());
    repo.get_by_chapter(&story_id, chapter_number).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn update_blueprint(
    blueprint_id: String,
    req: UpdateBlueprintRequest,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = BlueprintRepository::new(pool.inner().clone());
    repo.update(&blueprint_id, req).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn delete_blueprint(
    blueprint_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = BlueprintRepository::new(pool.inner().clone());
    repo.delete(&blueprint_id).map_err(AppError::from)
}

// ==================== Draft Commands ====================

#[command(rename_all = "snake_case")]
pub async fn create_draft(
    story_id: String,
    chapter_number: i32,
    version: i32,
    status: String,
    source: String,
    content: String,
    word_count: i32,
    model_used: Option<String>,
    cost: Option<f64>,
    metadata: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<Draft, AppError> {
    let repo = DraftRepository::new(pool.inner().clone());
    let status = status.parse()?;
    let source = source.parse()?;
    repo.create(
        &story_id, chapter_number, version, status, source, &content,
        word_count, model_used.as_deref(), cost, metadata.as_deref(),
    ).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_draft(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<Draft>, AppError> {
    let repo = DraftRepository::new(pool.inner().clone());
    repo.get_by_id(&draft_id).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_story_chapter_drafts(
    story_id: String,
    chapter_number: i32,
    pool: State<'_, DbPool>,
) -> Result<Vec<Draft>, AppError> {
    let repo = DraftRepository::new(pool.inner().clone());
    repo.get_by_story_chapter(&story_id, chapter_number).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_latest_draft(
    story_id: String,
    chapter_number: i32,
    pool: State<'_, DbPool>,
) -> Result<Option<Draft>, AppError> {
    let repo = DraftRepository::new(pool.inner().clone());
    repo.get_latest_by_chapter(&story_id, chapter_number).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_finalized_draft(
    story_id: String,
    chapter_number: i32,
    pool: State<'_, DbPool>,
) -> Result<Option<Draft>, AppError> {
    let repo = DraftRepository::new(pool.inner().clone());
    repo.get_finalized_by_chapter(&story_id, chapter_number).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn update_draft_status(
    draft_id: String,
    status: String,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = DraftRepository::new(pool.inner().clone());
    let status = status.parse()?;
    repo.update_status(&draft_id, status).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn update_draft_content(
    draft_id: String,
    content: String,
    word_count: i32,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = DraftRepository::new(pool.inner().clone());
    repo.update_content(&draft_id, &content, word_count).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn delete_draft(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = DraftRepository::new(pool.inner().clone());
    repo.delete(&draft_id).map_err(AppError::from)
}

// ==================== Revision Commands ====================

#[command(rename_all = "snake_case")]
pub async fn create_revision(
    story_id: String,
    draft_id: String,
    revision_index: i32,
    revision_type: String,
    user_prompt: Option<String>,
    original_content: String,
    revised_content: String,
    word_count: i32,
    change_summary: Option<String>,
    model_used: Option<String>,
    cost: Option<f64>,
    metadata: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<Revision, AppError> {
    let repo = RevisionRepository::new(pool.inner().clone());
    let revision_type = revision_type.parse()?;
    repo.create(
        &story_id, &draft_id, revision_index, revision_type,
        user_prompt.as_deref(), &original_content, &revised_content,
        word_count, change_summary.as_deref(), model_used.as_deref(),
        cost, metadata.as_deref(),
    ).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_draft_revisions(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<Revision>, AppError> {
    let repo = RevisionRepository::new(pool.inner().clone());
    repo.get_by_draft(&draft_id).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_revision(
    revision_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<Revision>, AppError> {
    let repo = RevisionRepository::new(pool.inner().clone());
    repo.get_by_id(&revision_id).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn update_revision_status(
    revision_id: String,
    status: String,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = RevisionRepository::new(pool.inner().clone());
    let status = status.parse()?;
    repo.update_status(&revision_id, status).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn delete_revision(
    revision_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = RevisionRepository::new(pool.inner().clone());
    repo.delete(&revision_id).map_err(AppError::from)
}

// ==================== Pipeline Review Commands ====================

#[command(rename_all = "snake_case")]
pub async fn create_pipeline_review(
    story_id: String,
    draft_id: String,
    review_index: i32,
    content: String,
    dimensions: Option<Vec<ReviewDimension>>,
    issues: Option<Vec<ReviewIssueItem>>,
    overall_score: Option<f32>,
    review_focus: Option<String>,
    model_used: Option<String>,
    cost: Option<f64>,
    metadata: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<PipelineReview, AppError> {
    let repo = PipelineReviewRepository::new(pool.inner().clone());
    repo.create(
        &story_id, &draft_id, review_index, &content,
        dimensions.as_deref(), issues.as_deref(),
        overall_score, review_focus.as_deref(), model_used.as_deref(),
        cost, metadata.as_deref(),
    ).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_draft_reviews(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<PipelineReview>, AppError> {
    let repo = PipelineReviewRepository::new(pool.inner().clone());
    repo.get_by_draft(&draft_id).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_latest_pipeline_review(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<PipelineReview>, AppError> {
    let repo = PipelineReviewRepository::new(pool.inner().clone());
    repo.get_latest_by_draft(&draft_id).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn delete_pipeline_review(
    review_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = PipelineReviewRepository::new(pool.inner().clone());
    repo.delete(&review_id).map_err(AppError::from)
}

// ==================== Post Process Commands ====================

#[command(rename_all = "snake_case")]
pub async fn create_post_process_run(
    story_id: String,
    chapter_number: i32,
    source_label: String,
    scope: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<PostProcessRun, AppError> {
    let repo = PostProcessRepository::new(pool.inner().clone());
    repo.create_run(&story_id, chapter_number, &source_label, scope.as_deref(),
    ).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_post_process_run(
    run_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<PostProcessRun>, AppError> {
    let repo = PostProcessRepository::new(pool.inner().clone());
    repo.get_run_by_id(&run_id).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_story_chapter_post_process_runs(
    story_id: String,
    chapter_number: i32,
    pool: State<'_, DbPool>,
) -> Result<Vec<PostProcessRun>, AppError> {
    let repo = PostProcessRepository::new(pool.inner().clone());
    repo.get_runs_by_story_chapter(&story_id, chapter_number).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn update_post_process_run_status(
    run_id: String,
    status: String,
    error_message: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = PostProcessRepository::new(pool.inner().clone());
    let status = status.parse()?;
    repo.update_run_status(&run_id, status, error_message.as_deref()).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn create_post_process_step(
    run_id: String,
    step_key: String,
    step_label: String,
    critical: bool,
    pool: State<'_, DbPool>,
) -> Result<PostProcessStep, AppError> {
    let repo = PostProcessRepository::new(pool.inner().clone());
    repo.create_step(&run_id, &step_key, &step_label, critical,
    ).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_post_process_steps(
    run_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<PostProcessStep>, AppError> {
    let repo = PostProcessRepository::new(pool.inner().clone());
    repo.get_steps_by_run(&run_id).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn update_post_process_step_status(
    step_id: String,
    status: String,
    log_output: Option<String>,
    error_message: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = PostProcessRepository::new(pool.inner().clone());
    let status = status.parse()?;
    repo.update_step_status(&step_id, status, log_output.as_deref(), error_message.as_deref(),
    ).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn delete_post_process_run(
    run_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = PostProcessRepository::new(pool.inner().clone());
    repo.delete_run(&run_id).map_err(AppError::from)
}

// ==================== LLM Call Commands ====================

#[command(rename_all = "snake_case")]
pub async fn record_llm_call(
    req: RecordLlmCallRequest,
    total_tokens: i32,
    duration_ms: i32,
    prompt_preview: Option<String>,
    metadata: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<LlmCall, AppError> {
    let repo = LlmCallRepository::new(pool.inner().clone());
    repo.create(req, total_tokens, duration_ms, prompt_preview.as_deref(), metadata.as_deref())
        .map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_story_llm_calls(
    story_id: String,
    limit: i64,
    pool: State<'_, DbPool>,
) -> Result<Vec<LlmCall>, AppError> {
    let repo = LlmCallRepository::new(pool.inner().clone());
    repo.get_by_story(&story_id, limit).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_recent_llm_calls(
    limit: i64,
    pool: State<'_, DbPool>,
) -> Result<Vec<LlmCall>, AppError> {
    let repo = LlmCallRepository::new(pool.inner().clone());
    repo.get_recent(limit).map_err(AppError::from)
}

#[derive(serde::Serialize)]
pub struct LlmCallStats {
    pub count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[command(rename_all = "snake_case")]
pub async fn get_llm_call_stats(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<LlmCallStats, AppError> {
    let repo = LlmCallRepository::new(pool.inner().clone());
    let (count, total_tokens, total_cost) = repo.get_stats_by_story(&story_id).map_err(AppError::from)?;
    Ok(LlmCallStats { count, total_tokens, total_cost })
}

// ==================== Character State Commands ====================

#[command(rename_all = "snake_case")]
pub async fn update_character_state(
    character_id: String,
    state: CharacterState,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = CharacterRepository::new(pool.inner().clone());
    repo.update_character_state(&character_id, &state).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn batch_update_character_states(
    updates: Vec<(String, CharacterState)>,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let repo = CharacterRepository::new(pool.inner().clone());
    let updates_ref: Vec<(String, CharacterState)> = updates;
    repo.batch_update_states(&updates_ref).map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_character_state(
    character_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<CharacterState>, AppError> {
    let repo = CharacterRepository::new(pool.inner().clone());
    repo.get_character_state(&character_id).map_err(AppError::from)
}
