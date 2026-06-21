#![allow(dead_code)]
use std::sync::Arc;

use tauri::command;

use super::{KbImportResult, KbSearchResult, KbStats};
use crate::{error::AppError, ports::VectorStore};

/// 导入文本到知识库（定稿后调用）
#[command(rename_all = "snake_case")]
pub async fn kb_import_text(
    story_id: String,
    chapter_number: i32,
    content: String,
    source_label: String,
    store: tauri::State<'_, Arc<dyn VectorStore>>,
) -> Result<KbImportResult, AppError> {
    super::import_text(
        store.inner().as_ref(),
        &story_id,
        chapter_number,
        &content,
        &source_label,
    )
    .await
    .map_err(AppError::from)
}

/// 语义检索
#[command(rename_all = "snake_case")]
pub async fn kb_search(
    story_id: String,
    query: String,
    top_k: Option<usize>,
    chapter_range: Option<(i32, i32)>,
    search_mode: Option<String>,
    store: tauri::State<'_, Arc<dyn VectorStore>>,
) -> Result<Vec<KbSearchResult>, AppError> {
    let top_k = top_k.unwrap_or(5);
    let mode = search_mode.as_deref().unwrap_or("hybrid");

    super::kb_search(
        store.inner().as_ref(),
        &story_id,
        &query,
        top_k,
        chapter_range,
        mode,
    )
    .await
    .map_err(AppError::from)
}

/// 删除某章的向量记录
#[command(rename_all = "snake_case")]
pub async fn kb_delete_chapter(
    story_id: String,
    chapter_number: i32,
    store: tauri::State<'_, Arc<dyn VectorStore>>,
) -> Result<usize, AppError> {
    super::delete_chapter_vectors(store.inner().as_ref(), &story_id, chapter_number)
        .await
        .map_err(AppError::from)
}

/// 获取知识库统计（简化版）
#[command(rename_all = "snake_case")]
pub async fn kb_stats(_story_id: String) -> Result<KbStats, AppError> {
    // LanceDB 目前没有直接的 count API，通过搜索空字符串获取估算值
    // 实际实现中可维护统计表
    Ok(KbStats {
        total_vectors: 0,
        total_chapters: 0,
        last_imported_at: None,
    })
}
