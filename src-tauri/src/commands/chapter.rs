//! Chapter commands

use tauri::{AppHandle, State};

use crate::{
    db::{ChapterRepository, CreateChapterRequest, DbPool},
    error::AppError,
    story_system::chapter_service::ChapterService,
};

#[tauri::command(rename_all = "snake_case")]
pub fn get_story_chapters(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::Chapter>, AppError> {
    crate::db::ChapterRepository::new(pool.inner().clone())
        .get_by_story(&story_id)
        .map_err(AppError::from)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_story_chapters_paged(
    story_id: String,
    limit: i64,
    offset: i64,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::Chapter>, AppError> {
    crate::db::ChapterRepository::new(pool.inner().clone())
        .get_by_story_paged(&story_id, limit, offset)
        .map_err(AppError::from)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_chapter(
    id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<crate::db::Chapter>, AppError> {
    crate::db::ChapterRepository::new(pool.inner().clone())
        .get_by_id(&id)
        .map_err(AppError::from)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_chapter(
    id: String,
    title: Option<String>,
    outline: Option<String>,
    content: Option<String>,
    word_count: Option<i32>,
    pool: State<'_, DbPool>,
    app: AppHandle,
    automation_service: tauri::State<'_, crate::automation::service::AutomationService>,
    vector_store: State<'_, std::sync::Arc<dyn crate::ports::VectorStore>>,
) -> Result<(), AppError> {
    let pool = pool.inner().clone();
    let automation_service = automation_service.inner().clone();
    let vector_store = vector_store.inner().clone();

    // v0.23.20: DB 操作用 spawn_blocking 包裹，防止连接池满时阻塞 Tauri async
    // runtime。 根因：update_chapter 此前是同步 fn，pool.get()
    // 在连接池耗尽时无限阻塞， 导致前端"保存中..."永不消失，同时阻塞 Genesis
    // 等创作流程。
    let title_for_update = title.clone();
    let chapter_info = tokio::task::spawn_blocking({
        let pool = pool.clone();
        let id = id.clone();
        move || {
            let repo = crate::db::ChapterRepository::new(pool);
            // 先查询 story_id 和 chapter_number（P0-3 修复: 避免 unwrap_or_default
            // 导致空字符串）
            let info = repo.get_by_id(&id).ok().flatten();
            // 执行更新
            repo.update(&id, title_for_update, outline, content, word_count)
                .map_err(AppError::from)?;
            Ok::<_, AppError>(info)
        }
    })
    .await
    .map_err(|e| AppError::Internal {
        message: format!("spawn_blocking panicked: {}", e),
    })??;

    let story_id_opt = chapter_info.as_ref().map(|c| c.story_id.clone());
    let chapter_number = chapter_info.map(|c| c.chapter_number).unwrap_or(0);

    // 委托领域服务处理业务编排（内部 spawn 后台任务，不阻塞）
    if let Some(ref story_id) = story_id_opt {
        let service = ChapterService::new(pool, app, vector_store);
        service.on_chapter_updated(
            &id,
            story_id,
            chapter_number,
            title,
            word_count,
            &automation_service,
        );
    }

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_chapter(id: String, pool: State<'_, DbPool>, app: AppHandle) -> Result<(), AppError> {
    let repo = crate::db::ChapterRepository::new(pool.inner().clone());
    // 先查询 story_id，删除后无法再获取（P0-3 修复: 避免 unwrap_or_default
    // 导致空字符串）
    let story_id_opt = repo.get_by_id(&id).ok().flatten().map(|c| c.story_id);
    repo.delete(&id).map_err(AppError::from)?;
    if let Some(story_id) = story_id_opt {
        let _ = crate::state_sync::StateSync::emit_chapter_deleted(&app, &id, &story_id);
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_chapter(
    story_id: String,
    chapter_number: i32,
    title: Option<String>,
    outline: Option<String>,
    content: Option<String>,
    pool: State<'_, DbPool>,
    app: AppHandle,
    automation_service: tauri::State<'_, crate::automation::service::AutomationService>,
    vector_store: State<'_, std::sync::Arc<dyn crate::ports::VectorStore>>,
) -> Result<crate::db::Chapter, AppError> {
    let pool = pool.inner().clone();
    let repo = ChapterRepository::new(pool.clone());

    // 如果该 chapter_number 已存在，直接返回已有章节（幂等）
    if let Ok(chapters) = repo.get_by_story(&story_id) {
        if let Some(existing) = chapters
            .into_iter()
            .find(|c| c.chapter_number == chapter_number)
        {
            log::info!(
                "[create_chapter] Chapter {} already exists for story {}, returning existing",
                chapter_number,
                story_id
            );
            return Ok(existing);
        }
    }

    let req = CreateChapterRequest {
        story_id: story_id.clone(),
        chapter_number,
        title: title.clone(),
        outline,
        content,
    };
    let chapter = repo.create(req).map_err(AppError::from)?;

    // 委托领域服务处理后续业务编排
    let service = ChapterService::new(pool, app, vector_store.inner().clone());
    service.on_chapter_created(&chapter, title, automation_service.inner());

    Ok(chapter)
}
