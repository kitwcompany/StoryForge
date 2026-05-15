use super::types::*;
use crate::db::{DbPool, DraftRepository, DraftStatus, PipelineReviewRepository, BlueprintRepository, CharacterRepository};
use crate::llm::LlmService;

/// 执行 AI 审稿
///
/// 1. 读取 refined 草稿、前文内容、角色状态
/// 2. 构建审稿 prompt（含评审维度）
/// 3. 调用 LLM 生成结构化审稿报告（JSON）
/// 4. 解析 JSON，创建 review 记录
/// 5. 更新 draft 状态为 reviewed
pub async fn review_draft(
    story_id: &str,
    draft_id: &str,
    review_focus: Option<&str>,
    config: &PipelineConfig,
    pool: &DbPool,
    _llm_service: &LlmService,
    callbacks: &dyn PipelineCallbacks,
) -> Result<ReviewResult, PipelineError> {
    callbacks.progress("review", 0.1);

    // 1. 读取草稿
    let draft_repo = DraftRepository::new(pool.clone());
    let draft = draft_repo.get_by_id(draft_id)
        .map_err(|e| PipelineError { phase: "review".to_string(), message: format!("读取草稿失败: {}", e), recoverable: true })?
        .ok_or_else(|| PipelineError { phase: "review".to_string(), message: "草稿不存在".to_string(), recoverable: true })?;

    // 验证草稿状态
    if draft.status != DraftStatus::Refined && draft.status != DraftStatus::Draft {
        return Err(PipelineError {
            phase: "review".to_string(),
            message: format!("草稿状态为 {:?}，无法审稿。请先执行修稿。", draft.status),
            recoverable: true,
        });
    }

    callbacks.progress("review", 0.2);

    // 2. 读取蓝图和角色
    let blueprint_repo = BlueprintRepository::new(pool.clone());
    let blueprint = blueprint_repo.get_by_chapter(story_id, draft.chapter_number)
        .map_err(|e| PipelineError { phase: "review".to_string(), message: format!("读取蓝图失败: {}", e), recoverable: true })?;

    let character_repo = CharacterRepository::new(pool.clone());
    let characters = character_repo.get_by_story(story_id)
        .map_err(|e| PipelineError { phase: "review".to_string(), message: format!("读取角色失败: {}", e), recoverable: true })?;

    callbacks.progress("review", 0.3);

    // 3. 构建审稿 prompt
    let prompt = build_review_prompt(
        &draft.content,
        blueprint.as_ref(),
        &characters,
        review_focus,
        config,
    );

    callbacks.log("[审稿] 已构建审稿 prompt");
    callbacks.progress("review", 0.4);

    // 4. 调用 LLM（TODO: 接入实际 LLM 调用，解析 JSON）
    // 占位：返回默认审稿结果
    let review_result = ReviewResult {
        review_id: String::new(), // 将在保存后填充
        overall_score: 85.0,
        dimensions: config.review_dimensions.iter().map(|dim| ReviewDimensionResult {
            name: dim.clone(),
            score: 85.0,
            comment: format!("[占位] {} 维度评审尚未接入 LLM", dim),
        }).collect(),
        issues: vec![],
        summary: "[占位] 审稿报告尚未接入 LLM。".to_string(),
    };

    callbacks.log("[审稿] AI 审稿完成");
    callbacks.progress("review", 0.8);

    // 5. 创建 review 记录
    let review_repo = PipelineReviewRepository::new(pool.clone());
    let review_index = review_repo.get_by_draft(draft_id)
        .map(|revs| revs.len() as i32 + 1)
        .unwrap_or(1);

    let dimensions_str = serde_json::to_string(&review_result.dimensions).unwrap_or_default();
    let issues_str = serde_json::to_string(&review_result.issues).unwrap_or_default();

    let db_dimensions: Vec<crate::db::ReviewDimension> = serde_json::from_str(&dimensions_str).unwrap_or_default();
    let db_issues: Vec<crate::db::ReviewIssueItem> = serde_json::from_str(&issues_str).unwrap_or_default();

    let review = review_repo.create(
        story_id,
        draft_id,
        review_index,
        &review_result.summary,
        Some(&db_dimensions),
        Some(&db_issues),
        Some(review_result.overall_score),
        review_focus,
        None,
        None,
        None,
    ).map_err(|e| PipelineError { phase: "review".to_string(), message: format!("保存审稿报告失败: {}", e), recoverable: false })?;

    // 6. 更新 draft 状态
    draft_repo.update_status(draft_id, DraftStatus::Reviewed)
        .map_err(|e| PipelineError { phase: "review".to_string(), message: format!("更新草稿状态失败: {}", e), recoverable: false })?;

    callbacks.progress("review", 1.0);

    Ok(ReviewResult {
        review_id: review.id,
        ..review_result
    })
}

/// 构建审稿 prompt
fn build_review_prompt(
    draft_content: &str,
    blueprint: Option<&crate::db::Blueprint>,
    characters: &[crate::db::Character],
    review_focus: Option<&str>,
    config: &PipelineConfig,
) -> String {
    let mut prompt = "# 审稿专家\n\n你是一位挑剔的读者、资深编辑和小说评论家。请对以下章节进行全方位的质量评审。\n\n## 评审维度\n请对以下每个维度给出 0-100 的评分和具体评价：\n".to_string();

    for (i, dim) in config.review_dimensions.iter().enumerate() {
        let desc = match dim.as_str() {
            "continuity" => "与前文是否矛盾？伏笔是否呼应？时间线是否一致？",
            "logic" => "因果逻辑是否通顺？角色动机是否合理？是否符合世界观设定？",
            "character" => "角色能力、位置、情感状态是否与前文一致？人设是否崩塌？",
            "foreshadow" => "是否与前后章节形成有机联系？伏笔回收是否自然？",
            "pacing" => "张弛有度？是否存在拖沓或跳跃？",
            "style" => "描写是否生动？对白是否自然？画面感强不强？",
            _ => "",
        };
        prompt.push_str(&format!("{}. **{}**: {}\n", i + 1, dim, desc));
    }

    if let Some(focus) = review_focus {
        prompt.push_str(&format!("\n## 审稿侧重点\n{}\n", focus));
    }

    prompt.push_str("\n## 上下文信息\n");
    if let Some(bp) = blueprint {
        if let Some(purpose) = &bp.purpose {
            prompt.push_str(&format!("- 本章目的：{}\n", purpose));
        }
        if let Some(hook) = &bp.suspense_hook {
            prompt.push_str(&format!("- 悬念钩子：{}\n", hook));
        }
    }

    if !characters.is_empty() {
        prompt.push_str("- 出场角色：");
        let names: Vec<String> = characters.iter().map(|c| c.name.clone()).collect();
        prompt.push_str(&names.join(", "));
        prompt.push('\n');
    }

    prompt.push_str("\n## 待审稿内容\n```\n");
    prompt.push_str(draft_content);
    prompt.push_str("\n```\n\n## 输出格式（严格 JSON）\n```json\n{\n  \"overall_score\": 85,\n  \"dimensions\": [\n    {\"name\": \"剧情连贯性\", \"score\": 90, \"comment\": \"...\"}\n  ],\n  \"issues\": [\n    {\"severity\": \"high\", \"dimension\": \"角色一致性\", \"description\": \"...\", \"suggestion\": \"...\"}\n  ],\n  \"summary\": \"总体评价...\"\n}\n```\n\n注意：overall_score 是综合评分（0-100）；issues 数组可以为空；severity 只能是 critical / high / medium / low。");

    prompt
}

/// 解析审稿 JSON（容错）
pub fn parse_review_json(text: &str) -> Result<ReviewResult, PipelineError> {
    let mut clean = text.replace("```json", "").replace("```", "");
    clean = clean.trim().to_string();
    let first_brace = clean.find('{');
    let last_brace = clean.rfind('}');
    if let (Some(start), Some(end)) = (first_brace, last_brace) {
        clean = clean[start..=end].to_string();
    }
    serde_json::from_str::<ReviewResult>(&clean).map_err(|e| PipelineError {
        phase: "review".to_string(),
        message: format!("审稿报告 JSON 解析失败: {}", e),
        recoverable: true,
    })
}
