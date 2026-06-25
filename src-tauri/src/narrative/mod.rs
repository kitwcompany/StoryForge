//! Narrative Element Model — 统一叙事元素模型
//!
//! 核心理念：无论正向生成（Bootstrap/创世）还是逆向分析（拆书），
//! 操作的叙事元素是同一套抽象。
//!
//! 模块结构：
//! - elements: 统一数据模型（CharacterElement, SceneElement 等）
//! - pipeline: Pipeline trait 和通用基础设施
//! - prompts: 统一 Prompt 模板（生成/提取两用）
//! - genesis: GenesisPipeline — 正向/创世流程
//! - analysis: AnalysisPipeline — 逆向/分析流程
//! - progress: 统一进度事件系统
//! - audit: StoryStructureAuditor — 故事结构审计
//! - health: StoryHealthAnalyzer — 故事健康检查
//! - event: 叙事事件模型（LitSeg E1）
//! - thread: 叙事线索追踪模型（LitSeg E1）
//! - structure: 叙事结构定位模型（LitSeg E1）
//! - segment: 叙事感知分段模型（LitSeg E1）

pub mod analysis;
pub mod audit;
pub mod chunker;
pub mod elements;
pub mod event;
pub mod genesis;
pub mod health;
pub mod intensity_mapper;
pub mod litseg_pipeline;
pub mod pipeline;
pub mod progress;
pub mod prompts;
pub mod search;
pub mod segment;
pub mod structure;
pub mod structure_analyzer;
pub mod thread;
pub mod thread_tracker;

/// 用括号匹配从 LLM 响应中提取第一个完整的 JSON 对象。
///
/// 遍历字符，跟踪花括号深度（`{` +1, `}` -1），同时跳过字符串字面量
/// （`"..."` 内的 `{`/`}` 不计入深度）。当深度回到 0 时，即为 JSON 对象边界。
/// 这样即使 LLM 在 JSON 后输出包含 `}` 的额外文本，也不会误提取。
fn extract_first_json_object(content: &str) -> Result<&str, String> {
    let start = content
        .find('{')
        .ok_or_else(|| "No JSON object found in response".to_string())?;

    let bytes = content.as_bytes();
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut i = start;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
        } else if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Ok(&content[start..=i]);
            }
        }
        i += 1;
    }

    Err("JSON object not properly closed (unmatched braces)".to_string())
}

/// 从 LLM 响应中提取 JSON 对象，并修复常见语法错误（尾随逗号、空值、markdown
/// 围栏等）
pub fn extract_and_sanitize_json(content: &str) -> Result<String, String> {
    // 1. 基础提取：用括号匹配找第一个完整的 JSON 对象 { ... } （不使用
    //    rfind('}')，因为 LLM 可能在 JSON 后输出额外文本 其中包含 }
    //    字符，导致提取过多内容 → serde_json "trailing characters" 错误）
    let raw = extract_first_json_object(content)?;

    // 2. 移除 markdown 代码围栏标记（```json ... ```）
    let mut s = raw.to_string();
    for fence in ["```json", "```JSON", "```", "`"] {
        s = s.replace(fence, "");
    }

    // 3. 移除 UTF-8 BOM 和控制字符
    s = s.trim().to_string();
    s = s.replace('\u{feff}', "");

    // 4. 修复字符串内的未转义换行符和回车符（LLM 经常在 JSON 字符串值中直接换行）
    // 使用状态机：仅在字符串内部替换实际换行符为 \n
    {
        let mut result = String::with_capacity(s.len());
        let mut in_string = false;
        let mut escaped = false;
        for ch in s.chars() {
            if in_string {
                if escaped {
                    escaped = false;
                    result.push(ch);
                } else if ch == '\\' {
                    escaped = true;
                    result.push(ch);
                } else if ch == '"' {
                    in_string = false;
                    result.push(ch);
                } else if ch == '\n' {
                    result.push_str("\\n");
                } else if ch == '\r' {
                    // 跳过 \r，因为 \r\n 已经被处理为 \\n
                } else {
                    result.push(ch);
                }
            } else {
                if ch == '"' {
                    in_string = true;
                }
                result.push(ch);
            }
        }
        s = result;
    }

    // 5. 移除 C 风格注释（// 和 /* */）—— LLM 有时会在 JSON 中插入注释
    {
        let mut result = String::with_capacity(s.len());
        let mut in_string = false;
        let mut escaped = false;
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if in_string {
                if escaped {
                    escaped = false;
                    result.push(ch);
                } else if ch == '\\' {
                    escaped = true;
                    result.push(ch);
                } else if ch == '"' {
                    in_string = false;
                    result.push(ch);
                } else {
                    result.push(ch);
                }
            } else {
                if ch == '"' {
                    in_string = true;
                    result.push(ch);
                } else if ch == '/' && chars.peek() == Some(&'/') {
                    // 跳过单行注释
                    chars.next(); // skip second /
                    while let Some(c) = chars.next() {
                        if c == '\n' {
                            result.push('\n'); // 保留换行以保持行号
                            break;
                        }
                    }
                } else if ch == '/' && chars.peek() == Some(&'*') {
                    // 跳过多行注释
                    chars.next(); // skip *
                    while let Some(c) = chars.next() {
                        if c == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                } else {
                    result.push(ch);
                }
            }
        }
        s = result;
    }

    // 6. 修复尾随逗号：`,]` → `]` 和 `,}` → `}`
    let mut prev;
    loop {
        prev = s.clone();
        s = s.replace(",]", "]");
        s = s.replace(",}", "}");
        s = s.replace(", ]", "]");
        s = s.replace(", }", "}");
        if s == prev {
            break;
        }
    }

    // 7. 修复空值：`: ,` → `: null,`，`: ]` → `: null]`，`: }` → `: null}`
    for (bad, good) in [
        (": ,", ": null,"),
        (":,", ": null,"),
        (": ]", ": null]"),
        (": }", ": null}"),
        (":}", ": null}"),
    ] {
        s = s.replace(bad, good);
    }

    // 注意：不要替换中文引号「」『』为 ASCII 引号，这会破坏 JSON 字符串结构
    // 如果 JSON 键名或值边界使用了中文引号，那是 LLM 的格式错误，应由 LLM 修正

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_with_trailing_text_containing_braces() {
        // LLM 返回 JSON 后附带额外文本，其中包含 } 字符
        let content = r#"{
  "title": "异星末世",
  "description": "一句话简介",
  "genre": "科幻",
  "tone": "暗黑",
  "pacing": "快节奏",
  "themes": ["生存", "希望"],
  "target_length": "长篇100万字"
}

## 详细说明
故事背景设定在 {年份: 2087} 的未来世界。
角色设定包含 {name: "主角"} 等属性。
"#;
        let result = extract_and_sanitize_json(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["title"], "异星末世");
        assert_eq!(parsed["target_length"], "长篇100万字");
    }

    #[test]
    fn test_extract_json_with_markdown_fence_and_trailing() {
        let content =
            "```json\n{\"title\": \"test\", \"genre\": \"科幻\"}\n```\n\n这是额外说明文字";
        let result = extract_and_sanitize_json(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["title"], "test");
    }

    #[test]
    fn test_extract_json_brace_in_string_value() {
        // JSON 字符串值中包含 } 字符
        let content = r#"{"title": "test } end", "genre": "科幻"}额外文本}"#;
        let result = extract_and_sanitize_json(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["title"], "test } end");
    }

    #[test]
    fn test_extract_json_nested_objects() {
        let content = r#"{"meta": {"title": "test"}, "genre": "科幻"}"#;
        let result = extract_and_sanitize_json(content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["meta"]["title"], "test");
    }

    #[test]
    fn test_extract_json_no_object() {
        let result = extract_and_sanitize_json("没有 JSON 的纯文本");
        assert!(result.is_err());
    }
}

// pub use elements::*;
// pub use pipeline::*;
// pub use progress::*;
