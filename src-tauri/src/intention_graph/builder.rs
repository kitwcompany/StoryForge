//! 意图合成流水线
//!
//! 三阶段合成：Query Synthesis → Chain Expansion → Atomic Intention Extraction
//! 将用户自然语言输入转化为标准化的原子意图节点。

use crate::error::AppError;
use crate::intention_graph::IntentContext;
use crate::llm::LlmService;

use super::models::*;

/// 意图合成流水线
pub struct IntentSynthesisPipeline {
    llm_service: LlmService,
}

impl IntentSynthesisPipeline {
    pub fn new(llm_service: LlmService) -> Self {
        Self { llm_service }
    }

    /// 阶段一：Query Synthesis
    /// 将用户输入合成为结构化的查询意图
    pub fn synthesize_query(
        &self,
        user_input: &str,
        _context: &IntentContext,
    ) -> SynthesizedQuery {
        // 基于关键词和上下文的快速合成（无需 LLM）
        let normalized = user_input.to_lowercase();

        // 检测明确的创作意图
        let is_prose_request = [
            "写", "write", "创作", "开始写", "写小说", "写故事",
            "写一章", "写开篇", "写正文", "start writing",
            "write a novel", "write a story", "write chapter", "begin writing",
        ]
        .iter()
        .any(|kw| normalized.contains(kw));

        let is_style_request = [
            "风格", "style", "文风", "润色", "polish", "enhance style",
            "改风格", "调整风格",
        ]
        .iter()
        .any(|kw| normalized.contains(kw));

        let is_character_request = [
            "角色", "character", "人物", "person",
            "创建角色", "create character", "修改角色", "update character",
        ]
        .iter()
        .any(|kw| normalized.contains(kw));

        let is_world_request = [
            "世界观", "world", "设定", "setting",
            "修改世界观", "update world", "世界规则",
        ]
        .iter()
        .any(|kw| normalized.contains(kw));

        let is_outline_request = [
            "大纲", "outline", "规划", "plan",
            "生成大纲", "create outline", "故事结构",
        ]
        .iter()
        .any(|kw| normalized.contains(kw));

        let primary_intent = if is_prose_request {
            "generate prose"
        } else if is_style_request {
            "enhance style"
        } else if is_character_request {
            "manage character"
        } else if is_world_request {
            "manage world building"
        } else if is_outline_request {
            "plan structure"
        } else {
            "analyze intent"
        };

        SynthesizedQuery {
            raw_input: user_input.to_string(),
            primary_intent: primary_intent.to_string(),
            confidence: if is_prose_request || is_style_request || is_character_request {
                0.9
            } else {
                0.6
            },
            detected_keywords: vec![],
            context_hints: vec![],
        }
    }

    /// 阶段二：Chain Expansion
    /// 将主意图扩展为意图链（基于历史执行图的共现模式）
    pub fn expand_chain(&self, query: &SynthesizedQuery) -> Vec<String> {
        // 基于主意图的预设链扩展规则
        // 实际实现中会查询 asset_asset_edges 的 tool_next 关系
        match query.primary_intent.as_str() {
            "generate prose" => vec![
                "generate prose".to_string(),
                "inspect quality".to_string(),
                "revise content".to_string(),
            ],
            "enhance style" => vec![
                "enhance style".to_string(),
                "apply style dna".to_string(),
                "inspect quality".to_string(),
            ],
            "manage character" => vec![
                "manage character".to_string(),
                "update knowledge graph".to_string(),
            ],
            "manage world building" => vec![
                "manage world building".to_string(),
                "update knowledge graph".to_string(),
            ],
            "plan structure" => vec![
                "plan structure".to_string(),
                "generate outline".to_string(),
            ],
            _ => vec![query.primary_intent.clone()],
        }
    }

    /// 阶段三：Atomic Intention Extraction
    /// 将意图链分解为原子化的动词-宾语节点
    pub fn extract_atomic_intentions(&self, chain: &[String]) -> Vec<IntentionNode> {
        let mut nodes = Vec::new();

        for intent_str in chain {
            let parts: Vec<&str> = intent_str.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let verb = parts[0];
                let object = parts[1];
                let description = format!("{} {}", verb, object);
                nodes.push(IntentionNode::atomic(verb, object, &description));
            }
        }

        nodes
    }

    /// 完整三阶段合成（未来接入 LLM 增强）
    pub fn synthesize_full(
        &self,
        user_input: &str,
        context: &IntentContext,
    ) -> Result<IntentSynthesisResult, AppError> {
        // Phase 1: Query Synthesis (rule-based for now)
        let query = self.synthesize_query(user_input, context);

        // Phase 2: Chain Expansion (rule-based for now)
        let chain = self.expand_chain(&query);

        // Phase 3: Atomic Extraction
        let sub_intentions = self.extract_atomic_intentions(&chain);

        let root_intention = if let Some(first) = sub_intentions.first() {
            first.clone()
        } else {
            IntentionNode::atomic("analyze", "intent", "analyze user intent")
        };

        Ok(IntentSynthesisResult {
            root_intention,
            sub_intentions,
            confidence: query.confidence,
            chain_expansion: chain,
        })
    }
}

/// 合成后的查询结构
#[derive(Debug, Clone)]
pub struct SynthesizedQuery {
    pub raw_input: String,
    pub primary_intent: String,
    pub confidence: f64,
    pub detected_keywords: Vec<String>,
    pub context_hints: Vec<String>,
}
