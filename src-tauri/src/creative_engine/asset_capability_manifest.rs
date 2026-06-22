//! 运行时创作资产能力清单
//!
//! 在应用启动时，把 strategy::load_all_assets 得到的全部 SelectableAsset
//! 组装成一份紧凑的文本摘要，供 PromptSynthesizer（TriShot Call 1）和
//! ModelGateway 调度参考。清单在每次启动时重新生成，因此新增/修改的技能、
//! 体裁画像、工作流等都能自动反映。

use std::sync::Arc;

use crate::{
    db::{DbPool, GenreProfileRepository},
    skills::Skill,
    strategy::{load_all_assets, models::SelectableAsset},
};

/// 运行时创作资产能力清单
#[derive(Debug, Default)]
pub struct AssetCapabilityManifest {
    /// 原始资产列表（保留给代码查询）
    pub assets: Vec<SelectableAsset>,
    /// 注入 LLM prompt 的紧凑文本摘要
    pub compact_summary: String,
}

impl AssetCapabilityManifest {
    /// 从数据库和技能管理器构建清单
    pub fn build_from(
        repo: &GenreProfileRepository,
        skills: &[Skill],
    ) -> Result<Self, crate::error::AppError> {
        let assets = load_all_assets(repo, skills)?;
        let compact_summary = build_compact_summary(&assets, 6000);
        Ok(Self {
            assets,
            compact_summary,
        })
    }

    /// 从 DbPool 构建（启动路径常用）
    pub fn from_pool(pool: DbPool, skills: &[Skill]) -> Result<Self, crate::error::AppError> {
        let repo = GenreProfileRepository::new(pool);
        Self::build_from(&repo, skills)
    }

    /// 获取摘要文本
    pub fn summary(&self) -> &str {
        &self.compact_summary
    }

    /// 按 ID 查找资产
    pub fn find(&self, id: &str) -> Option<&SelectableAsset> {
        self.assets.iter().find(|a| a.id == id)
    }

    /// 把选中的资产 ID 展开成适合传给 GatewayRequest.asset_tags 的标签集合
    pub fn tags_for_selected(&self, selected_ids: &[String]) -> Vec<String> {
        let mut tags: Vec<String> = Vec::new();
        for id in selected_ids {
            if let Some(asset) = self.find(id) {
                // 短 id（如 snowflake）和 kind（如 methodology）都作为 tag
                let short = id.rsplit_once('.').map(|(_, s)| s).unwrap_or(id);
                tags.push(short.to_string());
                tags.push(asset.kind.to_string());
            } else {
                // 即使找不到资产，也把短 id 当 tag 透传
                let short = id.rsplit_once('.').map(|(_, s)| s).unwrap_or(id);
                tags.push(short.to_string());
            }
        }
        tags.sort_unstable();
        tags.dedup();
        tags
    }
}

/// 把资产列表渲染成紧凑分组文本
fn build_compact_summary(assets: &[SelectableAsset], max_chars: usize) -> String {
    use crate::strategy::models::AssetKind;
    let mut sections: Vec<String> = Vec::new();
    let mut kinds: Vec<AssetKind> = assets.iter().map(|a| a.kind).collect();
    kinds.sort_unstable_by_key(|k| format!("{}", k));
    kinds.dedup();

    for kind in kinds {
        let group: Vec<&SelectableAsset> = assets.iter().filter(|a| a.kind == kind).collect();
        if group.is_empty() {
            continue;
        }
        let kind_name = format!("{}", kind);
        let mut lines = vec![format!("【{}】", kind_name)];
        for asset in group {
            lines.push(format!(
                "- {} ({}): {} [何时使用: {}]",
                asset.id, asset.name, asset.description, asset.when_to_use
            ));
        }
        sections.push(lines.join("\n"));
    }

    let joined = sections.join("\n\n");
    if joined.chars().count() > max_chars {
        let truncated: String = joined.chars().take(max_chars).collect();
        format!("{}\n…（资产清单已截断，共 {} 项）", truncated, assets.len())
    } else {
        format!("{}\n（共 {} 项创作资产）", joined, assets.len())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::strategy::models::{AssetKind, SelectableAsset};

    fn dummy_asset(
        id: &str,
        kind: AssetKind,
        name: &str,
        desc: &str,
        when: &str,
    ) -> SelectableAsset {
        SelectableAsset {
            id: id.to_string(),
            kind,
            name: name.to_string(),
            description: desc.to_string(),
            when_to_use: when.to_string(),
            input_description: None,
            output_description: None,
            payload: serde_json::Value::Null,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_build_compact_summary_groups_by_kind() {
        let assets = vec![
            dummy_asset(
                "methodology.snowflake",
                AssetKind::Methodology,
                "雪花法",
                "自顶向下扩展",
                "从概念开始",
            ),
            dummy_asset(
                "beat_card.reversal",
                AssetKind::BeatCard,
                "反转桥段",
                "制造反转",
                "需要反转时",
            ),
        ];
        let summary = build_compact_summary(&assets, 6000);
        assert!(summary.contains("【methodology】"));
        assert!(summary.contains("【beat_card】"));
        assert!(summary.contains("methodology.snowflake"));
        assert!(summary.contains("共 2 项创作资产"));
    }

    #[test]
    fn test_tags_for_selected_expands_kind_and_short_id() {
        let manifest = AssetCapabilityManifest {
            assets: vec![dummy_asset(
                "methodology.snowflake",
                AssetKind::Methodology,
                "雪花法",
                "",
                "",
            )],
            compact_summary: String::new(),
        };
        let tags = manifest.tags_for_selected(&["methodology.snowflake".to_string()]);
        assert!(tags.contains(&"snowflake".to_string()));
        assert!(tags.contains(&"methodology".to_string()));
    }
}
