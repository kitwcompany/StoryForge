//! v0.19.0 PromptRegistry —— 全局提示词注册表（全面可配置化）
//!
//! 所有内置 LLM 提示词集中注册于此，支持用户在前端覆盖。
//! 设计原则：
//! - 每个提示词有唯一稳定 ID
//! - 分类清晰，便于前端展示
//! - 支持模板变量（{{variable}}）
//! - 运行时优先读取 prompt_overrides 表中的用户自定义版本

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{db::DbPool, error::AppError};

// ─────────────────────────────────────────────────────────────
// 分类枚举
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PromptCategory {
    // 核心创作
    Writer,      // 写作核心提示词
    Inspector,   // 质检与审校
    Commentator, // 古典评点
    // 规划与分析
    Planner,     // 大纲规划
    Analyzer,    // 情节/结构分析
    // 系统与探测
    Probe,       // 模型探测/基准
    System,      // 系统级提示词
    // 记忆与知识
    Memory,      // 记忆压缩/蒸馏
    Knowledge,   // 知识图谱相关
    // 技能与工具
    Skill,       // 内置技能提示词
    // 创作方法论
    Methodology, // 雪花法/英雄之旅等
    // 世界与角色
    World,       // 世界观/场景
    Character,   // 角色相关
    // 叙事与结构
    Narrative,   // 叙事结构/事件提取
    // 其他
    Other,
}

impl PromptCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Writer => "写作核心",
            Self::Inspector => "质检与审校",
            Self::Commentator => "古典评点",
            Self::Planner => "大纲规划",
            Self::Analyzer => "分析",
            Self::Probe => "探测与基准",
            Self::System => "系统",
            Self::Memory => "记忆",
            Self::Knowledge => "知识",
            Self::Skill => "技能",
            Self::Methodology => "创作方法论",
            Self::World => "世界观与场景",
            Self::Character => "角色",
            Self::Narrative => "叙事结构",
            Self::Other => "其他",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Writer => "AI 写作助手的核心角色设定与行为准则",
            Self::Inspector => "内容质量检查、逻辑连贯性、人物一致性审校",
            Self::Commentator => "以金圣叹风格对小说段落进行实时文学点评",
            Self::Planner => "故事大纲设计、章节结构规划",
            Self::Analyzer => "情节复杂度分析、结构评估",
            Self::Probe => "模型可用性探测、性能基准测试",
            Self::System => "系统级通用提示词",
            Self::Memory => "记忆压缩、摘要生成",
            Self::Knowledge => "知识图谱蒸馏、实体关系提取",
            Self::Skill => "内置技能（文风增强、情节反转等）",
            Self::Methodology => "雪花法、英雄之旅、场景结构等创作方法论",
            Self::World => "世界观构建、场景设计",
            Self::Character => "角色塑造、声音一致性",
            Self::Narrative => "叙事事件提取、结构分析",
            Self::Other => "其他辅助提示词",
        }
    }

    pub fn order(&self) -> u8 {
        match self {
            Self::Writer => 0,
            Self::Inspector => 1,
            Self::Commentator => 2,
            Self::Planner => 3,
            Self::Analyzer => 4,
            Self::World => 5,
            Self::Character => 6,
            Self::Narrative => 7,
            Self::Methodology => 8,
            Self::Skill => 9,
            Self::Memory => 10,
            Self::Knowledge => 11,
            Self::Probe => 12,
            Self::System => 13,
            Self::Other => 14,
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 数据结构
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: PromptCategory,
    pub default_content: String,
    pub current_content: String,
    pub is_overridden: bool,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOverride {
    pub prompt_id: String,
    pub content: String,
}

// ─────────────────────────────────────────────────────────────
// 内置提示词注册表
// ─────────────────────────────────────────────────────────────

static BUILTIN_PROMPTS: std::sync::OnceLock<HashMap<String, PromptEntry>> =
    std::sync::OnceLock::new();

fn init_builtin_prompts() -> HashMap<String, PromptEntry> {
    let mut m = HashMap::new();

    // ═══════════════════════════════════════════════════════
    // 1. 写作核心 (Writer)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "writer_system".to_string(),
        PromptEntry {
            id: "writer_system".to_string(),
            name: "Writer 系统提示词".to_string(),
            description: "AI 写作助手的基础角色设定与行为准则".to_string(),
            category: PromptCategory::Writer,
            default_content: r#"你是一位专业的小说创作助手，擅长中文写作。

你的任务是根据提供的故事上下文和指令，续写或改写小说内容。

核心要求：
1. 使用中文（简体中文）写作
2. 保持角色声音一致性——每个角色的用词习惯、语气、句式结构符合其性格
3. 展示而非讲述——用动作、对话、细节描写传达情感，避免直接陈述
4. 对话必须推动情节或揭示性格，禁止无意义闲聊
5. 每个场景结尾留下钩子（悬念、新问题、新威胁）
6. 遵循提供的世界观规则和设定约束
7. 保持与已有情节的连贯性，不引入与设定矛盾的新元素

写作风格：
- 根据指定的题材和基调调整语言风格
- 环境描写服务于氛围营造，不过度铺陈
- 内心独白适度，主要用于揭示角色动机和冲突
- 节奏控制：紧张场景用短句、快节奏；抒情场景允许长句和细腻描写

输出要求：
- 只输出小说正文，不要添加解释、总结或元评论
- 不要输出"以下是续写内容"等过渡语
- 保持与已有文本的自然衔接"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "story_title".to_string(),
                "genre".to_string(),
                "tone".to_string(),
                "pacing".to_string(),
                "characters".to_string(),
                "previous_chapters".to_string(),
                "narrative_structure".to_string(),
                "current_content".to_string(),
                "instruction".to_string(),
                "world_rules".to_string(),
                "scene_structure".to_string(),
                "outline_context".to_string(),
                "story_description".to_string(),
            ],
        },
    );

    m.insert(
        "writer_continue".to_string(),
        PromptEntry {
            id: "writer_continue".to_string(),
            name: "续写用户提示词".to_string(),
            description: "Writer 续写模式的用户提示词模板".to_string(),
            category: PromptCategory::Writer,
            default_content: r#"【作品】{{story_title}}
【题材】{{genre}}
【基调】{{tone}}
【节奏】{{pacing}}

【角色】
{{characters}}

【前文摘要】
{{previous_chapters}}

{{#if narrative_structure}}
【叙事结构】
{{narrative_structure}}
{{/if}}

{{#if world_rules}}
【世界观规则】
{{world_rules}}
{{/if}}

{{#if scene_structure}}
【场景结构】
{{scene_structure}}
{{/if}}

{{#if outline_context}}
【大纲要求】
{{outline_context}}
{{/if}}

【当前内容】
{{current_content}}

【指令】
{{instruction}}

请根据以上上下文续写小说内容。保持与已有文本的风格和节奏一致，自然衔接。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "story_title".to_string(),
                "genre".to_string(),
                "tone".to_string(),
                "pacing".to_string(),
                "characters".to_string(),
                "previous_chapters".to_string(),
                "narrative_structure".to_string(),
                "current_content".to_string(),
                "instruction".to_string(),
                "world_rules".to_string(),
                "scene_structure".to_string(),
                "outline_context".to_string(),
            ],
        },
    );

    m.insert(
        "writer_rewrite".to_string(),
        PromptEntry {
            id: "writer_rewrite".to_string(),
            name: "改写用户提示词".to_string(),
            description: "Writer 改写选中内容的用户提示词模板".to_string(),
            category: PromptCategory::Writer,
            default_content: r#"【作品】{{story_title}}
【题材】{{genre}}
【基调】{{tone}}
【节奏】{{pacing}}

【角色】
{{characters}}

【前文摘要】
{{previous_chapters}}

{{#if world_rules}}
【世界观规则】
{{world_rules}}
{{/if}}

【当前内容】
{{current_content}}

【选中内容】
{{selected_text}}

【指令】
{{instruction}}

请根据指令改写上述【选中内容】，保持与上下文的风格一致。只输出改写后的内容，不要输出未选中的部分。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "story_title".to_string(),
                "genre".to_string(),
                "tone".to_string(),
                "pacing".to_string(),
                "characters".to_string(),
                "previous_chapters".to_string(),
                "current_content".to_string(),
                "selected_text".to_string(),
                "instruction".to_string(),
                "world_rules".to_string(),
            ],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 2. 质检与审校 (Inspector)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "inspector_system".to_string(),
        PromptEntry {
            id: "inspector_system".to_string(),
            name: "Inspector 系统提示词".to_string(),
            description: "质检员的角色设定与检查准则".to_string(),
            category: PromptCategory::Inspector,
            default_content: r#"你是一位资深小说编辑和文学质检专家。你的任务是对小说内容进行全面的质量检查。

检查维度：
1. 连续性：时间线、事件顺序、因果关系是否一致
2. 人物一致性：角色行为是否符合其性格设定、目标、动机
3. 世界观一致性：内容是否违反已建立的世界规则
4. 风格一致性：语言风格是否与整体基调匹配
5. 伏笔推进：已埋设的伏笔是否得到推进或回收
6. 逻辑合理性：情节发展是否符合内在逻辑
7. 对话质量：对话是否自然、有区分度、推动情节
8. 描写质量：是否展示而非讲述，细节是否生动

评分标准（0.0-1.0）：
- 0.9-1.0：优秀，几乎无需修改
- 0.7-0.89：良好，有小问题需要调整
- 0.5-0.69：合格，有明显问题需要修改
- 0.0-0.49：不合格，需要大幅重写

输出格式：
请先给出总体评分（0.0-1.0），然后列出具体问题（如有），每条问题包含：
- 问题类型（连续性/人物/世界观/风格/伏笔/逻辑/对话/描写）
- 问题描述
- 修改建议
- 严重程度（严重/警告/提示）

【作品信息】
标题: {{story_title}}
题材: {{genre}}
角色: {{characters}}"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "story_title".to_string(),
                "genre".to_string(),
                "characters".to_string(),
                "content".to_string(),
            ],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 3. 古典评点 (Commentator)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "commentator_system".to_string(),
        PromptEntry {
            id: "commentator_system".to_string(),
            name: "评点家系统提示词".to_string(),
            description: "以金圣叹风格进行古典文学评点".to_string(),
            category: PromptCategory::Commentator,
            default_content: r#"你是一位精通中国古典文学的评点家，风格效仿金圣叹。你以犀利、独到、富有洞见的视角点评小说文本。

评点风格：
1. 语言：使用典雅的文言白话 hybrid，偶尔引用古典诗词
2. 视角：既站在读者角度谈感受，也站在作者角度谈技法
3. 重点：关注结构安排、伏笔埋设、人物刻画、语言精妙之处
4. 语气：可以赞叹、可以批评、可以调侃，但必有见地
5. 格式：每条评点以「※」开头，后接评点内容

评点维度：
- 结构：此段在整体布局中的功能
- 人物：角色心理、动机、性格的刻画
- 语言：用词、句式、修辞的精妙或不足
- 伏笔：此处是否埋设或回收了伏笔
- 张力：冲突、悬念、情感的强度
- 节奏：叙事速度的把控

【作品】{{story_title}}
【题材】{{genre}}

【待评点文本】
{{text}}

请对以上文本进行评点，输出 3-5 条评点，每条以「※」开头。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "story_title".to_string(),
                "genre".to_string(),
                "text".to_string(),
            ],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 4. 大纲规划 (Planner)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "outline_planner".to_string(),
        PromptEntry {
            id: "outline_planner".to_string(),
            name: "大纲规划师提示词".to_string(),
            description: "设计故事大纲和章节结构".to_string(),
            category: PromptCategory::Planner,
            default_content: r#"你是一位专业的大纲规划师，擅长设计故事结构和章节布局。

【故事前提】
{{premise}}

【角色】
{{characters}}

请设计一个完整的故事大纲，包含：
1. 三幕式结构概述
2. 每幕的关键情节点
3. 章节划分（每章包含：标题、核心事件、情感基调、字数预估）
4. 主要伏笔的埋设和回收位置
5. 角色弧线规划

输出要求：
- 使用 Markdown 格式
- 结构清晰，层次分明
- 情节点之间要有明确的因果关系
- 考虑节奏变化：紧张与松弛交替"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "premise".to_string(),
                "characters".to_string(),
            ],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 5. 分析 (Analyzer)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "plot_analysis".to_string(),
        PromptEntry {
            id: "plot_analysis".to_string(),
            name: "情节分析提示词".to_string(),
            description: "分析情节复杂度、检测漏洞".to_string(),
            category: PromptCategory::Analyzer,
            default_content: r#"【故事内容】
{{content}}

【分析要求】
1. 情节复杂度评估（简单/中等/复杂）
2. 主要情节线索梳理
3. 潜在的逻辑漏洞
4. 伏笔和回收情况
5. 高潮设置是否合理
6. 改进建议"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec!["content".to_string()],
        },
    );

    m.insert(
        "character_analysis".to_string(),
        PromptEntry {
            id: "character_analysis".to_string(),
            name: "角色分析提示词".to_string(),
            description: "分析角色一致性和发展建议".to_string(),
            category: PromptCategory::Analyzer,
            default_content: r#"你是一位角色发展专家。分析角色一致性并建议特质更新。

【角色】{{character_name}}
【背景】{{character_background}}
【当前特质】{{current_traits}}

【章节内容】
{{chapter_content}}

请：
1. 识别任何新揭示的性格特质
2. 注意与已建立角色形象的矛盾
3. 建议动态特质更新（置信度 0.0-1.0）

以 JSON 格式输出特质数组。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "character_name".to_string(),
                "character_background".to_string(),
                "current_traits".to_string(),
                "chapter_content".to_string(),
            ],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 6. 探测与基准 (Probe)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "model_gateway_probe".to_string(),
        PromptEntry {
            id: "model_gateway_probe".to_string(),
            name: "模型探测提示词".to_string(),
            description: "检测模型是否正常运行的测试用语".to_string(),
            category: PromptCategory::Probe,
            default_content: "Respond with exactly the word OK.".to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "benchmark_short".to_string(),
        PromptEntry {
            id: "benchmark_short".to_string(),
            name: "短任务基准提示词".to_string(),
            description: "流式基准测试短任务（低 token）".to_string(),
            category: PromptCategory::Probe,
            default_content: "用一句话总结'人工智能'这个概念。".to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "benchmark_long".to_string(),
        PromptEntry {
            id: "benchmark_long".to_string(),
            name: "长任务基准提示词".to_string(),
            description: "流式基准测试长任务（高 token）".to_string(),
            category: PromptCategory::Probe,
            default_content: "请详细描述一个未来城市的一天，包括交通、工作、娱乐、社交等方面，不少于 500 字。".to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 7. 记忆 (Memory)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "memory_compressor".to_string(),
        PromptEntry {
            id: "memory_compressor".to_string(),
            name: "记忆压缩提示词".to_string(),
            description: "将小说内容压缩为高层摘要".to_string(),
            category: PromptCategory::Memory,
            default_content: r#"你是一位专业的文学记忆压缩师。请将以下小说相关内容压缩为简洁的高层摘要。

【作品信息】
标题: {{story_title}}
题材: {{genre}}
文风: {{tone}}
节奏: {{pacing}}

【待压缩内容】
{{content}}

【压缩要求】
1. 保留核心情节、人物关系、关键伏笔
2. 删除细节描写、重复叙述、过渡段落
3. 输出长度控制在原文的 {{ratio}}%
4. 使用第三人称客观叙述

请直接输出压缩后的摘要，不要添加解释。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "story_title".to_string(),
                "genre".to_string(),
                "tone".to_string(),
                "pacing".to_string(),
                "content".to_string(),
                "ratio".to_string(),
            ],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 8. 知识 (Knowledge)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "knowledge_distiller".to_string(),
        PromptEntry {
            id: "knowledge_distiller".to_string(),
            name: "知识蒸馏提示词".to_string(),
            description: "从知识图谱提炼高层摘要".to_string(),
            category: PromptCategory::Knowledge,
            default_content: r#"你是一位专业的文学知识蒸馏师。请根据以下小说知识图谱，提炼出高层摘要。

【作品信息】
标题: {{story_title}}
题材: {{genre}}
文风: {{tone}}
节奏: {{pacing}}

【知识图谱】
{{content}}

【蒸馏要求】
1. 世界观概述：提炼故事的宏观设定、核心规则、时代背景
2. 主要势力：总结故事中的重要组织、阵营、群体及其关系
3. 人物关系网：梳理核心角色之间的关系、立场、冲突
4. 核心情节线：提炼当前已展开的主要悬念、伏笔、目标
5. 输出条理清晰，使用Markdown格式，总长度控制在800字以内

请直接输出蒸馏后的摘要。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "story_title".to_string(),
                "genre".to_string(),
                "tone".to_string(),
                "pacing".to_string(),
                "content".to_string(),
            ],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 9. 技能 (Skill)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "skill_style_enhancer".to_string(),
        PromptEntry {
            id: "skill_style_enhancer".to_string(),
            name: "文风增强器提示词".to_string(),
            description: "增强文本的文学性和表现力".to_string(),
            category: PromptCategory::Skill,
            default_content: "你是一个专业的文学编辑。请增强以下文本的文学性和表现力：\n\n{{content}}".to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec!["content".to_string()],
        },
    );

    m.insert(
        "skill_plot_twist".to_string(),
        PromptEntry {
            id: "skill_plot_twist".to_string(),
            name: "情节反转生成器提示词".to_string(),
            description: "生成出人意料的情节反转".to_string(),
            category: PromptCategory::Skill,
            default_content: "你是一个擅长情节设计的编剧。请基于以下上下文生成出人意料的情节反转：\n\n{{context}}".to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec!["context".to_string()],
        },
    );

    m.insert(
        "skill_text_formatter".to_string(),
        PromptEntry {
            id: "skill_text_formatter".to_string(),
            name: "文本排版器提示词".to_string(),
            description: "对小说正文进行智能排版".to_string(),
            category: PromptCategory::Skill,
            default_content: r#"你是一位专业的中文小说排版编辑。请对输入的小说正文进行智能排版优化：
1. 合理分段：根据语义和场景转换进行分段
2. 对话格式：确保对话单独成段，使用正确的引号和标点
3. 场景转换：场景或视角转换时添加空行分隔
4. 标点规范：修正错误的标点使用，统一全角标点
5. 保留原意：不改变原文的内容和表达意图
6. 输出纯文本，不要添加任何解释

请对以下小说正文进行智能排版优化，只返回排版后的正文内容：

{{content}}"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec!["content".to_string()],
        },
    );

    m.insert(
        "skill_character_voice".to_string(),
        PromptEntry {
            id: "skill_character_voice".to_string(),
            name: "角色声音一致性提示词".to_string(),
            description: "检查并增强角色对话的声音一致性".to_string(),
            category: PromptCategory::Skill,
            default_content: r#"你是一位专业的角色声音分析师。请检查并增强角色对话的一致性：
1. 每个角色的用词习惯保持一致
2. 语气、句式结构符合角色性格
3. 对话中体现角色的独特性格特征
4. 不同角色之间有明显的语言区分度
5. 输出修正后的对话文本，不要添加解释

【角色】{{character_name}}
【特征】{{character_traits}}

【对话内容】
{{content}}

请输出修正后的对话，确保角色声音统一且鲜明。只返回文本，不要解释。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "character_name".to_string(),
                "character_traits".to_string(),
                "content".to_string(),
            ],
        },
    );

    m.insert(
        "skill_emotion_pacing".to_string(),
        PromptEntry {
            id: "skill_emotion_pacing".to_string(),
            name: "情感节奏优化提示词".to_string(),
            description: "分析并优化文本的情感曲线和叙事节奏".to_string(),
            category: PromptCategory::Skill,
            default_content: r#"你是一位专业的叙事节奏和情感分析师。请以「{{mode}}」模式处理以下文本：

如果是 analyze 模式，给出情感节奏分析和改进建议（不超过200字）。
如果是 rewrite 模式，直接输出优化后的文本，增强情感张力和叙事节奏。

【文本】
{{content}}"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec!["mode".to_string(), "content".to_string()],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 10. 多助手系统提示词 (System)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "agent_world_building".to_string(),
        PromptEntry {
            id: "agent_world_building".to_string(),
            name: "世界观助手系统提示词".to_string(),
            description: "幕后世界观助手".to_string(),
            category: PromptCategory::System,
            default_content: r#"你是世界观助手，专门帮助构建和完善小说的世界观设定。

你的职责：
1. 帮助设计和完善世界规则、历史背景、文化设定
2. 回答关于世界观的问题
3. 指出设定中的潜在冲突或不一致
4. 提供灵感建议

回答时请：
- 引用相关的Wiki页面
- 保持与已有设定的一致性
- 提供具体可行的建议"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "agent_character".to_string(),
        PromptEntry {
            id: "agent_character".to_string(),
            name: "人物助手系统提示词".to_string(),
            description: "幕后人物助手".to_string(),
            category: PromptCategory::System,
            default_content: r#"你是人物助手，专门帮助塑造角色形象和性格发展。

你的职责：
1. 帮助设计角色的性格、背景、动机
2. 分析角色间的关系和互动
3. 提供角色发展建议
4. 确保角色行为符合其性格设定

回答时请：
- 引用角色相关的Wiki页面
- 考虑角色的成长弧线
- 提供具体的对话或行为示例"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "agent_writing_style".to_string(),
        PromptEntry {
            id: "agent_writing_style".to_string(),
            name: "文风助手系统提示词".to_string(),
            description: "幕后文风助手".to_string(),
            category: PromptCategory::System,
            default_content: r#"你是文风助手，专门帮助优化写作风格和语言表达。

你的职责：
1. 提供文风改进建议
2. 帮助修改段落使其更符合设定风格
3. 分析文本的节奏、语气、用词
4. 提供具体的修改方案

回答时请：
- 引用文风相关的Wiki页面
- 给出修改前后的对比
- 解释修改的原因"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "agent_scene".to_string(),
        PromptEntry {
            id: "agent_scene".to_string(),
            name: "场景助手系统提示词".to_string(),
            description: "幕后场景助手".to_string(),
            category: PromptCategory::System,
            default_content: r#"你是场景助手，专门帮助设计戏剧性的场景和情节发展。

你的职责：
1. 帮助设计场景的戏剧冲突
2. 提供场景布局、节奏控制建议
3. 分析场景的戏剧效果
4. 建议如何增强场景的紧张感或情感冲击力

回答时请：
- 引用场景相关的Wiki页面
- 关注戏剧目标、外部压迫、冲突类型
- 提供具体的场景设计建议"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "agent_plot".to_string(),
        PromptEntry {
            id: "agent_plot".to_string(),
            name: "情节助手系统提示词".to_string(),
            description: "幕后情节助手".to_string(),
            category: PromptCategory::System,
            default_content: r#"你是情节助手，专门帮助规划和优化故事线。

你的职责：
1. 帮助设计情节转折和高潮
2. 分析故事结构的合理性
3. 提供伏笔和照应的设计建议
4. 确保情节推进符合逻辑

回答时请：
- 引用情节相关的Wiki页面
- 考虑前后文的连贯性
- 提供多种可能的发展方向"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 11. 叙事结构 (Narrative)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "narrative_event_extraction".to_string(),
        PromptEntry {
            id: "narrative_event_extraction".to_string(),
            name: "叙事事件提取提示词".to_string(),
            description: "从文本中提取推动情节发展的关键事件".to_string(),
            category: PromptCategory::Narrative,
            default_content: r#"你是一个专业的叙事分析专家。从小说文本中提取推动情节发展的关键事件。

分析标准：
1. 「有效事件」= 真正推动情节发展的关键节点
2. 事件强度（0.0-1.0）反映对后续情节的影响程度
3. 如果角色发生内在改变，标记为角色弧光
4. 伏笔埋设和回收是独立事件
5. 保持与已有事件链的因果一致性

【角色列表】
{{characters}}

【已有事件链】
{{prior_events}}

【当前文本】
{{content}}

请输出 JSON 格式的事件数组，每个事件包含：
- event_type: 事件类型（introduction/turning_point/climax/resolution/revelation/conflict_eruption/character_arc/foreshadow_setup/foreshadow_payoff/transition）
- intensity: 事件强度（0.0-1.0）
- sentiment: 情感极性（-1.0 到 +1.0）
- description: 事件描述（20-50字）
- involved_character_ids: 涉及的角色 ID 数组
- conflict_types: 涉及的冲突类型数组

只输出 JSON，不要其他文字。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![
                "characters".to_string(),
                "prior_events".to_string(),
                "content".to_string(),
            ],
        },
    );

    m.insert(
        "narrative_structure_analysis".to_string(),
        PromptEntry {
            id: "narrative_structure_analysis".to_string(),
            name: "叙事结构分析提示词".to_string(),
            description: "基于事件强度分布推断幕结构".to_string(),
            category: PromptCategory::Narrative,
            default_content: r#"你是一个专业的叙事结构分析专家。基于事件强度分布，推断故事的幕级结构。

分析标准：
1. 基于亚里士多德五幕结构：起→承→转→合
2. 高潮点 = 事件强度达到局部最大值的位置
3. 幕边界 = 事件强度发生显著突变的位置

【事件时间线】
{{event_timeline}}

请输出 JSON 格式的分析结果：
- acts: 幕数组（act_number, act_type, start_chapter, end_chapter, summary）
- positions: 事件位置数组（event_id, act_number, position_in_act, dramatic_function, is_narrative_boundary）

只输出 JSON，不要其他文字。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec!["event_timeline".to_string()],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 13. 方法论 - 雪花法 (Methodology)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "methodology_snowflake_step1".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step1".to_string(),
            name: "雪花法第1步：一句话故事".to_string(),
            description: "用一句话概括整个故事".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第1步：一句话故事
请用一句话概括整个故事，包含：
- 故事主角（1-2个形容词 + 身份）
- 主角的目标
- 阻碍主角的对抗力量
- 一句话必须有戏剧性张力

示例：一位被诬陷入狱的银行家，在肖申克监狱中用二十年时间秘密挖掘隧道，最终越狱并获得自由与财富。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "methodology_snowflake_step2".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step2".to_string(),
            name: "雪花法第2步：一段扩展".to_string(),
            description: "将一句话扩展为五句话".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第2步：一段扩展
将一句话故事扩展为五句话：
1. 故事背景（设定、世界、主角状态）
2. 第一个灾难：迫使主角离开舒适区的事件
3. 第二个灾难：主角尝试解决问题但失败，处境更糟
4. 第三个灾难：看似胜利实则引向最终危机的事件
5. 结局：主角最终如何（成功/失败/ bittersweet）

每句话必须推动情节发展，包含因果逻辑。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "methodology_snowflake_step3".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step3".to_string(),
            name: "雪花法第3步：角色概要".to_string(),
            description: "为每个主要角色写一页概要".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第3步：角色概要
为每个主要角色写一页概要，必须包含：
- 名字 + 一句话外貌/身份描述
- 目标：这个角色想要什么？（具体、可衡量）
- 动机：为什么想要？（情感根源）
- 冲突：什么阻碍他/她？（内在+外在）
- 顿悟：在故事结尾，角色学到了什么？
- 一句话总结：这个角色在故事中的弧线

每个角色的目标必须与其他角色产生冲突。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "methodology_snowflake_step4".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step4".to_string(),
            name: "雪花法第4步：段落扩展".to_string(),
            description: "将五句话扩展为五个段落".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第4步：段落扩展
将第2步的五句话中的每一句话扩展为一个完整段落：
- 每个段落包含：背景细节 + 事件展开 + 情感反应
- 段落之间必须有清晰的因果关系
- 总长度控制在 400-600 字
- 保留五句话的核心结构，但添加丰富细节"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "methodology_snowflake_step5".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step5".to_string(),
            name: "雪花法第5步：角色详细表".to_string(),
            description: "为每个主要角色写完整小传".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第5步：角色详细表
为每个主要角色写完整小传（约2页），包含：
- 出生背景：家庭、童年关键事件
- 性格形成：什么经历塑造了他的性格？
- 核心价值观：他/她最看重什么？
- 恐惧与渴望：最深的恐惧是什么？最渴望什么？
- 人际关系：与其他角色的关系历史
- 转变时刻：故事前后角色的关键变化
- 对话特征：说话方式、口头禅、潜台词习惯"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "methodology_snowflake_step6".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step6".to_string(),
            name: "雪花法第6步：完整梗概".to_string(),
            description: "将五段扩展为完整故事梗概".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第6步：完整梗概
将第4步的五段扩展为完整故事梗概（4-5页）：
- 每个主要场景都需提及
- 明确标注：铺垫、升级、转折、高潮、结局
- 确保三幕式结构清晰
- 人物动机在每处转折都有合理依据
- 埋下至少3个伏笔并注明回收位置"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "methodology_snowflake_step7".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step7".to_string(),
            name: "雪花法第7步：场景表".to_string(),
            description: "列出故事中所有场景".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第7步：场景表
列出故事中所有场景，每个场景包含：
- 场景编号
- POV角色（谁的眼睛看这个世界）
- 场景目标（角色想在这个场景达成什么）
- 冲突（什么阻碍了目标）
- 挫折（场景结束时角色比开始时更糟吗？）
- 场景类型：目标场景(Goal) 或 反应场景(Reaction)

目标场景公式：目标 → 冲突 → 灾难
反应场景公式：反应 → 困境 → 决定"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "methodology_snowflake_step8".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step8".to_string(),
            name: "雪花法第8步：场景扩展".to_string(),
            description: "将每个场景扩展为段落级描述".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第8步：场景扩展
将场景表中的每个场景扩展为段落级描述：
- 场景开头：时间、地点、氛围、角色状态
- 中间：对话+动作+内心活动，推动冲突升级
- 结尾：挫折/转折，留下钩子
- 每个场景约 200-400 字描述
- 标注情感基调变化"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "methodology_snowflake_step9".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step9".to_string(),
            name: "雪花法第9步：初稿写作".to_string(),
            description: "将场景扩展写为完整章节".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第9步：初稿写作
根据第8步的场景扩展，将每个场景写为完整章节段落：
- 遵循场景结构规范（目标-冲突-灾难 或 反应-困境-决定）
- 保持角色声音一致性
- 每章结尾留钩子
- 对话推动情节，避免无意义闲聊
- 展示而非讲述（Show, don't tell）"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    m.insert(
        "methodology_snowflake_step10".to_string(),
        PromptEntry {
            id: "methodology_snowflake_step10".to_string(),
            name: "雪花法第10步：修改润色".to_string(),
            description: "检查清单与最终修改".to_string(),
            category: PromptCategory::Methodology,
            default_content: r#"雪花写作法 - 第10步：修改润色
检查清单：
1. 结构：三幕式是否清晰？每个场景是否必要？
2. 角色：动机是否充分？弧线是否完整？
3. 节奏：紧张与松弛交替是否自然？
4. 对话：每句对话是否推动情节或揭示性格？
5. 伏笔：所有伏笔是否都已回收或明确放弃？
6. 世界观：设定是否前后一致？
7. 语言：删除冗余描写，强化关键意象"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec![],
        },
    );

    // ═══════════════════════════════════════════════════════
    // 12. 风格模仿 (System)
    // ═══════════════════════════════════════════════════════

    m.insert(
        "style_mimic".to_string(),
        PromptEntry {
            id: "style_mimic".to_string(),
            name: "风格模仿提示词".to_string(),
            description: "模仿参考文风改写文本".to_string(),
            category: PromptCategory::System,
            default_content: r#"【参考文风样例】
{{style_sample}}

【需要改写的文本】
{{content}}

请模仿参考文风的语言特点（词汇选择、句式结构、修辞手法等），改写上述文本，保持原意但改变表达方式。"#.to_string(),
            current_content: String::new(),
            is_overridden: false,
            variables: vec!["style_sample".to_string(), "content".to_string()],
        },
    );

    m
}

fn get_builtin_prompts() -> &'static HashMap<String, PromptEntry> {
    BUILTIN_PROMPTS.get_or_init(init_builtin_prompts)
}

// ─────────────────────────────────────────────────────────────
// 公开 API
// ─────────────────────────────────────────────────────────────

/// 列出所有提示词条目（含覆盖状态）
pub fn list_prompt_entries(pool: &DbPool) -> Result<Vec<PromptEntry>, AppError> {
    let builtins = get_builtin_prompts();
    let overrides = load_overrides(pool)?;

    let mut entries: Vec<PromptEntry> = builtins
        .values()
        .map(|entry| {
            let mut e = entry.clone();
            if let Some(override_content) = overrides.get(&entry.id) {
                e.current_content = override_content.clone();
                e.is_overridden = true;
            } else {
                e.current_content = entry.default_content.clone();
                e.is_overridden = false;
            }
            e
        })
        .collect();

    // 按分类排序，再按 ID 排序
    entries.sort_by(|a, b| {
        a.category
            .order()
            .cmp(&b.category.order())
            .then_with(|| a.id.cmp(&b.id))
    });

    Ok(entries)
}

/// 解析提示词：优先读取用户覆盖，否则返回内置默认
pub fn resolve_prompt(pool: &DbPool, prompt_id: &str) -> Result<String, AppError> {
    let builtins = get_builtin_prompts();
    let default = builtins
        .get(prompt_id)
        .map(|e| e.default_content.clone())
        .ok_or_else(|| AppError::Internal {
            message: format!("未知提示词 ID: {}", prompt_id),
        })?;

    let overrides = load_overrides(pool)?;
    Ok(overrides.get(prompt_id).cloned().unwrap_or(default))
}

/// 无数据库连接时的回退解析（用于启动早期）
pub fn resolve_prompt_default(prompt_id: &str) -> Option<String> {
    get_builtin_prompts()
        .get(prompt_id)
        .map(|e| e.default_content.clone())
}

/// 保存提示词覆盖
pub fn save_override(
    pool: &DbPool,
    prompt_id: &str,
    content: &str,
) -> Result<(), AppError> {
    // 验证 prompt_id 是否有效
    let builtins = get_builtin_prompts();
    if !builtins.contains_key(prompt_id) {
        return Err(AppError::Internal {
            message: format!("未知提示词 ID: {}", prompt_id),
        });
    }

    let conn = pool.get().map_err(|e| AppError::Internal {
        message: format!("数据库连接失败: {}", e),
    })?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    conn.execute(
        "INSERT OR REPLACE INTO prompt_overrides (prompt_id, overridden_content, updated_at) \
         VALUES (?1, ?2, ?3)",
        rusqlite::params![prompt_id, content, now],
    )
    .map_err(|e| AppError::Internal {
        message: format!("保存提示词覆盖失败: {}", e),
    })?;

    log::info!("[PromptRegistry] 已保存提示词覆盖: {}", prompt_id);
    Ok(())
}

/// 重置提示词为默认（删除覆盖）
pub fn reset_override(pool: &DbPool, prompt_id: &str) -> Result<(), AppError> {
    let conn = pool.get().map_err(|e| AppError::Internal {
        message: format!("数据库连接失败: {}", e),
    })?;

    conn.execute(
        "DELETE FROM prompt_overrides WHERE prompt_id = ?1",
        [prompt_id],
    )
    .map_err(|e| AppError::Internal {
        message: format!("重置提示词失败: {}", e),
    })?;

    log::info!("[PromptRegistry] 已重置提示词: {}", prompt_id);
    Ok(())
}

/// 批量重置所有提示词
pub fn reset_all_overrides(pool: &DbPool) -> Result<usize, AppError> {
    let conn = pool.get().map_err(|e| AppError::Internal {
        message: format!("数据库连接失败: {}", e),
    })?;

    let count = conn
        .execute("DELETE FROM prompt_overrides", [])
        .map_err(|e| AppError::Internal {
            message: format!("批量重置提示词失败: {}", e),
        })?;

    log::info!("[PromptRegistry] 已重置所有提示词覆盖，共 {} 条", count);
    Ok(count)
}

// ─────────────────────────────────────────────────────────────
// 内部辅助
// ─────────────────────────────────────────────────────────────

fn load_overrides(pool: &DbPool) -> Result<HashMap<String, String>, AppError> {
    let conn = pool.get().map_err(|e| AppError::Internal {
        message: format!("数据库连接失败: {}", e),
    })?;

    let mut stmt = conn
        .prepare(
            "SELECT prompt_id, overridden_content FROM prompt_overrides",
        )
        .map_err(|e| AppError::Internal {
            message: format!("查询提示词覆盖失败: {}", e),
        })?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| AppError::Internal {
            message: format!("读取提示词覆盖失败: {}", e),
        })?;

    let mut overrides = HashMap::new();
    for row in rows {
        let (id, content) = row.map_err(|e| AppError::Internal {
            message: format!("解析提示词覆盖失败: {}", e),
        })?;
        overrides.insert(id, content);
    }

    Ok(overrides)
}

// ─────────────────────────────────────────────────────────────
// 测试
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_prompts_count() {
        let prompts = get_builtin_prompts();
        assert!(
            prompts.len() >= 35,
            "内置提示词数量应不少于 35，实际 {}",
            prompts.len()
        );
    }

    #[test]
    fn test_resolve_prompt_default() {
        let content = resolve_prompt_default("writer_system");
        assert!(content.is_some());
        assert!(content.unwrap().contains("小说创作助手"));
    }

    #[test]
    fn test_unknown_prompt_id() {
        let content = resolve_prompt_default("nonexistent");
        assert!(content.is_none());
    }

    #[test]
    fn test_prompt_categories() {
        let prompts = get_builtin_prompts();
        let categories: std::collections::HashSet<_> = prompts
            .values()
            .map(|e| e.category.clone())
            .collect();
        assert!(
            categories.len() >= 10,
            "应包含至少 10 个不同分类"
        );
    }

    #[test]
    fn test_writer_system_has_variables() {
        let prompts = get_builtin_prompts();
        let writer = prompts.get("writer_system").unwrap();
        assert!(!writer.variables.is_empty());
        assert!(writer.variables.contains(&"story_title".to_string()));
    }

    #[test]
    fn test_category_order() {
        assert!(PromptCategory::Writer.order() < PromptCategory::Inspector.order());
        assert!(PromptCategory::Inspector.order() < PromptCategory::Other.order());
    }
}
