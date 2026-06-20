//! 意图上下文管理
//!
//! 维护用户意图的会话级上下文，支持意图链追踪和历史意图记忆。

use std::collections::VecDeque;

use super::models::*;

/// 意图上下文：单次请求/会话的意图状态
#[derive(Debug, Clone)]
pub struct IntentContext {
    /// 当前故事 ID
    pub story_id: Option<String>,
    /// 当前章节号
    pub chapter_number: i32,
    /// 用户输入历史（最近 N 条）
    pub input_history: VecDeque<String>,
    /// 已识别的意图链
    pub intention_chain: Vec<IntentionNode>,
    /// 已执行的资产
    pub executed_assets: Vec<String>,
    /// 会话级参数（如风格权重、温度等）
    pub session_params: serde_json::Value,
    /// 最大历史长度
    max_history: usize,
}

impl IntentContext {
    pub fn new() -> Self {
        Self {
            story_id: None,
            chapter_number: 1,
            input_history: VecDeque::with_capacity(10),
            intention_chain: Vec::new(),
            executed_assets: Vec::new(),
            session_params: serde_json::json!({}),
            max_history: 10,
        }
    }

    pub fn with_story_id(mut self, story_id: String) -> Self {
        self.story_id = Some(story_id);
        self
    }

    pub fn with_chapter_number(mut self, chapter_number: i32) -> Self {
        self.chapter_number = chapter_number;
        self
    }

    /// 添加用户输入到历史
    pub fn add_input(&mut self, input: String) {
        self.input_history.push_back(input);
        if self.input_history.len() > self.max_history {
            self.input_history.pop_front();
        }
    }

    /// 添加意图到链
    pub fn add_intention(&mut self, intention: IntentionNode) {
        self.intention_chain.push(intention);
    }

    /// 标记资产已执行
    pub fn mark_executed(&mut self, asset_id: String) {
        self.executed_assets.push(asset_id);
    }

    /// 获取意图链的文本表示（用于 LLM 提示词）
    pub fn intention_chain_text(&self) -> String {
        self.intention_chain
            .iter()
            .map(|i| i.canonical_text())
            .collect::<Vec<_>>()
            .join(" → ")
    }

    /// 获取已执行资产的文本表示
    pub fn executed_assets_text(&self) -> String {
        self.executed_assets.join(", ")
    }

    /// 设置会话参数
    pub fn set_param<T: serde::Serialize>(&mut self, key: &str, value: T) {
        if let serde_json::Value::Object(ref mut map) = self.session_params {
            if let Ok(v) = serde_json::to_value(value) {
                map.insert(key.to_string(), v);
            }
        }
    }

    /// 获取会话参数
    pub fn get_param(&self, key: &str) -> Option<&serde_json::Value> {
        if let serde_json::Value::Object(ref map) = self.session_params {
            map.get(key)
        } else {
            None
        }
    }
}

impl Default for IntentContext {
    fn default() -> Self {
        Self::new()
    }
}
