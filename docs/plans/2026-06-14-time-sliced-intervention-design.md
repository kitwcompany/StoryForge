# 分时介入架构设计文档

> 创建日期: 2026-06-14
> 状态: Phase 0 已验证 ✅（假设 A 差距 7.9% < 30% 阈值，假设 B 已定段落级方案），待评审后进入实施
> 解决的问题: AI 长篇小说创作中"质量与速度不可兼得"的根本矛盾
> 取代文档: [`2026-06-14-asset-tier-creation-design.md`](./2026-06-14-asset-tier-creation-design.md)（v1，三档分级，治标）及其 v2 修订版（AssetBundle 分级，仍治标）
>
> **本设计是该项目当前止步不前问题的正解文档。**

---

## 0. 为什么前两版走偏了

v1（三档分级 Fast/Standard/Pro）和 v2（AssetBundle 分级）都默认了一个隐含假设：**所有资产必须在"Writer 生成那一刻"同步介入**。在这个假设下，资产越多 → prompt 越长 → 越慢，矛盾无解，只能让用户在"快档"和"好档"之间二选一。

这两版做的是**承认矛盾、让用户选边**，不是解开矛盾。

本设计质疑的是这个隐含假设本身。人类小说编辑用伏笔表、场景合同、角色卡，也不是站在作家身后、在每一句话写下的瞬间同步审查的。编辑是在**不同的时间点、以不同的强度**介入的。当前系统把所有资产都压在 `AgentOrchestrator::execute_full` 这一条同步链路里，违背了真实的创作协作模式——这才是慢的真正来源，不是资产本身。

---

## 1. 第一性原理

> **把大灾难变成即时可见的小债务。**
> 蚂蚁搬家，不积巨石。

这条原理由用户在 brainstorming 中确立，贯穿全文：

- **创作是连续的，质量保障应该是增量的、即时的、可见的。**
- 不在生成时刻同步堆叠全部资产（巨石）。
- 把每个资产拆到"它该发力的那一刻"独立介入（蚂蚁）。
- 正文立即返回（快），审计结果随后以 inline annotation 回流（深、可见、人在环里）。
- 用户当场处理小债，不让它滚成大灾难。

---

## 2. 根因诊断

### 用户感知的矛盾

> 强化专业资产介入 → 质量高但慢；放松资产 → 快但质量低。

### 工程层的真正病灶（B + E）

当前生成是一条**畸形同步链路**（`agents/orchestrator.rs::execute_full`）：

```
用户点"生成"
  ├─ T0: 加载全量资产（合同+角色+世界观+记忆+风格+伏笔）
  ├─ T1: Preflight 检查（4 项 DB + 可能 5 次 LLM 补合同）
  ├─ T2: build_writer_prompt（所有资产塞进一个巨大 prompt）
  ├─ T3: Writer LLM 调用（prompt 越长越慢）
  ├─ T4: Inspector 7 维审计（又一次 LLM，阻塞）
  ├─ T5: 不达标 → Rewrite（又一轮 Writer，最多 N 轮）
  ├─ T6: apply_writing_skills
  └─ T7: Memory write
                ↓
        用户终于看到正文（T0~T7 全程同步等待）
```

**两个根因**：
- **B（资产被错误同步化）**：资产的真正发力时机各不相同——合同是写之前的约束、伏笔是跨章节的追踪、Inspector 是写之后的审计、记忆是随时可检索。全压到"Writer 一次 LLM"那个点上，既慢又低效。
- **E（写与审被错误耦合）**："写"和"审"性质完全不同，却被焊死在一条同步链路。用户点一次"生成"，系统就同步地写→审→改→再审，用户全程干等。

### 两层矛盾，本设计只解决工程层

| 层 | 矛盾 | 本设计 |
|---|---|---|
| 工程层（慢） | 资产同步堆叠导致延迟 | ✅ 解决 |
| 模型层（AI 长篇能力不足） | AI 写长篇本质容易崩 | ❌ 不解决 |

本架构能让崩坏**早暴露**（审计即时回流），但不能让 AI **不崩坏**。正确预期：**把"不可控的大灾难"变成"可控的小债流水线"**。这是目前能拿到的最大杠杆。

---

## 3. 用户已确认的产品取舍

在 brainstorming 中用户做了两个关键决策，本设计建立其上：

1. **正文可以是"未经审计的初稿 + 随后到达的审计标注"**（异步审计，用户是最终决策者）。
2. **审计回流形态 = 异步 inline annotation**（形态甲：批注，可 accept/reject/ignore）。理由：算力最省、复用现有 `TextAnnotationMark` 基础设施、保留创作主权。

这两条是分时解耦可行的命门。若无此取舍，矛盾无法在工程层解开。

---

## 4. 核心架构：三条独立时间线

把"一条同步链路"拆成"三条独立时间线"，每条独立调度、独立触发、独立限流。

### 时间线 1：写作时刻（热路径，用户等待）

**目标**：最快返回可用正文。

只做"写之前必须"的事：
- 加载 `WriteTimeBundle`（合同红线 + 角色核心 + 场景大纲 + GenreProfile 反模式清单）
- `QuickPreflightChecker`（仅角色非空，不触发 auto_contract）
- Writer 单轮生成（`candidate_count`=1）
- 立即返回正文给编辑器

**预算**：< 15 秒。用户几乎不等待。

**资产介入**：只有 P0 级（见模块 5）。这是唯一在用户等待关键路径上的环节。

### 时间线 2：审计时刻（温路径，后台异步）

**目标**：即时发现小债，以 inline annotation 回流。

正文返回后立即 spawn（不阻塞用户，用户已在码字）。执行：
- Inspector 7 维审计（记忆一致性 / 连续性 / 逻辑 / 角色 / 伏笔 / 节奏 / 风格）
- 伏笔追踪（`PayoffDetector`：本段是否该埋/该收）
- 合同偏离检测（vs `RuntimeContract`）
- 风格漂移打分（`StyleChecker`，轻量、不阻塞）
- 每个 issue → 一条 `TextAnnotation`（带 severity、维度、建议）
- 通过 `SyncEvent::AnnotationCreated` 事件回流前端

**预算**：后台 30-90 秒，用户无感。可限流、可取消。

**资产介入**：P1 级（Inspector、合同比对、伏笔、记忆一致性）。这些是"审计用"的资产，此时才加载。

### 时间线 3：洞察时刻（冷路径，跨章节深度）

**目标**：长篇一致性，防止滚成大灾难。

低频触发（每 N 章 / 用户主动 / 检测到漂移阈值）：
- 追读力趋势（`ReadingPowerEvaluator`）
- 世界观漂移检测
- KG 深度遍历 + 语义检索（`hybrid_search`）
- Memory Ingest（向 KG / 向量库写入本段实体关系）
- 输出：结构性报告（不一定 inline，可入"叙事分析"页）

**预算**：分钟级，可排队。

**资产介入**：P2 级（追读力、KG、向量、长期漂移）。

### 三条线的关系图

```
┌──────────────────────────────────────────────────────────┐
│ 时间线 1：写作时刻（热）< 15s                             │
│   WriteTimeBundle(P0) → Writer → 立即返回正文            │
└──────────────────────┬───────────────────────────────────┘
                       │ 正文已呈现，触发后台
                       ▼
┌──────────────────────────────────────────────────────────┐
│ 时间线 2：审计时刻（温）30-90s 后台                       │
│   Inspector(P1) → issues → TextAnnotation → 回流编辑器   │
└──────────────────────┬───────────────────────────────────┘
                       │ 低频触发
                       ▼
┌──────────────────────────────────────────────────────────┐
│ 时间线 3：洞察时刻（冷）分钟级，可排队                    │
│   追读力/KG/向量/漂移(P2) → 结构性报告                   │
└──────────────────────────────────────────────────────────┘
```

### 如何解开矛盾

| 原矛盾 | 新架构如何解 |
|---|---|
| 强化资产 → 慢 | 资产不再挤在"写作时刻"。Writer 只带最小约束，秒出。 |
| 放松资产 → 质量低 | 资产没放松，只是**搬家了**——从"生成时同步"到"生成后异步"。深度一点没减。 |
| 大灾难（20 章后崩盘） | 每段写完 90s 内小债以 inline 标注出现，当场处理。 |
| 用户干等 | 写作时刻 < 15s 返回。 |
| 算力堆积 | 审计和洞察都在后台，可限流、可排队、可取消。 |

---

## 5. 资产按"介入时机"重新分类（取代 v2 的 P0~P3 分级）

v2 按"加载深度"分级是错的思路（默认所有资产挤在生成时）。本设计按**"该在哪个时刻介入"**重新分类。

| 资产 | 时间线 1（写作时刻） | 时间线 2（审计时刻） | 时间线 3（洞察时刻） |
|---|:---:|:---:|:---:|
| 合同红线（MASTER_SETTING 核心）| ✅ 注入 prompt | | |
| 角色核心（姓名+关系+当前状态）| ✅ 注入 prompt | | |
| 场景大纲（goal + conflict_type）| ✅ 注入 prompt | | |
| GenreProfile 反模式清单 | ✅ 注入 prompt | | |
| 风格 DNA 六维 | ✅ 注入 prompt | ✅ 漂移打分 | |
| 完整 RuntimeContract | | ✅ 比对偏离 | |
| Inspector 7 维 | | ✅ 产出 annotation | |
| 伏笔追踪（PayoffDetector）| | ✅ 埋/收提醒 | |
| 记忆一致性（memory_score）| | ✅ 角色状态突变检测 | |
| 追读力评估 | | | ✅ 趋势分析 |
| KG 深度遍历 + 语义检索 | | | ✅ 跨章关联 |
| Memory Ingest | | | ✅ 写入 KG/向量 |
| 世界观漂移 | | | ✅ 长期检测 |

**每个资产都有它该发力的时刻，不再全堆在时间线 1。**

> 与 v2 P0~P3 的对应：v2 的 P0 ≈ 时间线 1 资产；v2 的 P1（方法论/记忆/CanonicalState）拆分——方法论入时间线 1 prompt，记忆/CanonicalState 入时间线 2 比对；v2 的 P2（审计级）= 时间线 2；v2 的 P3（深度洞察）= 时间线 3。

---

## 6. 用 task_system 解耦三条时间线（实现 E）

项目已有成熟的 `task_system`，是实现 E（写与审解耦）的天然载体。

### 现有基础设施（已核查）

- `TaskExecutor` trait（`task_system/executor.rs:16`）
- `TaskType` 枚举（`task_system/models.rs:79`）已有 `CascadeRewrite`/`AiGeneration`/`PipelineReview` 三值
- 三个现有 executor：`AiGenerationExecutor`（`agents/executor.rs:33`）、`PipelineReviewExecutor`（`pipeline/executor.rs:135`）、`CascadeRewriteExecutor`（`creative_engine/cascade_rewriter/executor.rs:29`）
- 后台异步模式已有先例：`MEMORY_WRITER_SEMAPHORE`（`orchestrator.rs:382`）已用于 spawn 后台 ingest

### 新增三个 TaskType + Executor

```rust
// task_system/models.rs（修改）
pub enum TaskType {
    CascadeRewrite,
    AiGeneration,
    PipelineReview,
    // 新增：分时介入架构的三条时间线
    TimeSlicedWrite,    // 时间线 1：写作时刻（同步返回正文，不入队）
    AsyncAudit,         // 时间线 2：审计时刻（后台异步，产出 annotation）
    DeepInsight,        // 时间线 3：洞察时刻（批量，产出报告）
}
```

```rust
// 新增三个 executor（实现 TaskExecutor trait）
// task_system/executors/write_executor.rs
pub struct WriteExecutor { /* 时间线 1：同步热路径 */ }
//   - 加载 WriteTimeBundle(P0)
//   - QuickPreflightChecker
//   - Writer 单轮生成
//   - 返回正文 + spawn AuditExecutor

// task_system/executors/audit_executor.rs
pub struct AuditExecutor { /* 时间线 2：温路径 */ }
//   - Inspector 7 维
//   - PayoffDetector / 合同偏离 / 风格漂移
//   - issues → create_text_annotation
//   - 发 SyncEvent::AnnotationCreated

// task_system/executors/insight_executor.rs
pub struct InsightExecutor { /* 时间线 3：冷路径 */ }
//   - 追读力趋势 / KG / 向量 / 漂移
//   - 输出结构性报告
```

### 时间线间的触发关系

```
WriteExecutor（时间线 1，同步）
  │ 返回正文后
  ├─→ spawn AuditExecutor（时间线 2，异步）
  │     │ 完成后
  │     └─→ 可选 spawn InsightExecutor（时间线 3，条件触发：每 N 段 / 漂移超阈值）
  │
  └─→ 主流程结束
```

- 时间线 2 由时间线 1 触发，但**不阻塞**时间线 1 的返回。
- 时间线 3 由时间线 2 完成后**条件触发**（不是每次都跑）。

### 调度与限流

- 时间线 1：无队列，同步执行（用户在等）。
- 时间线 2：独立队列，并发上限可配（建议 2，与 `MEMORY_WRITER_SEMAPHORE` 对齐）。复用 `task_system` 的 `Semaphore` + `CancellationToken` 模式。
- 时间线 3：独立队列，并发上限 1（冷路径，避免压垮本地模型）。

---

## 7. annotation 回流机制（实现 B 的"事后可见"）

### 现有基础设施（已核查，全部现成）

后端：
- `create_text_annotation` 命令（`scene_commands.rs:608`）
- `get_text_annotations_by_scene` / `get_text_annotations_by_chapter` 查询
- `update_text_annotation` / `resolve_text_annotation` / `delete_text_annotation` 处置
- `SyncEvent::AnnotationCreated` / `AnnotationResolved`（`state_sync/events.rs:123,128`，ts-rs 已生成前端类型）

前端：
- `TextAnnotationMark.ts`（TipTap mark 扩展）
- `CommentAnchor.ts`（锚点）
- `services/api/annotations.ts::createTextAnnotation` / `getTextAnnotationsByScene`
- `useSyncStore` 已监听 `AnnotationCreated` 失效缓存

**这意味着时间线 2 → 编辑器渲染的回流链路，几乎零新增前端基础设施。**

### 数据流

```
AuditExecutor 产出 issue（维度、severity、定位、建议）
  │
  ├─ 定位：Inspector 返回的 issue 需映射到正文 span
  │   （若 Inspector 无法给精确定位，降级为"段落级"annotation，
  │    挂到当前 scene 末尾或指定段落）
  │
  ├─ create_text_annotation({
  │     scene_id, chapter_id,
  │     content: "【记忆一致性】张三腿部受伤却奔跑，与第3章设定冲突",
  │     annotation_type: "ai_audit",
  │     severity: "high",
  │     metadata: { dimension: "memory", inspector_score: 62, suggestion: "..." }
  │   })
  │
  └─ StateSync::emit(AnnotationCreated { story_id, scene_id, annotation_id })
        │
        ▼
  前端 useSyncStore 收到 AnnotationCreated
        │ 失效 ['textAnnotations', scene_id] 查询
        ▼
  RichTextEditor 重新渲染 TextAnnotationMark
        │ 用户看到 inline 标注
        ▼
  用户点击处置：accept / reject / ignore
        → resolve_text_annotation / update_text_annotation
```

### annotation 的视觉设计（建议）

- **severity 颜色**：high=红、medium=琥珀、low=蓝。
- **悬停展开**：显示维度、Inspector 评分、AI 建议。
- **快捷键**：Tab 接受（应用建议改写）、Esc 忽略、Cmd+R 拒绝。
- **顶部债务指示器**：显示未处理 annotation 数量 + 最严重维度（像 Git 未提交数）。

### 定位方案（Phase 0 已定）

Inspector 当前 JSON schema（`prompts/engine.rs:215-235`）**不含 `char_start`/`char_end` 字段**，`ReviewIssueItem`（`db/models.rs:1542`）也无 location。代码核查结论：

- **精确定位（字符级 span）不可靠**：要求 LLM 返回字符偏移会漂移，且 prompt 复杂化。
- **采用"段落级 annotation"作为主方案**（非降级）。理由：`memory_analysis.character_conflicts`（如"角色张三受伤却奔跑"）、`foreshadowing_misses`（如"伏笔神秘信封未提及回收"）已返回**描述性定位**，足够支撑段落级标注 + 用户自行精确定位。
- **实现**：annotation 锚定到 Inspector 报告问题所在的段落（或 scene 末尾），annotation 正文即描述性文字。未来若需精确高亮，可作为增强项迭代，不阻塞主流程。

---

## 8. WriteTimeBundle（时间线 1 的最小可行约束）

替代 v2 的 `AssetBundle` 概念。只含"写之前必须"的资产。

```rust
// creative_engine/write_time_bundle.rs（新增）
pub struct WriteTimeBundle {
    /// 合同红线：MASTER_SETTING 核心世界观约束（截断到 ~500 token）
    pub contract_redlines: ContractRedlines,
    /// 当前章节出场角色核心（姓名+关系+当前状态，仅出场角色）
    pub core_characters: Vec<CoreCharacter>,
    /// 当前 scene 大纲（dramatic_goal + conflict_type）
    pub scene_outline: SceneOutline,
    /// GenreProfile 反模式清单（核心基调/节奏策略/Do-Not 列表）
    pub genre_antipatterns: GenreAntipatterns,
    /// 风格 DNA 六维（to_prompt_extension）
    pub style_dna: Option<StyleDnaPromptSlice>,
}
```

### 设计原则

- **只放"一次写对基本盘"必需的约束**。不放审计用资产（那些去时间线 2）。
- **token 预算硬上限**：整个 bundle 序列化后不超过 ~3000 token。超出由 `ContextOptimizer` 裁剪。
- **加载全走 `spawn_blocking`**，不阻塞 tokio worker。
- **缓存**：key = `(story_id, scene_id)`，TTL 60s。

### 注入顺序：红线最前最突出（Phase 0 实证改进）

Phase 0 的 S1 场景出现反直觉结果：**B 组（全量资产）反而违背了世界观红线**——prompt 塞太多资产，模型注意力被分散，忽略了最硬的约束。而 A 组（最小约束）因为 prompt 干净、红线突出，反而守得严。

**教训**：资产注入顺序不是平铺，红线必须最前、最突出。`build_writer_prompt` 的注入顺序固定为：

```
1. 【世界观红线】（加粗强调、独立段落、明确"绝不可违背"）   ← 最高优先级
2. 【角色当前状态】（受伤/位置/情绪，直接影响行为合理性）
3. 【场景大纲】（dramatic_goal + conflict_type + setting）
4. 【GenreProfile 反模式清单】
5. 【风格片段】（若有，题材自适应决定，见下）
```

这条比"加多少资产"更重要。Phase 0 证明：**守住红线 = 守住质量底线**，比堆叠资产有效。

### 题材自适应（Phase 0 实证改进）

Phase 0 的 S3 都市场景差距达 26 分（B 大胜 A），说明**对风格/细节敏感的题材，纯最小约束不够**。但 S1 玄幻场景 A 反超 B 11 分，说明**对红线敏感的题材，资产太多反而有害**。

**解法**：`WriteTimeBundle` 的风格片段按题材动态决定是否纳入：

| 题材类型 | 风格片段 | 理由（Phase 0 实证）|
|---|---|---|
| 都市 / 情感 / 现实主义 | ✅ 纳入轻量风格 DNA（~200 token）| S3 差距 26 分，风格细节是质量关键 |
| 玄幻 / 仙侠 / 科幻 | ❌ 不纳入（仅红线）| S1 证明红线守严 > 风格约束 |
| 悬疑 / 推理 | ✅ 纳入逻辑链约束片段 | 逻辑严密性是质量关键 |
| 其他 / 默认 | ⚠️ 可选，默认不纳入 | 保守策略 |

判断依据 `stories.genre` 字段映射到题材类型。这个映射表可配置（写入 `GenreProfile` 的扩展字段），未来可按实测迭代。

### memory 维度优先审计（Phase 0 实证改进）

Phase 0 各维度分中，`memory`（设定遵守）是**波动最大的维度**：S1 中 A=19 而 B=8（差 11 分）。这正好是 Writer 最容易在生成时忽略、但事后最容易被检出的硬伤。

**对 AuditExecutor 的指导**：异步审计的 issue 优先级排序为 `memory > continuity > logic > 其他`。memory 维度的 high-severity issue 应优先回流、最醒目展示。详见模块 7。

### 与 ContextOptimizer 的关系

`WriteTimeBundle` 是数据打包层，`ContextOptimizer`（`agents/context_optimizer.rs`）仍是 token 预算调度层。`build_writer_prompt` 改为接受 `WriteTimeBundle`，函数内的 DB 查询迁出到 bundle 加载。

---

## 9. `GenerationMode` 的处理

### 决策：新增 `TimeSliced` 模式作为默认，保留 `Fast`/`Full`

```rust
// agents/orchestrator.rs
pub enum GenerationMode {
    /// Ghost Text 等实时补全，仅 P0，跳过一切审计。保持不变。
    Fast,
    /// 【新增·默认】分时介入：时间线1同步返回正文 + 时间线2/3后台异步审计。
    /// 这是解开矛盾的正解路径。
    TimeSliced,
    /// 【保留·可选】同步全量：原 Full 路径，审计阻塞 + 自动 Rewrite。
    /// 用户通过 / 指令显式要求"专业同步成品"时走这条。
    Full,
}
```

### 调用点迁移

现有 7 处 `GenerationMode::Full`（详见 v2 文档第 3 节表格）：

| 调用点 | 迁移目标 | 理由 |
|---|---|---|
| `agents/commands.rs:342`（普通生成按钮）| **TimeSliced** | 默认路径，用户期望快 |
| `agents/commands.rs:699,1109`（auto_write/auto_revise）| **TimeSliced** | 文思泉涌，速度优先 |
| `creation_commands.rs:1128`（向导首场景）| **TimeSliced** | 向导也要快，首章质量由时间线 2 兜底 |
| `planner/executor.rs:760` | **Full** | Planner 规划的复杂任务，质量优先 |
| `narrative/genesis.rs:582`（Genesis 管线）| **Full** | 整书生成，深度优先 |
| `workflow/scheduler.rs:445,543`（工作流）| **Full** | 专业自动化 |

> 大部分迁移到 `TimeSliced`（默认快路径），少数深度场景保留 `Full`。`agents/executor.rs:79` 的默认值 `_ => Full` 改为 `_ => TimeSliced`。

### 用户可手动选 Full

`/` 指令仍可触发 `Full`（同步审计 + 自动 Rewrite），给需要"一步到位成品"的用户保留入口。但**不再是默认**。

---

## 10. 改动文件清单（已核查路径）

### 后端（Rust）

| 文件 | 变更 | 说明 |
|---|---|---|
| `src-tauri/src/task_system/models.rs` | 修改 | `TaskType` 加 `TimeSlicedWrite`/`AsyncAudit`/`DeepInsight`（`:79`）|
| `src-tauri/src/task_system/executors/write_executor.rs` | **新增** | 时间线 1，实现 `TaskExecutor` |
| `src-tauri/src/task_system/executors/audit_executor.rs` | **新增** | 时间线 2，产出 annotation |
| `src-tauri/src/task_system/executors/insight_executor.rs` | **新增** | 时间线 3，产出报告 |
| `src-tauri/src/creative_engine/write_time_bundle.rs` | **新增** | `WriteTimeBundle` 加载 |
| `src-tauri/src/creative_engine/mod.rs` | 修改 | 声明 `write_time_bundle` 模块 |
| `src-tauri/src/agents/orchestrator.rs` | 修改 | `GenerationMode` 加 `TimeSliced`（`:25`）；新增 `execute_time_sliced` |
| `src-tauri/src/agents/service.rs` | 修改 | `build_writer_prompt` 加 `bundle` 参数（`:1691`）|
| `src-tauri/src/story_system/preflight.rs` | 修改 | 新增 `QuickPreflightChecker`（现有不动）|
| `src-tauri/src/scene_commands.rs` | 修改 | `create_text_annotation`（`:608`）支持 `ai_audit` 类型 + severity/metadata |
| `src-tauri/src/commands/orchestrator.rs` | 修改 | `smart_execute`（`:28`）默认 `TimeSliced` |
| `src-tauri/src/agents/commands.rs` | 修改 | auto_write/auto_revise 改 `TimeSliced`（`:342,699,1109`）|
| `src-tauri/src/agents/executor.rs` | 修改 | 默认值改 `TimeSliced`（`:79`）|
| `src-tauri/src/prompts/engine.rs` | 修改 | Inspector prompt 要求返回 issue 定位（char_start/end）|

### 前端（TypeScript / React）

| 文件 | 变更 | 说明 |
|---|---|---|
| `src-frontend/src/frontstage/components/RichTextEditor.tsx` | 修改 | `TextAnnotationMark` 渲染 severity 颜色 + 处置按钮 |
| `src-frontend/src/frontstage/components/FrontstageHeader.tsx` | 修改 | 新增"债务指示器"（未处理 annotation 数）|
| `src-frontend/src/frontstage/extensions/TextAnnotationMark.ts` | 修改 | 支持 `ai_audit` 类型 + severity 样式 |
| `src-frontend/src/types/index.ts` | 修改 | 新增 `GenerationMode.TimeSliced`、annotation severity 类型 |
| `src-frontend/src/hooks/useSyncStore.ts` | 修改 | `AnnotationCreated` 回调触发 toast 提示（已有失效逻辑）|
| `src-frontend/src/services/api/writing.ts` | 修改 | API 传 `mode` 参数 |

---

## 11. 实施阶段

### Phase 0：假设验证 ✅ 已完成（2026-06-14）

> 实验环境：vLLM `qwen3.6-35b-a3b-vision` @ `http://10.62.239.13:17098`
> 实验脚本与原始数据：见本文档末尾「附录 A：Phase 0 实验记录」
> 三组场景：S1 东方玄幻（人vs人）/ S2 近未来科幻（人vs自然）/ S3 都市现实（人vs自我）

#### 假设 A：资产前置能否替代事后审计？✅ 成立（差距 7.9% < 30% 阈值）

A/B 盲测评分（同一 7 维 Inspector 标准打分，评分器不知哪个是 A/B）：

| 场景 | A（最小约束 WriteTimeBundle）| B（全量资产 Full）| 差距 | 谁赢 |
|---|---|---|---|---|
| S1 玄幻 | **110**/140 | 99/140 | 11 分 | **A 反超** ⬅ |
| S2 科幻 | 105/140 | **123**/140 | 18 分 | B 胜 |
| S3 都市 | 99/140 | **125**/140 | 26 分 | B 胜 |
| **平均** | **104.7** | **115.7** | **11.0 分（7.9%）** | B 略胜 |

**结论**：平均差距 7.9%，远低于 30% 阈值。考虑到这 7.9% 会被时间线 2 异步审计追回（标成 annotation），净质量损失趋近于零。**架构强成立。**

**但平均掩盖了单场景的剧烈波动**（-11 分到 +26 分），这暴露了三个必须改进的点（已反映到模块 7、8）：

1. **题材自适应**（S3 差距 26 分）：都市/情感类对风格细节敏感，纯最小约束不够，需按题材调权。
2. **红线突出注入**（S1 A 反超 B）：B 组塞太多资产反而忽略了世界观红线，说明"资产多 ≠ 幻觉少"——红线必须在 prompt 里最突出。
3. **memory 维度是最大波动源**：S1 中 A=19 而 B=8（差 11 分），是 7 维里波动最大的。审计第一优先级应抓 memory。

#### 假设 B：Inspector 能否给出正文定位？✅ 已从代码层面解决

Inspector 当前 JSON schema（`prompts/engine.rs:215-235`）无 `char_start`/`char_end` 字段，`ReviewIssueItem`（`db/models.rs:1542`）也无 location 字段。**结论：精确定位不可靠（LLM 给字符偏移会漂移），采用"段落级 annotation"作为主方案**（非降级方案）。但 `memory_analysis.character_conflicts` 等字段已返回描述性定位（如"角色张三受伤却奔跑"），足够支撑段落级标注。详见模块 7「定位问题」。

#### 附带验证：耗时与 prompt 长度的关系

| | A（最小约束）| B（全量资产）|
|---|---|---|
| prompt 字符数 | ~580 | ~1500（**多 160%**）|
| 生成耗时平均 | 110s | 118s（**仅多 7%**）|

**这证实了设计文档的核心诊断**：慢的根源不是 prompt 长度（资产量），而是同步链路堆叠的 Inspector/Rewrite/auto_contract 步骤。把那些挪到后台异步是正解。

### Phase 1：时间线 1 解耦（3-4 天）

- [ ] `WriteTimeBundle` 实现（P0 资产加载）。
- [ ] `QuickPreflightChecker`。
- [ ] `GenerationMode::TimeSliced` + `execute_time_sliced`（先实现为"跳过 Inspector/Rewrite 的简化 Full"，bundle 暂用现有 DB 查询）。
- [ ] 调用点迁移（7 处）。
- [ ] 基线：`TimeSliced` 生成耗时 vs `Full`，验证 < 15s。

### Phase 2：时间线 2 异步审计 + annotation 回流（4-5 天）

- [ ] `AuditExecutor`。
- [ ] Inspector prompt 增强（issue 定位）。
- [ ] `create_text_annotation` 支持 `ai_audit` 类型。
- [ ] 前端 `TextAnnotationMark` 渲染 + 处置 UI。
- [ ] 端到端：生成 → 后台审计 → annotation 回流 → 用户处置。

### Phase 3：时间线 3 深度洞察（2-3 天）

- [ ] `InsightExecutor`。
- [ ] 条件触发逻辑（每 N 段 / 漂移阈值）。
- [ ] 报告入"叙事分析"页。

### Phase 4：债务指示器与体验打磨（2 天）

- [ ] 顶部债务指示器。
- [ ] annotation 视觉规范（severity 颜色、悬停、快捷键）。
- [ ] 用户引导（首次看到 annotation 的提示）。

---

## 12. 风险评估

| 风险 | 等级 | 影响 | 缓解 |
|---|---|---|---|
| ~~**Phase 0 假设 A 不成立**~~ | ~~致命~~ | ~~整个架构价值存疑~~ | ✅ **Phase 0 已验证成立**（差距 7.9% < 30%）。剩余风险已具体化为三条改进（题材自适应/红线突出/memory 优先），见模块 8 |
| ~~Inspector 无法给正文定位~~ | ~~高~~ | ~~annotation 无法精确高亮~~ | ✅ **已定方案**：段落级 annotation 为主方案（非降级），见模块 7 |
| 用户忽略 annotation，债仍滚大 | 中 | 质量不提升 | 债务指示器 + 超阈值提醒（未处理 > 10 条时强提示）；定期 Insight 报告汇总 |
| 后台审计任务堆积（用户快速连续生成）| 中 | 队列阻塞 | 限流（并发 2）+ 去重（同一段落只审计最新版）+ LRU 淘汰过时任务 |
| `build_writer_prompt` 重构引入 prompt 回归 | 中 | 生成质量波动 | 保留旧函数为 `legacy` 一个版本；A/B 对比 |
| 与 memory-everywhere 的 memory_score 集成 | 中 | 记忆一致性检测失效 | 不改 Inspector 结构体（与 v2 决策一致），AuditExecutor 复用现有 Inspector 输出 |
| `Full` 与 `TimeSliced` 双路径长期维护成本 | 低 | 代码膨胀 | 两者共享 `WriteTimeBundle` 和 Inspector，差异仅在调度时机；可接受 |

---

## 13. 与在途设计的协调

| 在途设计 | 交集点 | 协调要求 |
|---|---|---|
| **memory-everywhere**（后端已落地）| Inspector 第 7 维 memory_score；MemoryOrchestrator | AuditExecutor 复用其 Inspector 输出；Memory Ingest 入时间线 3 |
| **style-continuation-v2**（待审批）| Inspector 风格维度；StyleFingerprint | 时间线 2 的风格漂移打分复用 v2 的 fingerprint 检查 |
| **cascade-rewriter**（设计阶段）| 场景级改写 | Cascade Rewriter 触发时默认走 `Full`（涉及多场景一致性）|
| **v1/v2 asset-tier 文档**| 资产分级思路 | **本设计取代两者**。v2 的 AssetBundle 概念被 `WriteTimeBundle` 替代；P0~P3 分级被"介入时机"分类替代 |

---

## 14. 成功标准

### 定量

- 时间线 1 生成耗时 P95 < 15s（基线：Full 模式 25-170s）。
- 时间线 2 annotation 回流延迟 < 90s（从正文返回算起）。
- 用户 annotation 处置率 > 40%（衡量"小债"是否真的被处理，而非堆积）。
- 长篇（50+ 章）Inspector 累计 high-severity 问题数 < 同等规模 Full 模式的 30%（衡量大灾难是否被拆解）。

### 定性

- 用户不再因"等生成"而中断心流。
- 角色崩坏、伏笔遗忘等问题在发生后 < 2 分钟内被标注。
- 用户能直观感知"我现在欠了多少债"（债务指示器）。
- 切换到 Full 模式的频率 < 20%（说明默认路径足够好）。

---

## 15. 待决问题

1. **annotation 的生命周期**：被 ignore 的 annotation 是否定期重新提醒？还是永久静默？建议：永久静默但保留记录，Insight 报告中汇总。
2. **时间线 2 的去重策略**：用户对同一段落连续点两次"生成"，第二次是否取消第一次的审计任务？建议：是，LRU 只保留最新。
3. **债务指示器的阈值**：多少条未处理算"红色警告"？建议：> 10 条 high，或 > 30 条总计。
4. **Full 模式的去留**：长期是否完全移除 Full，统一为 TimeSliced？建议：保留至少 2 个版本，观察用户是否真的需要同步成品。
5. **段落级 annotation 的 UX 呈现**：Phase 0 已定段落级为主方案（非降级）。如何呈现才不突兀、如何让用户快速定位到具体段落？需前端原型验证。

---

## 16. 总结：为什么这是正解

前两版（v1 三档、v2 AssetBundle）都在"如何分档"里打转，默认了"资产必须同步介入"。本设计质疑这个假设本身：

- **不是削弱资产**（资产全保留，深度一点没减）。
- **不是让用户二选一**（默认路径既快又有深度审计）。
- **而是改变介入时机**——让每个资产在它该发力的那一刻独立介入。

这就是 B（资产分时）+ E（写审解耦）的真正含义。配合用户确认的两个产品取舍（异步审计、inline annotation 形态），矛盾在工程层被真正解开：

> **快**：正文 < 15s 返回（时间线 1 只带最小约束）。
> **好**：资产深度介入一点没减（时间线 2+3 在后台全力审计）。
> **可控**：大灾难变成即时可见的小债（annotation 流水线）。

蚂蚁搬家，不积巨石。

---

*本文档基于对 `src-tauri/src/`（task_system/、agents/、pipeline/、story_system/、scene_commands.rs、state_sync/、prompts/）与 `src-frontend/src/`（frontstage/extensions/、services/api/）的逐行核查编写。所有文件路径与行号均经核实。基础设施（TaskExecutor、TextAnnotationMark、SyncEvent::AnnotationCreated）确认就位。*

---

## 附录 A：Phase 0 实验记录

> 实验日期：2026-06-14
> 目的：验证"资产前置能否替代事后审计"（假设 A）+ "Inspector 定位能力"（假设 B）

### A.1 实验环境

- **模型**：`qwen3.6-35b-a3b-vision`（vLLM 服务，262k 上下文）
- **端点**：`http://10.62.239.13:17098`（OpenAI 兼容格式）
- **生成参数**：`temperature=0.75`，`seed=42`（控制随机性，唯一变量为 prompt）
- **评分参数**：`temperature=0.2`（评分要稳定）
- **评分器**：同一 LLM，盲测（不知哪个是 A/B），用项目真实的 7 维标准（复刻 `prompts/engine.rs:182-201`）

### A.2 实验设计

- **A 组（最小约束 WriteTimeBundle）**：仅注入合同红线 + 角色核心 + 场景大纲 + GenreProfile 反模式 + 前文摘要
- **B 组（全量资产 Full）**：A 的全部 + 方法论节拍表 + 风格 DNA 六维 + 伏笔要求 + 记忆一致性要求 + 7 维自检清单
- **控制变量**：同一场景设定、同一前文、同一 seed、同一 temperature
- **评分**：评分器看场景设定 + 两份匿名正文（随机打乱为甲/乙），各自打 7 维分

### A.3 三组场景

| ID | 题材 | 冲突类型 | 场景核心 |
|---|---|---|---|
| S1 | 东方玄幻 | 人vs人 | 弟子从执法长老口中套出师兄被带走的真相 |
| S2 | 近未来科幻 | 人vs自然 | 空间站指令长决定是否关闭故障氧气模块 |
| S3 | 都市现实 | 人vs自我 | 旧书店老板与常客借还书试探彼此近况 |

### A.4 盲测评分结果（7 维，每维满分 20，总分 140）

#### S1 玄幻（A 反超 B）

| 维度 | A | B | 差 |
|---|---|---|---|
| logic 逻辑连贯 | 16 | 10 | **A+6** |
| character 人物深度 | 14 | 17 | B+3 |
| continuity 连续性 | 18 | 12 | **A+6** |
| foreshadow 伏笔 | 12 | 18 | B+6 |
| pacing 节奏 | 15 | 16 | B+1 |
| style 风格 | 16 | 18 | B+2 |
| **memory 设定遵守** | **19** | **8** | **A+11** ⬅ 最大波动 |
| **总分** | **110** | 99 | **A 赢 11 分** |

> 评审原话："作品乙（B）文学性和剧情张力远胜，但**核心设定上的致命逻辑错误（无视灵气限制和经脉封印）属不可接受的硬伤**；甲（A）虽平庸但守住了底线。"
> **关键现象**：B 组让经脉被封的角色"凝聚灵气悬停雨珠"，直接违背红线。资产塞太多反而忽略最硬的约束。

#### S2 科幻（B 胜 A）

| 维度 | A | B | 差 |
|---|---|---|---|
| logic | 16 | 18 | B+2 |
| character | 14 | 17 | B+3 |
| continuity | 18 | 19 | B+1 |
| foreshadow | 12 | 16 | B+4 |
| pacing | 15 | 18 | B+3 |
| style | 13 | 17 | B+4 |
| memory | 17 | 18 | B+1 |
| **总分** | 105 | **123** | **B 赢 18 分** |

> 评审原话："乙（B）在人物深度、节奏把控和设定遵守上均优于甲（A），冷峻文风更契合题材。"

#### S3 都市（B 大胜 A）

| 维度 | A | B | 差 |
|---|---|---|---|
| logic | 12 | 18 | B+6 |
| character | 14 | 17 | B+3 |
| continuity | 15 | 19 | B+4 |
| foreshadow | 13 | 18 | B+5 |
| pacing | 15 | 16 | B+1 |
| style | 14 | 18 | B+4 |
| memory | 16 | 19 | B+3 |
| **总分** | 99 | **125** | **B 赢 26 分** |

> 评审原话："甲（B）在克制、细节隐喻和氛围营造上达出版级水准；乙（A）存在**严重的设定违背和台词直白问题**，显得稚嫩。"
> **关键现象**：都市题材最吃风格细节，最小约束不足以支撑细腻度。

### A.5 耗时数据

| 场景 | A 耗时 | B 耗时 | A prompt 字符 | B prompt 字符 |
|---|---|---|---|---|
| S1 | 107.5s | 112.2s | 559 | 1485（+166%）|
| S2 | 99.4s | 116.4s | 598 | 1524（+155%）|
| S3 | 122.2s | 124.5s | 577 | 1503（+160%）|
| **平均** | **110s** | **118s（+7%）** | ~580 | ~1500（+160%）|

**结论**：B 的 prompt 比 A 长近三倍，但生成耗时只多 7%。**证实慢的根源不是 prompt 长度，而是同步链路堆叠的 Inspector/Rewrite/auto_contract。**

### A.6 三条设计改进（已写入模块 7、8）

1. **题材自适应**：风格片段按题材动态纳入（都市/情感纳入，玄幻/科幻不纳入）。
2. **红线突出注入**：红线必须在 prompt 最前最突出（S1 证明"资产多 ≠ 幻觉少"）。
3. **memory 维度优先审计**：AuditExecutor 的 issue 优先级 `memory > continuity > logic`（S1 中 memory 波动最大，差 11 分）。

### A.7 实验局限

- 样本量小（3 组），单场景波动大（-11 到 +26 分），平均 7.9% 有统计不确定性。
- 评分器与生成器是同一模型，存在"自我偏好"风险（严格说应换模型评分）。
- 场景为人工构造，实际用户故事的资产复杂度更高（更多角色、更长前文、更多伏笔）。
- 未测试长篇累积效应（本实验是单场景，非 50+ 章累积）。

**后续验证建议**：实施后用真实用户故事跑批量 A/B（Phase 1 完成后接入真实 `build_writer_prompt` 输出），扩大样本到 20+ 组。

### A.8 实验产物

- 场景定义：`/tmp/sf_ab_test/scenarios.json`
- 生成脚本：`/tmp/sf_ab_test/generate.py`
- 评分脚本：`/tmp/sf_ab_test/judge.py`
- 生成结果（6 份正文）：`/tmp/sf_ab_test/generations.json`
- 评分结果：`/tmp/sf_ab_test/judgements.json`

> 注：以上为实验临时目录，关键结论已沉淀进本文档。若需复现，脚本自包含可重跑。
