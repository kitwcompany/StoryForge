#![allow(dead_code)]
//! Reference Book Repository
//!
//! 参考书籍、人物、场景的数据库存取层。

use chrono::Local;
use rusqlite::{params, OptionalExtension};

use super::models::*;
use crate::db::DbPool;

pub struct ReferenceBookRepository {
    pool: DbPool,
}

impl ReferenceBookRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 创建参考书籍记录
    pub fn create(&self, book: &ReferenceBook) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO reference_books (id, title, author, genre, word_count, file_format, \
             file_hash, file_path, world_setting, plot_summary, story_arc, analysis_status, \
             analysis_progress, analysis_error, task_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                book.id,
                book.title,
                book.author,
                book.genre,
                book.word_count,
                book.file_format,
                book.file_hash,
                book.file_path,
                book.world_setting,
                book.plot_summary,
                book.story_arc,
                book.analysis_status.to_string(),
                book.analysis_progress,
                book.analysis_error,
                book.task_id,
                book.created_at.to_rfc3339(),
                book.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// 根据ID获取
    pub fn get_by_id(&self, id: &str) -> Result<Option<ReferenceBook>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, author, genre, word_count, file_format, file_hash, file_path, \
             world_setting, plot_summary, story_arc, analyzed_structure_json, analysis_status, \
             analysis_progress, analysis_error, task_id, created_at, updated_at
             FROM reference_books WHERE id = ?1",
        )?;

        let book = stmt
            .query_row([id], |row| {
                let status_str: String = row.get(12)?;
                let status = status_str.parse().unwrap_or(AnalysisStatus::Pending);

                Ok(ReferenceBook {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    author: row.get(2)?,
                    genre: row.get(3)?,
                    word_count: row.get(4)?,
                    file_format: row.get(5)?,
                    file_hash: row.get(6)?,
                    file_path: row.get(7)?,
                    world_setting: row.get(8)?,
                    plot_summary: row.get(9)?,
                    story_arc: row.get(10)?,
                    analyzed_structure_json: row.get(11)?,
                    analysis_status: status,
                    analysis_progress: row.get(13)?,
                    analysis_error: row.get(14)?,
                    task_id: row.get(15)?,
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            })
            .optional()?;

        Ok(book)
    }

    /// 根据文件哈希获取（去重检查）
    pub fn get_by_hash(
        &self,
        hash: &str,
    ) -> Result<Option<ReferenceBook>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, author, genre, word_count, file_format, file_hash, file_path, \
             world_setting, plot_summary, story_arc, analyzed_structure_json, analysis_status, \
             analysis_progress, analysis_error, task_id, created_at, updated_at
             FROM reference_books WHERE file_hash = ?1",
        )?;

        let book = stmt
            .query_row([hash], |row| {
                let status_str: String = row.get(12)?;
                let status = status_str.parse().unwrap_or(AnalysisStatus::Pending);

                Ok(ReferenceBook {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    author: row.get(2)?,
                    genre: row.get(3)?,
                    word_count: row.get(4)?,
                    file_format: row.get(5)?,
                    file_hash: row.get(6)?,
                    file_path: row.get(7)?,
                    world_setting: row.get(8)?,
                    plot_summary: row.get(9)?,
                    story_arc: row.get(10)?,
                    analyzed_structure_json: row.get(11)?,
                    analysis_status: status,
                    analysis_progress: row.get(13)?,
                    analysis_error: row.get(14)?,
                    task_id: row.get(15)?,
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            })
            .optional()?;

        Ok(book)
    }

    /// 获取列表
    pub fn list_all(&self) -> Result<Vec<ReferenceBookListItem>, Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, author, genre, word_count, file_format, analysis_status, \
             analysis_progress, created_at
             FROM reference_books ORDER BY created_at DESC",
        )?;

        let books = stmt
            .query_map([], |row| {
                Ok(ReferenceBookListItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    author: row.get(2)?,
                    genre: row.get(3)?,
                    word_count: row.get(4)?,
                    file_format: row.get(5)?,
                    analysis_status: row.get(6)?,
                    analysis_progress: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(books)
    }

    /// 获取拆书分析摘要（用于策略选择 prompt 注入）。
    ///
    /// 从 `reference_books` 读取基础信息，并从 `narrative_scenes`
    /// 聚合世界观关键词与基调。任何步骤失败均返回 `Ok(None)`，
    /// 避免阻塞策略选择。
    pub fn get_book_analysis_summary(
        &self,
        book_id: &str,
    ) -> Result<Option<ReferenceBookSummary>, rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, title, genre, world_setting, story_arc
             FROM reference_books WHERE id = ?1",
        )?;

        let book = stmt
            .query_row([book_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })
            .optional()?;

        let (book_id, title, genre, world_setting, story_arc) = match book {
            Some(b) => b,
            None => return Ok(None),
        };

        // 聚合场景信息：summary / key_events / conflict_type / emotional_tone
        let mut stmt = conn.prepare(
            "SELECT summary, key_events, conflict_type, emotional_tone
             FROM narrative_scenes WHERE story_id = ?1 AND source = 'extracted'",
        )?;

        let scenes = stmt
            .query_map([book_id.as_str()], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut world_keywords = Vec::new();
        if let Some(ws) = world_setting {
            world_keywords.extend(extract_keywords(&ws));
        }

        let mut tone_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for (summary, key_events, conflict_type, emotional_tone) in scenes {
            if let Some(s) = summary {
                world_keywords.extend(extract_keywords(&s));
            }
            if let Some(ke) = key_events {
                world_keywords.extend(extract_keywords(&ke));
            }
            if let Some(ct) = conflict_type {
                if !ct.is_empty() && !world_keywords.contains(&ct) {
                    world_keywords.push(ct);
                }
            }
            if let Some(et) = emotional_tone {
                if !et.is_empty() {
                    *tone_counts.entry(et).or_insert(0) += 1;
                }
            }
        }

        world_keywords.truncate(20);

        let tone = tone_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| t);

        Ok(Some(ReferenceBookSummary {
            book_id,
            title,
            genre,
            world_keywords,
            arc_type: story_arc,
            tone,
        }))
    }

    /// 更新分析状态和进度
    pub fn update_status(
        &self,
        id: &str,
        status: AnalysisStatus,
        progress: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE reference_books SET analysis_status = ?1, analysis_progress = ?2, updated_at \
             = ?3 WHERE id = ?4",
            params![status.to_string(), progress, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// 更新关联的任务ID
    pub fn update_task_id(
        &self,
        id: &str,
        task_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE reference_books SET task_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![task_id, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// 更新分析结果
    pub fn update_analysis_result(
        &self,
        id: &str,
        title: Option<&str>,
        author: Option<&str>,
        genre: Option<&str>,
        world_setting: Option<&str>,
        plot_summary: Option<&str>,
        story_arc: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE reference_books SET title = COALESCE(?1, title), author = COALESCE(?2, \
             author), genre = ?3, world_setting = ?4, plot_summary = ?5, story_arc = ?6, \
             updated_at = ?7 WHERE id = ?8",
            params![
                title,
                author,
                genre,
                world_setting,
                plot_summary,
                story_arc,
                Local::now().to_rfc3339(),
                id
            ],
        )?;
        Ok(())
    }

    /// 更新分析结果（含叙事结构）
    pub fn update_analysis_result_with_structure(
        &self,
        id: &str,
        title: Option<&str>,
        author: Option<&str>,
        genre: Option<&str>,
        world_setting: Option<&str>,
        plot_summary: Option<&str>,
        story_arc: Option<&str>,
        analyzed_structure_json: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE reference_books SET title = COALESCE(?1, title), author = COALESCE(?2, \
             author), genre = ?3, world_setting = ?4, plot_summary = ?5, story_arc = ?6, \
             analyzed_structure_json = ?7, updated_at = ?8 WHERE id = ?9",
            params![
                title,
                author,
                genre,
                world_setting,
                plot_summary,
                story_arc,
                analyzed_structure_json,
                Local::now().to_rfc3339(),
                id
            ],
        )?;
        Ok(())
    }

    /// 更新错误信息
    pub fn update_error(&self, id: &str, error: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE reference_books SET analysis_status = 'failed', analysis_error = ?1, \
             updated_at = ?2 WHERE id = ?3",
            params![error, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// 删除
    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM reference_books WHERE id = ?1", [id])?;
        Ok(())
    }
}

/// 从文本中提取候选关键词（用于 `get_book_analysis_summary`）。
///
/// 简单实现：按常见中英文标点分词，过滤过短字符，去重。
fn extract_keywords(text: &str) -> Vec<String> {
    let delimiters: &[char] = &[
        ' ', '\t', '\n', '\r', ',', '，', '.', '。', ';', '；', ':', '：', '!', '！', '?', '？',
        '\"', '\'', '(', ')', '（', '）', '[', ']', '【', '】', '{', '}', '/', '、', '|', '｜',
        '&', '#', '*', '@', '$', '%', '^', '<', '>', '~', '`', '-', '_', '+', '=',
    ];

    let mut seen = std::collections::HashSet::new();
    text.split(delimiters)
        .map(|s| s.trim())
        .filter(|s| {
            // 保留中文词（>=2 字）或英文词（>=3 字母）
            let byte_len = s.bytes().len();
            let char_len = s.chars().count();
            !s.is_empty() && byte_len >= 2 && char_len >= 2
        })
        .filter_map(|s| {
            let owned = s.to_string();
            if seen.insert(owned.clone()) {
                Some(owned)
            } else {
                None
            }
        })
        .collect()
}
