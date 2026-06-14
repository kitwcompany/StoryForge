# 文思资产分级与多模式创作系统设计文档（v2 修订版）

> 创建日期: 2026-06-14
> 修订日期: 2026-06-14（基于代码现状核查重写）
> 状态: 设计草案，待评审
> 问题: AI 创作中资产深度介入导致生成过慢，质量与速度不可兼得
> 前序文档: [`2026-06-14-asset-tier-creation-design.md`](./2026-06-14-asset-tier-creation-design.md)（v1，已发现多处与代码现状不符，本版修正）

---

## 0. 修订说明（v1 → v2 关键变更）

v1 文档对代码现状的认知滞后于实际项目，存在系统性偏差。本版基于对 `src-tauri/src/` 与 `src-frontend/src/` 的逐行核查重写，主要修正：

| # | v1 的问题 | v2 的处理 |
|---|----------|----------|
| 1 | 新建 `AssetTier { L0..L3 }` 与现有 `ContextOptimizer` 的 `L0/L1/L2`（`agents/context_optimizer.rs:307-572`）命名冲突 | 改名 `AssetBundle`，明确与 ContextOptimizer 的边界（见模块 7） |
| 2 | `pipeline/inspector.rs` 不存在；Inspector 实际分布在 `agents/inspector.rs`（遗留）+ `pipeline/review.rs`（6维 prompt）+ `prompts/engine.rs`（已含第 7 维 `memory_score`） | 修正路径，承认 Inspector 已是 7 维 + style_analysis + memory_analysis（`prompts/engine.rs:182-231`） |
| 3 | "现有 Inspector 6 维度"滞后于 memory-everywhere 落地 | 按现状写为 7 维，Standard 跳过 Inspector（不再造 Light/Deep 两套结构体） |
| 4 | `src-frontend/src/agents/types.ts`、顶层 `commands.rs` 不存在 | 修正为 `types/index.ts`、`commands/` 目录 |
| 5 | 称 `smart_execute` "路由依赖后端 LLM 意图识别（一次 LLM 调用）" | 核查 `commands/orchestrator.rs:28`：`smart_execute` 用 `is_novel_creation_intent()`（纯规则）分流，**不发 LLM 做路由**。性能论证前提修正 |
| 6 | `GenerationTier { Standard, Pro }` 与现有 `GenerationMode { Fast, Full }`、`SubscriptionTier { Free, Pro }` 三个概念未厘清 | 决策：**扩展 `GenerationMode` 为三值 `{ Fast, Standard, Pro }`**，`SubscriptionTier` 退为门禁（见模块 2） |
| 7 | `build_writer_prompt(&self, task, tier: SubscriptionTier)` 的 `tier` 被误当作资产层级 | 澄清：现有 `tier` 是订阅等级，本设计新增 `AssetBundle` 参数，二者正交 |
| 8 | 合同体系被归 P2"审计级"，与"写作前门禁"职责矛盾 | 决策：**合同体系整体放 P1**（见模块 1），Standard 模式靠 GenreProfile + 角色核心兜底 |

---

## 1. 设计决策摘要

### 用户痛点

强化了专业资产介入就无法迅速得到正文内容，放松则无法达到应有质量。

### 根本原因（基于代码核查）

`AgentOrchestrator::generate` 当前只有两条路径（`agents/orchestrator.rs:333-335`）：

```rust
match mode {
    GenerationMode::Fast  => self.execute_fast(...),   // Ghost Text 用，跳过 Inspector
    GenerationMode::Full  => self.execute_full(...),    // 标准写作也走这条 —— 全量资产 + Inspector + Rewrite
}
```

所有非 Ghost Text 的生成（`auto_write`/`auto_revise`/普通生成按钮）**都硬编码 `GenerationMode::Full`**（`agents/commands.rs:342,699,1109`；`creation_commands.rs:1128`；`workflow/scheduler.rs:445,543`；`planner/executor.rs:760`；`narrative/genesis.rs:582`，共 7 处）。标准模式用户因此被迫承载：完整 Preflight（4 项 DB 查询 + 可能触发 5 次 LLM 的 `auto_contract`）、全量资产加载、7 维 Inspector 阻塞、最多 `max_feedback_loops` 轮 Rewrite。

### 解决方案（三项核心变更）

1. **扩展 `GenerationMode` 为三值** `{ Fast, Standard, Pro }`，资产深度内生于模式。
2. **新建 `AssetBundle` 资产打包层**（与现有 `ContextOptimizer` 解耦），按模式加载不同资产集合注入 Writer prompt。
3. **`/` 指令固定走 Pro 模式**，路由改为前端关键词匹配（现状已是精确字符串匹配，本设计只是扩展匹配表）。

### 三个正交概念的关系

| 概念 | 定义 | 控制什么 | 当前代码 |
|---|---|---|---|
| `GenerationMode` | 生成模式 | 执行路径（资产深度 + 是否 Inspector + 是否 Rewrite） | `agents/orchestrator.rs:25`，现为 `{Fast, Full}` |
| `AssetBundle` | 资产打包 | 注入 prompt 的资产集合（本设计新增） | 无，分散在 `build_writer_prompt` |
| `SubscriptionTier` | 订阅等级 | 用户付费额度（Free/Pro） | `subscription::SubscriptionTier`，作门禁 |

门禁规则：`SubscriptionTier::Free` 用户能否触发 `GenerationMode::Pro`？默认**允许**（Pro 模式只是更慢更深，不额外消耗用户配额之外的资源），但保留为可配置开关。

---

## 2. 模块 1：资产分级体系（AssetBundle）

### 设计原则

- `AssetBundle` 是**数据打包层**：决定「加载哪些资产、注入到 prompt 哪里」。
- `ContextOptimizer` 是 **token 预算调度层**：决定「在有限窗口内如何裁剪、排序、压缩」。二者正交，详见模块 7。
- 分级以「资产在生成链路中的职责」为准，不按"加载耗时"机械分层。

### 分级定义

#### P0 — 防错级（Fast/Standard/Pro 均加载）

确保 AI 生成不偏离基础设定的最小集合。Fast 模式（Ghost Text）也加载，因为补全同样不能出现不存在的角色。

- **L0 元数据**（复用 `ContextOptimizer::build_l0`）：故事标题 / 题材 / 基调 / 节奏
- **GenreProfile 基础约束**：`templates/genres.json` 中 43 个体裁模板的「核心基调 / 节奏策略 / 反模式清单」三字段（不含完整参考数据表）
- **角色核心**：姓名 + 关系（仅当前章节出场角色，由 `scene_characters` 表过滤）
- **场景大纲**：当前 scene 的 `dramatic_goal` + `conflict_type`
- **主线方向约束**：故事级一句话方向（`stories.description` 截断）

> 注：P0 不含合同体系。Standard 模式靠 GenreProfile 反模式清单 + 角色核心做"软约束"，不做合同级硬门禁。

#### P1 — 专业级（Standard/Pro 加载）

决定作品专业质量的核心引擎。Standard 模式加载这些资产**但不跑 Inspector/Rewrite**（资产注入 prompt 让 Writer 一次写好，而非写完再审）。

- **方法论引擎**：雪花法 / 场景节拍表 / 英雄之旅 / 人物深度 / 高密度世界构建（`creative_engine/methodology/`），仅加载当前故事 `methodology_id` 对应的当前步骤指引
- **风格 DNA**：六维模型（词汇/句法/修辞/视角/情感/对白）+ `StyleBlend`（若设置）+ `StyleFingerprint` 锚点片段（`creative_engine/style/`）
- **合同体系**：MASTER_SETTING + 当前 CHAPTER 的 `RuntimeContract`（**v1 误放 P2，本版上移**——它是写作前防偏离的核心约束，应在生成时就注入，而非写后审计）
- **三层记忆编排器**：Working + Episodic + Semantic 的 `MemoryPack`（`memory/orchestrator.rs`，memory-everywhere 已落地）
- **叙事快照**：`CanonicalState`（世界事实 / pending payoffs / 角色状态 / 冲突 / 时间线）

#### P2 — 审计级（仅 Pro 加载）

用于 Inspector 7 维深度评分的资产。Standard 模式跳过 Inspector，故不加载。

- **完整 ContractTree**：跨章节合同树（含 Review 合同），供 Inspector 维度 1「逻辑连贯性」比对
- **伏笔追踪全量**：`ForeshadowingTracker` 开放伏笔 + 逾期预警（供维度 4「伏笔回收」）
- **追读力历史**：`chapter_reading_power` 历史 N 章趋势（供维度 6「节奏把控」做相对评估）
- **Anti-AI 基线**：前文词汇/句式分布（供维度 5「风格」做漂移对比）

#### P3 — 深度洞察级（仅 Pro 加载，按需）

需要跨章节深度关联、且单次 LLM 调用难以覆盖的场景。

- **语义检索**：`hybrid_search_vectors`（BM25 + Vector RRF 融合，`memory/hybrid_search.rs`）
- **知识图谱深度查询**：KG 实体 + 关系深度遍历（`get_story_graph`）
- **记忆 Semantic 层向量化数据**：补充 MemoryPack Semantic 层的底层数据来源（修复 v1 把 MemoryPack 放 P1、向量检索放 P3 导致的断链）

> P3 与 P1 的记忆系统存在依赖：MemoryPack 的 Semantic 层依赖向量检索结果。Pro 模式下两者都加载，由 `MemoryOrchestrator` 内部协调；Standard 模式下 P3 不加载，MemoryPack 的 Semantic 层降级为「仅 Working + Episodic」。

### 模式 × 资产矩阵

| 资产 | Fast | Standard | Pro |
|---|:---:|:---:|:---:|
| P0 防错级 | ✅ | ✅ | ✅ |
| P1 专业级 | ❌ | ✅ | ✅ |
| P2 审计级 | ❌ | ❌ | ✅ |
| P3 深度洞察级 | ❌ | ❌ | ✅（按需） |

---

## 3. 模块 2：`GenerationMode` 扩展为三值

### 枚举定义

```rust
// agents/orchestrator.rs（修改现有枚举）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationMode {
    /// 快速模式：单轮 LLM，仅 P0 资产，跳过 Inspector / StyleChecker
    /// 用于 Ghost Text、实时补全等低延迟场景。保持现有行为不变。
    Fast,
    /// 标准模式：P0+P1 资产注入，单轮 Writer，跳过 Inspector 与 Rewrite
    /// 用于 auto_write / 普通生成按钮，追求速度。
    Standard,
    /// 专业模式：P0+P1+P2+P3 资产，7 维 Inspector 阻塞，最多 2 轮 Rewrite
    /// 用于 / 指令、明确要求专业水准的场景。质量优先。
    Pro,
}
```

### 现有 `Full` 的迁移

现有 7 处 `GenerationMode::Full` 调用点按语义重新映射：

| 调用点 | 现状 | 迁移目标 | 理由 |
|---|---|---|---|
| `agents/commands.rs:342`（普通生成） | Full | **Standard** | 用户点"生成"按钮，期望快速出结果 |
| `agents/commands.rs:699,1109`（`auto_write`/`auto_revise`） | Full | **Standard** | 文思泉涌的自动续写，速度优先 |
| `creation_commands.rs:1128`（创作向导首场景） | Full | **Pro** | 向导是用户明确的专业创作启动，质量优先 |
| `planner/executor.rs:760` | Full | **Pro** | Planner 执行的是用户规划的复杂任务 |
| `narrative/genesis.rs:582`（Genesis 管线） | Full | **Pro** | 整书生成，质量优先 |
| `workflow/scheduler.rs:445,543`（工作流调度） | Full | **Pro** | 工作流是专业自动化流程 |

> **兼容性**：`Full` 枚举值保留为 `Pro` 的别名（`pub const Full = Pro`）一个版本周期，避免遗漏调用点导致编译失败。下个版本移除。
>
> **兜底默认值**：`agents/executor.rs:79` 的 `_ => GenerationMode::Full`（未知 mode 字符串的默认）需同步改为 `GenerationMode::Standard`（更安全的默认，避免误触重型 Pro 路径）。

### 模式行为差异

| 维度 | Fast | Standard | Pro |
|---|---|---|---|
| 资产 | P0 | P0+P1 | P0+P1+P2+P3 |
| Preflight | 跳过 | QuickCheck（仅角色非空） | FullCheck（4 项 + auto_contract） |
| Writer 候选数 | 1 | 1（本地）/ 1-2（远端） | 1-2（远端）/ 1（本地） |
| Inspector | 跳过 | **跳过**（决策：Standard 完全不做 Inspector） | 7 维阻塞 |
| Rewrite | 跳过 | 跳过 | 最多 2 轮（`skip_rewrite_threshold`=0.90） |
| StyleChecker | 跳过 | 轻量（仅 fingerprint 打分，不阻塞） | 全维度 |
| apply_writing_skills | 否 | 否 | 是（emotion_pacing + style_enhancer） |
| Memory write | 否 | P1 三层（Working+Episodic，Semantic 降级） | 全量（含 P3 KG/向量） |
| Post-processing skills | 否 | 否 | 是 |

### 关于"无模式"

v1 把"无模式"列为三模式之一是概念混淆。实际不存在"无模式"执行路径——它是**待机态**（用户未触发任何生成）。`GenerationMode` 枚举只有三个值。用户输入 `/` 直接进入 Pro，不经过"先无后有"的状态机。

### Standard 模式跳过 Inspector 的决策依据

v1 设计了 LightInspector（不阻塞、仅信息收集），但存在两个问题：①不阻塞意味着异步跑，调度时机未定义；②收集的信息若无 Rewrite 闭环消费，就是死数据。

决策：**Standard 模式完全跳过 Inspector**（与跳过 Rewrite 一致）。质量保障靠 P0+P1 资产在 prompt 注入阶段约束 Writer「一次写对」，而非写后再审。这最大化速度，且避免死数据。若用户对 Standard 结果不满意，可手动触发 `/审校`（走 Pro 的 `review_draft`）。

---

## 4. 模块 3：`/` 指令路由

### 现状核查（修正 v1 描述）

`handleSlashSubmit`（`src-frontend/src/frontstage/components/RichTextEditor.tsx:603-619`）真实逻辑：

```ts
if (text === '自动续写') onSlashCommand?.('auto_write');
else if (text === '审校') onSlashCommand?.('auto_revise');
else onSmartGeneration?.(text);  // 统一走 smart_execute
```

即：两个**精确字符串匹配**（`自动续写`/`审校`）打开 WenSiPanel，其余兜底走 `smart_execute`。**不是 v1 说的"三路分发依赖后端 LLM IntentParser"**。

`smart_execute`（`commands/orchestrator.rs:28`）内部用 `is_novel_creation_intent(&user_input)`（**纯规则函数，不发 LLM**）判断是否启动 GenesisPipeline，否则走默认生成。`parse_intent` 命令（`commands/intent.rs:9`，调 `IntentParser::parse` → `llm_service.generate_for_task`，`intent.rs:124-129`）是另一条独立路径，`smart_execute` **没有调用它**。

> **性能论证修正**：v1 称"移除一次 LLM 调用"能省时间——但 `smart_execute` 本就不发 LLM 做路由。本设计的性能收益来自「Standard 模式跳过 Inspector/Rewrite/auto_contract」，**不来自**路由层省 LLM。

### 重构方向

`/` 是**专业命令的标志**，所有 `/` 指令固定走 `GenerationMode::Pro`。路由改为**前端扩展关键词匹配表**（现状已是精确匹配，本设计扩展为关键词表），匹配结果随 `command_type` 传给后端，后端据此选择执行器（Writer/Pipeline/Analyzer），但**统一以 Pro 模式运行**。

### 前端路由表

| 关键词（包含匹配） | `command_type` | 后端执行器 | 说明 |
|---|---|---|---|
| `自动续写` | `auto_write_pro` | `WriterAgent(mode=Pro)` | 升级现有精确匹配为 Pro 续写 |
| `审校` / `审读` / `润色` / `精修` | `pipeline_review` | `Pipeline(refine→review)` | 扩展现有"审校"为多关键词 |
| `续写` / `写` / `生成` / `继续` / `往下` | `writer_pro` | `WriterAgent(mode=Pro)` | 续写类 |
| `修改` / `优化` / `改` | `writer_revise_pro` | `WriterAgent(mode=Pro, revise)` | 修改类 |
| `分析` / `评价` / `追读力` / `诊断` | `analyzer_pro` | `Inspector(mode=Pro, readonly)` | 分析类，只评分不改写 |
| `角色状态` / `角色更新` | `character_state` | `CharacterStateService` | 角色状态类 |
| 其他自然语言 | `writer_pro_free` | `WriterAgent(mode=Pro, 自由指令)` | 兜底 |

### 实现位置（修正路径）

- 前端：`src-frontend/src/frontstage/components/RichTextEditor.tsx` 的 `handleSlashSubmit`（`:603`）扩展匹配表，新增 `command_type` 字段透传。
- 前端类型：`src-frontend/src/types/index.ts`（**v1 误写 `src-frontend/src/agents/types.ts`，该目录不存在**）。
- 后端：`commands/orchestrator.rs::smart_execute`（`:28`）增加 `command_type: Option<String>` 参数，据此分派执行器，统一 `GenerationMode::Pro`。
- **后端 `IntentParser` 不移除**（v1 建议移除）——它服务于 `parse_intent`/`execute_intent` 命令链，与本设计无冲突，保持现状。

---

## 5. 模块 4：Preflight 延迟校验

### 现状核查

`PreflightChecker::check_sync`（`story_system/preflight.rs:42`）执行 4 项检查：
1. MASTER_SETTING contract 存在（`story_contracts` 表查询）
2. CHAPTER contract 存在
3. Characters 非空（`< 2` 也警告）
4. 当前 scene 有 outline

失败返回 `PreflightResult { ready: false, missing_contracts }`，上游 `agents/service.rs::prepare_writer_context` 据此调 `AutoContractBuilder::auto_fill`（`story_system/auto_contract.rs:63`），按 角色→场景→MASTER→CHAPTER→大纲 顺序补齐，**每步一次 LLM 调用，最多 5 次**。

### 重构方案

保留现有 `PreflightChecker` 作为 `FullPreflightChecker`，新增 `QuickPreflightChecker`。

```rust
// story_system/preflight.rs（新增）
pub struct QuickPreflightChecker;

impl QuickPreflightChecker {
    /// 标准模式预检：仅检查角色非空。spawn_blocking 包裹 DB 查询。
    pub async fn check(pool: &DbPool, story_id: &str) -> PreflightResult {
        let pool = pool.clone();
        tokio::task::spawn_blocking(move || {
            let repo = CharacterRepository::new(pool);
            let chars = repo.get_by_story(&story_id).unwrap_or_default();
            if chars.is_empty() {
                PreflightResult::failed(vec!["NoCharacters".into()])
            } else {
                PreflightResult::ok()
            }
        }).await.unwrap_or_else(|e| PreflightResult::failed(vec![format!("join err: {e}")]))
    }
}
```

模式选择：
- **Fast**：跳过 Preflight（Ghost Text 容忍不完美）
- **Standard**：`QuickPreflightChecker`（仅角色非空；失败直接报错，不触发 auto_contract——Standard 追求速度，不花 5 次 LLM 补合同）
- **Pro**：`FullPreflightChecker`（现有 4 项 + auto_contract 补齐）

> Standard 模式角色为空时的处理：直接返回错误提示用户「请先添加角色」，而非后台静默补齐。这与 v1"失败返回 `NoCharacters`"一致，但明确「不触发 auto_contract」。

---

## 6. 模块 5：Inspector（不再拆分 Light/Deep）

### 决策：不拆分 Inspector 为两个结构体

v1 计划把 Inspector 拆成 `LightInspector`/`DeepInspector` 两个结构体。**本版取消该拆分**，理由：
1. Inspector 现已是 **7 维 + style_analysis + memory_analysis**（`prompts/engine.rs:182-231`），拆分会割裂 memory-everywhere 刚建立的维度扩展。
2. Standard 模式决策为**完全跳过 Inspector**（见模块 2），不需要 Light 版本。
3. 拆分两套结构体增加维护负担，且评分逻辑会重复。

### 改动方式

- **Standard 模式**：`AgentOrchestrator::execute_standard`（新增）在 Writer 生成后**直接保存**，不调用 Inspector。对应代码：跳过 `execute_full`（`agents/orchestrator.rs:586`）中"步骤 2: Inspector 质检"（`:673`）及其后的 Rewrite 循环。
- **Pro 模式**：复用现有 `execute_full` 全流程，Inspector 保持 7 维。Rewrite 上限调整为 2 轮（现有 `max_feedback_loops` 配置项，Pro 模式强制 `min(config, 2)`）。

### Inspector 维度现状（供参考，本设计不改）

`prompts/engine.rs:182-201` 定义的 7 维（每维满分 20，总分 140）：
1. 逻辑连贯性（logic）
2. 人物深度（character）
3. ~~（文档未列全，实际有连续性/伏笔/风格等，共 7 维）~~
6. 节奏把控（pacing）
7. 记忆一致性（memory，memory-everywhere 新增）

外加：`style_analysis`（风格一致性，有参考文本时）、`memory_analysis`（角色冲突 / 伏笔遗漏列表）。

---

## 7. 模块 6：`AgentOrchestrator` 重构

### 新增 `execute_standard` 路径

```rust
// agents/orchestrator.rs
impl AgentOrchestrator {
    pub async fn generate(&self, task: AgentTask, mode: GenerationMode) -> ... {
        let trace = ...;
        match mode {
            GenerationMode::Fast     => self.execute_fast(task, &trace).await,
            GenerationMode::Standard => self.execute_standard(task, &trace).await,  // 新增
            GenerationMode::Pro      => self.execute_full(task, &trace).await,      // 原 Full
        }
    }

    /// 标准模式：P0+P1 资产，单轮 Writer，无 Inspector，无 Rewrite
    async fn execute_standard(&self, task: AgentTask, trace: &Trace) -> ... {
        // 1. QuickPreflightChecker（仅角色非空）
        // 2. AssetBundle::load(Standard) → 注入 P0+P1 到 prompt
        // 3. WriterAgent 单轮生成（candidate_count=1）
        // 4. 轻量 StyleChecker（仅 fingerprint 打分，记录不阻塞）
        // 5. 保存 + Memory write（P1 三层，Semantic 降级）
        // 6. emit GenerationPhase::Completed
    }
}
```

### 三路径对照

| 步骤 | execute_fast | execute_standard（新） | execute_full（原 Pro） |
|---|---|---|---|
| Preflight | 跳过 | QuickCheck | FullCheck + auto_contract |
| 资产加载 | P0（最小） | P0+P1 | P0+P1+P2+P3 |
| Writer 候选 | 1 | 1 | 1-2 |
| Inspector | 跳过 | 跳过 | 7 维阻塞 |
| Rewrite | 跳过 | 跳过 | 最多 2 轮 |
| StyleChecker | 跳过 | 轻量（不阻塞） | 全维度 |
| Memory write | 跳过 | P1 三层（Semantic 降级） | 全量 |

---

## 8. 模块 7：`AssetBundle` 与 `ContextOptimizer` 的边界

### 两者职责（避免 v1 的重复造轮子）

| | `AssetBundle`（新增） | `ContextOptimizer`（现有） |
|---|---|---|
| **定位** | 数据打包层 | token 预算调度层 |
| **职责** | 决定「加载哪些资产」 | 决定「在有限窗口内如何裁剪/排序/压缩」 |
| **输入** | `GenerationMode` + `story_id` | `AssetBundle` 产出 + token 预算 |
| **输出** | 资产数据结构（待格式化） | 最终注入 prompt 的字符串 |
| **代码位置** | `creative_engine/asset_loader.rs`（新增） | `agents/context_optimizer.rs`（现有，不改职责） |
| **缓存** | 自带缓存（story_id + mode） | 现有 `ContextCache`（RwLock, 50 条, 300s TTL） |

### 协作流程

```
AgentOrchestrator::execute_*(task, mode)
  │
  ├─ 1. AssetBundle::load(mode, story_id)   ← 数据打包：按 mode 加载 P0~P3 资产
  │     └─ 返回 AssetBundle { p0: P0Assets, p1: Option<P1Assets>, ... }
  │
  ├─ 2. ContextOptimizer::build(bundle, budget)   ← 预算调度：裁剪到 token 预算内
  │     └─ 复用现有 build_l0/build_l1 逻辑，输入改为 AssetBundle 而非直接查 DB
  │
  └─ 3. build_writer_prompt(task, SubscriptionTier, context_str)   ← prompt 组装
```

### `AssetBundle` 接口

```rust
// creative_engine/asset_loader.rs（新增）
use crate::agents::orchestrator::GenerationMode;

/// 资产包：按模式加载的资产集合
pub struct AssetBundle {
    pub p0: P0Assets,                          // 防错级，必有
    pub p1: Option<P1Assets>,                  // 专业级，Standard/Pro 有
    pub p2: Option<P2Assets>,                  // 审计级，仅 Pro
    pub p3: Option<P3Assets>,                  // 深度洞察级，仅 Pro 按需
}

impl AssetBundle {
    /// 按 GenerationMode 加载资产。spawn_blocking 包裹 DB 查询。
    pub async fn load(mode: GenerationMode, story_id: &str, pool: &DbPool) -> Result<Self>;
}

pub struct P0Assets { /* genre_profile, core_characters, scene_outline, story_direction */ }
pub struct P1Assets { /* methodology_step, style_dna, contracts, memory_pack, canonical_state */ }
pub struct P2Assets { /* full_contract_tree, foreshadowing, reading_power_history, anti_ai_baseline */ }
pub struct P3Assets { /* hybrid_search_results, kg_query_results */ }
```

### 缓存策略

- `AssetBundle` 自带缓存：key = `(story_id, mode)`，TTL 60s（比 ContextCache 短，因资产变更频繁）。
- 与 `ContextCache`（`creative_engine/context_builder.rs:44`，50 条 300s）独立，不合并——两者缓存粒度与失效策略不同。

### `build_writer_prompt` 签名变更

```rust
// agents/service.rs:1691（修改）
// 现状：
async fn build_writer_prompt(&self, task: &AgentTask, tier: SubscriptionTier) -> String

// 改为：
async fn build_writer_prompt(
    &self,
    task: &AgentTask,
    tier: SubscriptionTier,        // 保留：订阅等级（门禁用）
    bundle: &AssetBundle,          // 新增：资产包（替代函数内散落的 DB 查询）
) -> String
```

现有函数体内的 DB 查询（角色/世界观/大纲/合同/风格等）迁移到 `AssetBundle::load`，`build_writer_prompt` 只做 prompt 字符串组装。这是本设计最大的重构面，需配合回归测试。

---

## 9. 改动文件清单（已核查路径）

### 后端（Rust）

| 文件 | 变更 | 说明 |
|---|---|---|
| `src-tauri/src/agents/orchestrator.rs` | 修改 | `GenerationMode` 加 `Standard` 值；新增 `execute_standard`；`Full`→`Pro` 别名 |
| `src-tauri/src/creative_engine/mod.rs` | 修改 | 声明 `asset_loader` 子模块 |
| `src-tauri/src/creative_engine/asset_loader.rs` | **新增** | `AssetBundle` + `load()` + 缓存 |
| `src-tauri/src/agents/service.rs` | 修改 | `build_writer_prompt` 加 `bundle` 参数（`:1691`）；DB 查询迁出 |
| `src-tauri/src/story_system/preflight.rs` | 修改 | 新增 `QuickPreflightChecker`（现有 `PreflightChecker` 不动） |
| `src-tauri/src/agents/commands.rs` | 修改 | `auto_write`/`auto_revise` 的 `Full`→`Standard`（`:342,699,1109`） |
| `src-tauri/src/creation_commands.rs` | 修改 | 向导首场景 `Full`→`Pro`（`:1128`） |
| `src-tauri/src/planner/executor.rs` | 修改 | `Full`→`Pro`（`:760`） |
| `src-tauri/src/narrative/genesis.rs` | 修改 | `Full`→`Pro`（`:582`） |
| `src-tauri/src/workflow/scheduler.rs` | 修改 | `Full`→`Pro`（`:445,543`） |
| `src-tauri/src/commands/orchestrator.rs` | 修改 | `smart_execute`（`:28`）加 `command_type` 参数，分派执行器 |
| `src-tauri/src/agents/context_optimizer.rs` | 修改 | `build_l0/build_l1` 输入从直接查 DB 改为接受 `AssetBundle` |

> v1 清单中的 `pipeline/inspector.rs`（不存在）、顶层 `commands.rs`（实为目录）、`agents/commands.rs::handleSlashSubmit`（实为前端函数）已删除/修正。

### 前端（TypeScript / React）

| 文件 | 变更 | 说明 |
|---|---|---|
| `src-frontend/src/frontstage/components/RichTextEditor.tsx` | 修改 | `handleSlashSubmit`（`:603`）扩展关键词匹配表，新增 `command_type` 透传 |
| `src-frontend/src/types/index.ts` | 修改 | 新增 `GenerationMode`/`CommandType` 类型（v1 误写 `agents/types.ts`） |
| `src-frontend/src/services/api/writing.ts` | 修改 | `autoWrite`/`autoRevise`/`smartExecute` 增加 `mode`/`command_type` 参数 |
| `src-frontend/src/frontstage/components/WenSiPanel.tsx` | 修改（可选）| Pro 模式进度提示文案 |

---

## 10. 实施阶段

### Phase 0：现状对齐与基线测量（先做，1-2 天）

- [ ] 实测 v0.12.0 下 `GenerationMode::Full` 各阶段耗时（preflight / context build / writer / inspector / rewrite），至少 3 个故事样本，产出基线表。**这是性能目标的依据，不做则目标无意义。**
- [ ] 确认 `execute_full` 中 Inspector + Rewrite 实际占比（若 < 30%，则跳过它们的收益有限，需重新评估方案）。

### Phase 1：`GenerationMode` 扩展 + 调用点迁移（2-3 天）

- [ ] `GenerationMode` 加 `Standard`，`Full`→`Pro` 别名（保证编译通过）。
- [ ] 7 处 `Full` 调用点按模块 2 表格迁移。
- [ ] 新增 `execute_standard`（先实现为「跳过 Inspector/Rewrite 的简化 Full」，AssetBundle 暂用现有 DB 查询）。
- [ ] 回归测试：`cargo test --lib`、E2E 生成流程。

### Phase 2：`AssetBundle` + Preflight 拆分（3-4 天）

- [ ] 新建 `creative_engine/asset_loader.rs`，实现 P0/P1 加载（P2/P3 可先返回 `None`）。
- [ ] 新增 `QuickPreflightChecker`。
- [ ] `build_writer_prompt` 接受 `AssetBundle`，迁移 DB 查询。
- [ ] `execute_standard` 接入 `AssetBundle::load(Standard)`。

### Phase 3：`/` 路由 + 前端适配（2 天）

- [ ] 前端 `handleSlashSubmit` 关键词表。
- [ ] 后端 `smart_execute` 加 `command_type`。
- [ ] 前端类型与 API 适配。

### Phase 4：P2/P3 资产 + Pro 模式完善（2-3 天）

- [ ] `AssetBundle` 补全 P2（审计级）+ P3（深度洞察级）加载。
- [ ] Pro 模式 Rewrite 上限强制 2 轮。
- [ ] Memory write 按模式分级（Standard 的 Semantic 降级）。

---

## 11. 风险评估

| 风险 | 等级 | 影响 | 缓解 |
|---|---|---|---|
| `build_writer_prompt` 重构引入 prompt 回归（资产注入顺序/格式变化导致生成质量波动） | **高** | 生成质量下降 | Phase 0 基线 + 每阶段 A/B 对比；保留旧 `build_writer_prompt` 为 `legacy` 函数一个版本周期 |
| Standard 模式跳过合同/Inspector，用户长期在 Standard 写作后切 Pro，Inspector 报大量问题（技术债爆发） | **中** | 用户体验差 | UI 提示「Standard 模式不保证一致性，重要章节建议用 Pro」；可选：Standard 模式定期（每 N 章）自动跑一次轻量检查 |
| `AssetBundle` 与 `ContextOptimizer` 双层抽象增加心智负担 | **中** | 维护成本 | 文档明确边界（模块 7）；命名区分（Bundle vs Optimizer） |
| `GenerationMode::Full`→`Pro` 别名遗漏迁移点 | **低** | 行为不一致 | `cargo check` 捕获；`grep -r "GenerationMode::Full"` 全量排查 |
| 前端关键词匹配覆盖不全（"往下写""接下去"等同义表达漏匹配） | **低** | 路由到兜底 WriterAgent | 兜底本身就是合理的 Pro 续写，不影响正确性；可迭代扩充词表 |
| 与 memory-everywhere（已落地）的 Inspector 第 7 维、MemoryOrchestrator 集成冲突 | **中** | 记忆系统功能退化 | 本设计不拆 Inspector 结构体（模块 5），Memory write 分级时 Semantic 降级方案需与 memory-everywhere 作者确认 |
| 与 style-continuation-v2（在途设计）的 Inspector 风格维度冲突 | **中** | 维度定义打架 | 两份设计都在改 Inspector，需统一评审（见下节） |

---

## 12. 与在途设计的协调

本设计与以下在途设计存在交集，**必须统一评审，避免互相覆盖**：

| 在途设计 | 交集点 | 协调要求 |
|---|---|---|
| **memory-everywhere**（2026-05-27，后端已落地）| Inspector 第 7 维 `memory_score`；MemoryOrchestrator 三层；Memory write 链路 | 本设计 Standard 模式的 Memory write 分级（Semantic 降级）需兼容其 MemoryOrchestrator；不拆 Inspector 结构体 |
| **style-continuation-v2**（2026-05-26，待审批）| Inspector 风格维度；StyleFingerprint 打分；StyleChecker | 本设计 Pro 模式的 StyleChecker「全维度」需包含 v2 的 fingerprint 检查；Standard 的「轻量 fingerprint」定义需与 v2 对齐 |
| **cascade-rewriter**（2026-05-24，设计阶段）| EntityMention 索引；场景级改写 | 无直接冲突，但 Cascade Rewriter 触发时应默认 Pro 模式（因其涉及多场景一致性） |

建议：三份设计合并为一次「Inspector + 资产 + 生成路径」的统一重构评审。

---

## 13. 预期效果与测量方法

### 性能预期（待 Phase 0 基线校准）

| 模式 | 重构前（v0.12.0 Full） | 重构后预期 | 测量方法 |
|---|---|---|---|
| Standard（普通生成 / auto_write） | 待测 | 较 Full 缩短 40-60% | 跳过 Inspector + Rewrite + auto_contract，主要省这几项 |
| Pro（/ 指令 / 向导 / Genesis） | 待测 | 与 Full 持平或略增（资产加载更全） | 质量提升，耗时不变 |
| Fast（Ghost Text） | 不变 | 不变 | 不改动 |

> **v1 称"加速 5-10x"缺乏依据，本版不承诺具体倍数**。实际加速取决于 Inspector + Rewrite + auto_contract 在 Full 总耗时中的占比，Phase 0 实测后才能确定。若实测占比 < 30%，本设计的边际收益有限，需重新评估是否值得重构。

### 质量预期

| 模式 | 质量定位 | 验证方式 |
|---|---|---|
| Standard | 「人类水平一次写对」，靠资产约束而非事后审计 | A/B 对比：同一 prompt，Standard vs Pro，人工盲评可接受度 |
| Pro | 专业水准，7 维 Inspector 保证 | 现有 Full 质量基线不退化 |

---

## 14. 待决问题（需评审时定夺）

1. **`SubscriptionTier::Free` 用户能否触发 `GenerationMode::Pro`？** 默认允许（Pro 不额外消耗配额），但产品可能想限制。需产品确认。
2. **Standard 模式是否定期自动跑轻量检查？**（防技术债爆发）若要，频率与触发条件？
3. **`AssetBundle` 的 P3「按需加载」触发条件？** 是每次 Pro 生成都加载，还是 Inspector 评分低于阈值时才补加载？后者更省但增加复杂度。
4. **现有 `GenerationMode::Full` 别名保留几个版本周期？** 建议 1 个（下个 minor 版本移除）。
5. **Standard 模式的 Memory write「Semantic 降级」具体降级到什么程度？** 完全不写 Semantic，还是写一个简化版？需与 memory-everywhere 设计者确认。

---

*本文档基于对 `src-tauri/src/`（agents/、pipeline/、story_system/、prompts/、commands/、creative_engine/、memory/）与 `src-frontend/src/`（frontstage/components/、services/api/、types/）的逐行核查编写。所有文件路径与行号均经核实。*
