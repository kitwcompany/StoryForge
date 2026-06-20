#![allow(dead_code)]
//! Prompt Template Engine - 提示词模板引擎
//!
//! v0.21.0: 清理死代码——删除未被调用的 PromptTemplate/PromptLibrary。
//! 仅保留 TemplateEngine（被 registry 和各模块的模板渲染使用）。

use std::collections::HashMap;

/// 模板引擎
pub struct TemplateEngine;

impl TemplateEngine {
    /// 渲染模板，替换 {{key}} 为对应值
    pub fn render(template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        // 简单变量替换: {{key}}
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // 清理未替换的变量（保留原样或替换为空）
        // 这里选择保留原样，以便调试

        result
    }

    /// 条件渲染: {{#if key}}...{{/if}}
    pub fn render_with_conditions(template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        // 处理条件块
        loop {
            let start_tag = result.find("{{#if ");
            if start_tag.is_none() {
                break;
            }
            let start = start_tag.unwrap();
            let cond_end = result[start..].find("}}").unwrap() + start;
            let condition_key = result[start + 6..cond_end].trim();

            let end_tag = result[cond_end..].find("{{/if}}").unwrap() + cond_end;
            let block_content = result[cond_end + 2..end_tag].to_string();

            let has_value = variables
                .get(condition_key)
                .map(|v| !v.is_empty() && v != "无" && v != "暂无" && v != "暂无角色信息")
                .unwrap_or(false);

            let replacement = if has_value {
                block_content
            } else {
                String::new()
            };

            result.replace_range(start..end_tag + 7, &replacement);
        }

        // 然后处理普通变量
        Self::render(&result, variables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_render() {
        let template = "Hello, {{name}}!";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        assert_eq!(TemplateEngine::render(template, &vars), "Hello, World!");
    }

    #[test]
    fn test_conditional_render() {
        let template = "{{#if has_data}}Data: {{data}}{{/if}}End";
        let mut vars = HashMap::new();
        vars.insert("has_data".to_string(), "yes".to_string());
        vars.insert("data".to_string(), "123".to_string());
        assert_eq!(
            TemplateEngine::render_with_conditions(template, &vars),
            "Data: 123End"
        );
    }

    #[test]
    fn test_conditional_skip() {
        let template = "{{#if missing}}Data: {{data}}{{/if}}End";
        let mut vars = HashMap::new();
        vars.insert("missing".to_string(), "".to_string());
        assert_eq!(
            TemplateEngine::render_with_conditions(template, &vars),
            "End"
        );
    }
}
