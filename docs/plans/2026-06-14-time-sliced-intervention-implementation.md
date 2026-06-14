# 分时介入架构 — 实施计划

> 创建日期: 2026-06-14
> 对应设计: [`2026-06-14-time-sliced-intervention-design.md`](./2026-06-14-time-sliced-intervention-design.md)（Phase 0 已验证 ✅）
> 状态: 待执行
> 总工期预估: 11-14 天（Phase 1~4）

---

## 实施总览

```
Phase 1: 时间线 1 解耦 ──────── 3-4 天  (让生成变快)
Phase 2: 时间线 2 异步审计 ──── 4-5 天  (让资产在后台发力 + annotation 回流)
Phase 3: 时间线 3 深度洞察 ──── 2-3 天  (长篇一致性)
Phase 4: 债务指示器与打磨 ───── 2 天    (体验闭环)
```

每个 Phase 产出**可独立验证**的成果，互不阻塞回滚。Phase 1 完成后用户就能感受到"生成变快"，Phase 2 完成后才有 annotation 回流。

**实施铁律**：每个任务完成后必须跑通 `cargo test --lib` + `npx tsc --noEmit`，不通过不进入下一任务。

---

## Phase 1：时间线 1 解耦（3-4 天）

**目标**：让普通生成走"最小约束 + 跳过 Inspector/Rewrite"的快路径，生成耗时 P95 < 15s。

### 任务 1.1：新增 `GenerationMode::TimeSliced` 枚举值

| 项 | 内容 |
|---|---|
| 文件 | `src-tauri/src/agents/orchestrator.rs:25` |
| 改动 | `enum GenerationMode { Fast, Full }` → 加 `TimeSliced` 变体 |
| 细节 | 加 `name()` 方法的 match 分支：`TimeSliced => "分时"` |
| 验收 | `cargo check` 通过；枚举三值可打印 |
| 依赖 | 无 |

### 任务 1.2：新增 `QuickPreflightChecker`

| 项 | 内容 |
|---|---|
| 文件 | `src-tauri/src/story_system/preflight.rs` |
| 改动 | 新增 `QuickPreflightChecker` 结构体 + `check()` 方法。仅检查角色非空（`CharacterRepository::get_by_story`），DB 查询用 `spawn_blocking` 包裹。失败返回 `PreflightResult::failed(["NoCharacters"])`，**不触发 auto_contract** |
| 参考 | 现有 `PreflightChecker::check_sync`（`:42`）的 spawn_blocking 模式 |
| 验收 | 单元测试：空角色 → failed；有角色 → ok；且不触发任何 LLM 调用 |
| 依赖 | 无 |

### 任务 1.3：新增 `execute_time_sliced` 方法

| 项 | 内容 |
|---|---|
| 文件 | `src-tauri/src/agents/orchestrator.rs` |
| 改动 | ① `generate()` 的 match（`:333-335`）加 `TimeSliced => self.execute_time_sliced(...)` 分支；② 新增 `execute_time_sliced` 方法，流程：`QuickPreflightChecker` → Writer 单轮生成（`candidate_count=1`）→ 轻量 StyleChecker（仅打分不阻塞）→ 保存 → `emit GenerationPhase::Completed` |
| 关键 | **跳过 Inspector、Rewrite、apply_writing_skills、Full Preflight**。这是速度的来源 |
| 暂时 | bundle 暂用现有 `build_writer_prompt` 的 DB 查询（任务 1.5 再替换为 WriteTimeBundle） |
| 验收 | `TimeSliced` 模式生成耗时显著低于 `Full`；E2E 测试：生成成功且无 Inspector 调用 |
| 依赖 | 1.1, 1.2 |

### 任务 1.4：调用点迁移（7 处）

| 项 | 内容 |
|---|---|
| 文件 | `agents/commands.rs:342,699,1109` / `creation_commands.rs:1128` / `planner/executor.rs:760` / `narrative/genesis.rs:582` / `workflow/scheduler.rs:445,543` / `agents/executor.rs:79` |
| 改动 | 按设计文档第 9 节表格迁移：普通生成/auto_write/auto_revise → `TimeSliced`；向导/Genesis/Planner/Workflow → `Full`；默认值 `_ => Full` → `_ => TimeSliced` |
| 验收 | `grep -r "GenerationMode::Full" src-tauri/src/` 只剩向导/Genesis/Planner/Workflow 5 处 + orchestrator 定义处 |
| 依赖 | 1.1, 1.3 |

### 任务 1.5：实现 `WriteTimeBundle`（含红线突出 + 题材自适应）

| 项 | 内容 |
|---|---|
| 文件 | 新增 `src-tauri/src/creative_engine/write_time_bundle.rs` + `creative_engine/mod.rs` 声明模块 |
| 改动 | 定义 `WriteTimeBundle` 结构体（合同红线 + 角色核心 + 场景大纲 + GenreProfile 反模式 + 可选风格片段）。`load(mode, story_id, pool)` 方法，全 `spawn_blocking`。缓存 key=`(story_id, scene_id)` TTL 60s |
| **红线突出** | 按 Phase 0 实证：`to_prompt()` 输出时红线段在最前、加粗强调「绝不可违背」、独立段落 |
| **题材自适应** | 按 `stories.genre` 判断：都市/情感/现实主义 → 纳入轻量风格 DNA（~200 token）；玄幻/科幻 → 不纳入。映射表写入 `GenreProfile` 扩展字段（先硬编码，后续可配置化） |
| 验收 | bundle 序列化 < 3000 token；不同题材输出不同的资产组合；单元测试覆盖题材分支 |
| 依赖 | 无（可与 1.3 并行，1.6 接入） |

### 任务 1.6：`build_writer_prompt` 接受 `WriteTimeBundle`

| 项 | 内容 |
|---|---|
| 文件 | `src-tauri/src/agents/service.rs:1691` |
| 改动 | 签名加 `bundle: &WriteTimeBundle` 参数；函数体内的 DB 查询（角色/世界观/大纲/合同）替换为从 bundle 读取；保留 `tier: SubscriptionTier` 参数不变（门禁用） |
| 兼容 | 保留旧签名一个版本为 `build_writer_prompt_legacy`，供 `Full` 模式过渡使用 |
| 验收 | A/B 对比：用 Phase 0 的 3 组场景，新 prompt 生成的正文质量不低于旧 prompt；`cargo test --lib` 全通过 |
| 依赖 | 1.3, 1.5 |

### Phase 1 出口验证

- [ ] `TimeSliced` 模式生成耗时 P95 < 15s（用真实故事测，非 mock）
- [ ] 普通生成/auto_write 走 `TimeSliced`，向导/Genesis 走 `Full`
- [ ] `cargo test --lib` + `npx tsc --noEmit` + `npx playwright test` 全通过
- [ ] 用户可感知"生成变快"

---

## Phase 2：时间线 2 异步审计 + annotation 回流（4-5 天）

**目标**：正文返回后后台跑 Inspector，问题以 inline annotation 回流，用户可处置。

### 任务 2.1：`TaskType` 新增 `AsyncAudit` + 注册

| 项 | 内容 |
|---|---|
| 文件 | `task_system/models.rs:79`（加 `AsyncAudit` 变体 + Display/FromStr 分支）|
| 改动 | 加枚举值 `"async_audit"` |
| 验收 | `cargo check` 通过；枚举可序列化 |
| 依赖 | 无 |

### 任务 2.2：实现 `AuditExecutor`

| 项 | 内容 |
|---|---|
| 文件 | 新增 `task_system/executors/audit_executor.rs` + `task_system/mod.rs` 声明 |
| 改动 | 实现 `TaskExecutor` trait。`can_handle` 匹配 `AsyncAudit`。`execute` 流程：① 调 `AgentService::execute_task(Inspector)`（复用现有 7 维 Inspector）② 解析返回的 `memory_analysis`/`style_analysis`/issues ③ 每个 issue → `create_text_annotation`（type=`ai_audit`）④ 发 `SyncEvent::AnnotationCreated` |
| **优先级** | 按 Phase 0 实证：issue 排序 `memory > continuity > logic > 其他`，memory 维度的 high-severity 最先回流 |
| **限流** | 复用 `MEMORY_WRITER_SEMAPHORE` 模式，并发上限 2，支持 `CancellationToken` |
| 验收 | 单元测试：mock Inspector 输出 → 产出正确数量的 annotation；取消令牌可中断 |
| 依赖 | 2.1 |

### 任务 2.3：`execute_time_sliced` 触发后台审计

| 项 | 内容 |
|---|---|
| 文件 | `src-tauri/src/agents/orchestrator.rs`（任务 1.3 的方法内）|
| 改动 | 在 `execute_time_sliced` 返回正文**之前**，`tokio::spawn` 一个 `AuditExecutor` 任务（传入 scene_id/content/story_id）。spawn 不阻塞返回 |
| **去重** | 同一 scene 若已有 pending 的 audit task，先取消旧的（LRU 只保留最新）|
| 验收 | 生成返回后，后台日志可见 audit 启动；连续生成两次，第一次的 audit 被取消 |
| 依赖 | 1.3, 2.2 |

### 任务 2.4：`create_text_annotation` 支持 `ai_audit` 类型

| 项 | 内容 |
|---|---|
| 文件 | `scene_commands.rs:608` + `db/repositories.rs`（TextAnnotationRepository）|
| 改动 | 现有签名已有 `from_pos`/`to_pos`（段落级定位可直接用）。需扩展：① 接受 `metadata: Option<serde_json::Value>`（存 dimension/severity/suggestion）② `annotation_type` 支持 `"ai_audit"` 值 ③ DB 表若缺 metadata 列则加 migration（`V028__text_annotation_metadata.sql`）|
| 验收 | 能写入 `ai_audit` 类型 + metadata；查询能读出 metadata |
| 依赖 | 无 |

### 任务 2.5：Inspector prompt 增强（段落定位）

| 项 | 内容 |
|---|---|
| 文件 | `src-tauri/src/prompts/engine.rs:182-235` |
| 改动 | 在 issues 输出格式中加 `paragraph_index`（段落序号，从 0 开始）。**不要求 char_start/char_end**（Phase 0 已定段落级为主方案）。保留现有 `memory_analysis.character_conflicts` 等描述性字段 |
| 验收 | Inspector 返回带 paragraph_index；解析逻辑能提取 |
| 依赖 | 无 |

### 任务 2.6：前端 `TextAnnotationMark` 渲染 `ai_audit`

| 项 | 内容 |
|---|---|
| 文件 | `src-frontend/src/frontstage/extensions/TextAnnotationMark.ts` + `RichTextEditor.tsx` |
| 改动 | ① mark 支持 `ai_audit` 类型，按 severity 渲染颜色（high=红/medium=琥珀/low=蓝）② 悬停展开 metadata（维度/评分/建议）③ 处置按钮：accept/reject/ignore ④ accept 调 `resolve_text_annotation`，reject 调 `delete`，ignore 调 `update`（标记 ignored）|
| 验收 | 后端发 annotation → 前端 3 秒内渲染；三种处置都能调通后端 |
| 依赖 | 2.4 |

### 任务 2.7：`useSyncStore` 处理 `AnnotationCreated` 回调

| 项 | 内容 |
|---|---|
| 文件 | `src-frontend/src/hooks/useSyncStore.ts` |
| 改动 | 现有已监听 `AnnotationCreated` 失效缓存（`useSyncStore.ts` 的 KEYS 工厂已有）。新增：触发 toast 提示「发现 N 个潜在问题」（仅 high-severity 才提示，避免打扰）|
| 验收 | 后台 audit 完成 → 前端 toast + annotation 出现 |
| 依赖 | 2.6 |

### 任务 2.8：端到端集成测试

| 项 | 内容 |
|---|---|
| 改动 | E2E 测试：生成一段正文 → 等待后台 audit → 验证 annotation 出现 → 处置一条 → 验证状态变更 |
| 验收 | Playwright 测试通过；整个链路 < 120s（生成 15s + audit 90s + 渲染）|
| 依赖 | 2.3~2.7 全部完成 |

### Phase 2 出口验证

- [ ] 正文返回后 90s 内，annotation 回流到编辑器
- [ ] annotation 可 accept/reject/ignore
- [ ] 连续生成不堆积（去重生效）
- [ ] `cargo test --lib` + `npx tsc --noEmit` + `npx playwright test` 全通过

---

## Phase 3：时间线 3 深度洞察（2-3 天）

**目标**：低频深度分析（追读力/KG/向量/漂移），防止长篇滚成大灾难。

### 任务 3.1：`TaskType` 新增 `DeepInsight` + `InsightExecutor`

| 项 | 内容 |
|---|---|
| 文件 | `task_system/models.rs`（加 `DeepInsight`）+ 新增 `task_system/executors/insight_executor.rs` |
| 改动 | `InsightExecutor::execute` 调用：① `ReadingPowerEvaluator`（追读力趋势）② `hybrid_search`（KG/向量深度检索）③ Memory Ingest（写入本段实体关系）④ 世界观漂移检测。输出结构性报告（JSON）|
| 限流 | 并发上限 1（冷路径）|
| 验收 | 单元测试：mock 输入 → 产出报告；取消令牌可中断 |
| 依赖 | 无 |

### 任务 3.2：条件触发逻辑

| 项 | 内容 |
|---|---|
| 文件 | `src-tauri/src/agents/orchestrator.rs`（`execute_time_sliced` 或 `AuditExecutor` 完成回调）|
| 改动 | 触发条件（满足任一）：① 距上次 Insight 超过 N 段（默认 5）② audit 发现 high-severity memory issue 超阈值（默认 3）③ 用户主动触发（"叙事分析"页按钮）|
| 验收 | 写 5 段后自动触发 Insight；手动按钮可触发 |
| 依赖 | 3.1, 2.2 |

### 任务 3.3：报告呈现

| 项 | 内容 |
|---|---|
| 文件 | `src-frontend/src/pages/NarrativeAnalysis.tsx` + 新增 API |
| 改动 | Insight 报告写入 DB（新表 `insight_reports` 或复用 `story_summaries`），前端"叙事分析"页展示趋势图 + 漂移警告 |
| 验收 | 报告可见；趋势图正确渲染 |
| 依赖 | 3.1 |

### Phase 3 出口验证

- [ ] 每 5 段自动触发深度分析
- [ ] 报告在"叙事分析"页可见
- [ ] 测试全通过

---

## Phase 4：债务指示器与体验打磨（2 天）

**目标**：让用户直观感知"欠了多少债"，闭环体验。

### 任务 4.1：债务指示器

| 项 | 内容 |
|---|---|
| 文件 | `src-frontend/src/frontstage/components/FrontstageHeader.tsx` + 新增 hook `useDebtIndicator` |
| 改动 | 顶部显示未处理 annotation 数量 + 最严重维度。超阈值（>10 high 或 >30 总计）变红警告。点击跳转到第一个 high-severity annotation |
| 数据源 | `get_text_annotations_by_scene` + 本地过滤 `resolved=false` |
| 验收 | 数量实时更新；超阈值变红；点击跳转正确 |
| 依赖 | Phase 2 完成 |

### 任务 4.2：annotation 视觉规范 + 快捷键

| 项 | 内容 |
|---|---|
| 文件 | `TextAnnotationMark.ts` + 全局快捷键 |
| 改动 | severity 颜色统一；快捷键 Tab=accept / Esc=ignore / Cmd+R=reject（复用现有 AiSuggestionNode 的快捷键模式）|
| 验收 | 快捷键可用；颜色一致 |
| 依赖 | 4.1 |

### 任务 4.3：首次引导 + 文档

| 项 | 内容 |
|---|---|
| 改动 | 首次出现 annotation 时弹引导卡片（"这是 AI 发现的潜在问题，你可以接受/忽略"）；更新 `docs/USER_GUIDE.md` 说明分时审计机制 |
| 验收 | 引导出现一次后不再弹；文档更新 |
| 依赖 | 4.1, 4.2 |

### Phase 4 出口验证

- [ ] 债务指示器准确反映未处理数
- [ ] 快捷键流畅
- [ ] 新用户能理解 annotation 机制

---

## 跨 Phase 的回归保障

每个 Phase 完成后必须全部通过：

```bash
cd src-tauri && cargo test --lib           # Rust 单元测试
cd src-frontend && npx tsc --noEmit        # 前端类型检查
npx playwright test                         # E2E
```

**关键 A/B 回归**：每个 Phase 完成后，用 Phase 0 的 3 组场景重跑生成，确保质量未退化（分数不低于 A 组基线 104.7 的 90%）。

---

## 风险与回滚

| 风险 | 触发条件 | 回滚动作 |
|---|---|---|
| Phase 1 后生成质量明显下降 | A/B 回归分数 < 94（基线 90%）| `build_writer_prompt_legacy` 回退；检查 WriteTimeBundle 注入顺序 |
| Phase 2 后台 audit 堆积 | 队列长度持续 > 5 | 降并发上限到 1；或临时关闭异步审计（`TimeSliced` 退化为纯快速生成）|
| annotation 噪音过多 | 单段产生 > 5 条 annotation | 提高 Inspector severity 阈值（只回流 medium 以上）|
| Inspector 无法给段落定位 | paragraph_index 命中率 < 50% | 全部挂到 scene 末尾（最降级），不阻塞主流程 |

---

## 任务依赖图

```
Phase 1:
  1.1 ─┬─→ 1.3 ──→ 1.4
  1.2 ─┘         ↘
  1.5 ──────────→ 1.6
                  ↓
              [Phase 1 出口]

Phase 2:
  2.1 → 2.2 ──→ 2.3 ──→ 2.8
              ↗
  2.4 (并行)
  2.5 (并行)
  2.6 → 2.7 ──→ 2.8
                  ↓
              [Phase 2 出口]

Phase 3:
  3.1 → 3.2 → 3.3

Phase 4:
  4.1 → 4.2 → 4.3
```

---

## 第一个任务

按依赖图，**任务 1.1（新增 `GenerationMode::TimeSliced` 枚举值）** 是无依赖的起点，且改动最小（一个枚举值 + match 分支），适合作为热身验证工具链。

要我现在开始执行任务 1.1 吗？还是你想先审阅这份实施计划？
