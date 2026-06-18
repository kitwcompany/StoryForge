//! AntiAI Rewriter — LLM 改写闸骨架（v0.17.1）
//!
//! 功能：当 [`AntiAiReview`] 命中关键问题（高严重度套话/AI 句式）时，
//! 调用 LLM 在不破坏故事情节的前提下，把被点名的段落改写为更自然的中文。
//!
//! v0.17.1 阶段说明：
//! - 这是骨架实现：所有 public API 已就绪，但默认行为是「直接返回原文」。
//! - v0.17.2 将接入实际 LLM 调用（走 router → CreativeWriting tier）。
//! - 设计上避免与现有 `agents/orchestrator.rs` Rewrite step 冲突：
//!   本闸只在「事后审查」阶段触发（AntiAiReview overall_score < threshold），
//!   而非主创作流程的内联 Rewrite 循环。
//!
//! 不接入生产：本模块当前不被任何业务路径调用，仅做接口预定义。

use serde::{Deserialize, Serialize};

use super::{AntiAiReview, ReviewIssue};

/// 改写策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewriteStrategy {
    /// 仅替换被命中的成语/套话/AI 句式，最大限度保留原文
    LocalReplace,
    /// 段落级重写：把整段重写为更自然的口吻
    ParagraphRewrite,
    /// 章节级重写：跨段重新编排（最重的手术）
    ChapterRewrite,
}

impl Default for RewriteStrategy {
    fn default() -> Self {
        Self::LocalReplace
    }
}

/// 改写请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteRequest {
    pub original_content: String,
    pub review: AntiAiReview,
    pub strategy: RewriteStrategy,
    /// 改写预算（字符数）。0 表示不限。
    pub budget_chars: usize,
}

/// 改写结果（含 diff 回流字段，便于前端展示前后对比）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteOutcome {
    pub rewritten_content: String,
    /// 是否真正发生改写。骨架阶段恒为 false。
    pub mutated: bool,
    /// 命中并被替换的具体片段（diff 回流，旧 → 新）。
    pub diffs: Vec<RewriteDiff>,
    /// 用于 UI 提示的简短理由。
    pub rationale: String,
}

/// 单个改写片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteDiff {
    pub before: String,
    pub after: String,
    pub reason: String,
}

/// 改写闸：根据五维审查结果触发 LLM 改写。
///
/// v0.17.1 骨架：直接返回原文 + 空 diff。
pub struct AntiAiRewriter;

impl AntiAiRewriter {
    pub fn new() -> Self {
        Self
    }

    /// 是否应该触发改写。判定依据：
    /// - overall_score < 60，或
    /// - 存在任一 high severity issue。
    pub fn should_trigger(&self, review: &AntiAiReview) -> bool {
        if review.overall_score < 60.0 {
            return true;
        }
        review.issues.iter().any(|i| i.severity == "high")
    }

    /// 主入口：异步改写。骨架阶段不调用 LLM。
    pub async fn rewrite(&self, request: RewriteRequest) -> Result<RewriteOutcome, String> {
        // v0.17.1: 骨架阶段——立即返回原文，标记未改写。
        // v0.17.2 将在此处调用 LLM，并填充 diffs。
        let _ = request.strategy;
        let _ = request.budget_chars;
        let _high_issues: Vec<&ReviewIssue> = request
            .review
            .issues
            .iter()
            .filter(|i| i.severity == "high")
            .collect();

        Ok(RewriteOutcome {
            rewritten_content: request.original_content,
            mutated: false,
            diffs: Vec::new(),
            rationale: "v0.17.1 骨架：未启用 LLM 改写".to_string(),
        })
    }
}

impl Default for AntiAiRewriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anti_ai::DimensionScore;

    fn empty_review(score: f64) -> AntiAiReview {
        AntiAiReview {
            overall_score: score,
            dimensions: vec![DimensionScore {
                name: "vocabulary".into(),
                score,
                weight: 1.0,
                description: "test".into(),
            }],
            issues: Vec::new(),
            suggestions: Vec::new(),
            flagged_passages: Vec::new(),
        }
    }

    fn with_high_issue() -> AntiAiReview {
        let mut r = empty_review(80.0);
        r.issues.push(ReviewIssue {
            dimension: "vocabulary".into(),
            severity: "high".into(),
            description: "套话过多".into(),
            example: "不是X而是Y".into(),
            suggestion: "改用具体动作描写".into(),
        });
        r
    }

    #[test]
    fn should_trigger_low_score() {
        let r = empty_review(40.0);
        assert!(AntiAiRewriter::new().should_trigger(&r));
    }

    #[test]
    fn should_not_trigger_clean_review() {
        let r = empty_review(85.0);
        assert!(!AntiAiRewriter::new().should_trigger(&r));
    }

    #[test]
    fn should_trigger_high_severity_even_with_high_score() {
        let r = with_high_issue();
        assert!(AntiAiRewriter::new().should_trigger(&r));
    }

    #[tokio::test]
    async fn skeleton_returns_original_content_unchanged() {
        let rewriter = AntiAiRewriter::new();
        let req = RewriteRequest {
            original_content: "测试段落。".to_string(),
            review: with_high_issue(),
            strategy: RewriteStrategy::LocalReplace,
            budget_chars: 0,
        };
        let outcome = rewriter.rewrite(req).await.unwrap();
        assert_eq!(outcome.rewritten_content, "测试段落。");
        assert!(!outcome.mutated);
        assert!(outcome.diffs.is_empty());
    }
}
