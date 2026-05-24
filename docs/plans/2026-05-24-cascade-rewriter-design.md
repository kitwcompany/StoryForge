# Cascade Rewriter 设计文档

**日期**: 2026-05-24
**状态**: 设计阶段
**依赖**: B2 记忆系统（已完成）
**模块定位**: `creative_engine::cascade_rewriter`
**驱动方**: Task System

---

## 1. 目的与范围

实现"幕后调整自动改写正文"的核心原则：当用户在 Backstage 修改角色设定、世界观、故事线时，系统自动识别受影响的场景段落，生成增量改写预览，经用户确认后应用。

### 不在范围内的
- 自动执行无需用户确认的静默改写（所有改写必须经过确认）
- 跨故事级联（仅处理同一故事内的实体变更）
- 非文本类变更（如结构调整、场景重排序）

---

## 2. 核心概念

| 术语 | 定义 |
|------|------|
| **Entity Mention Index** | 场景正文中对知识图谱实体的引用索引。记录 `(scene_id, entity_id, start_pos, end_pos, mention_text)`。 |
| **Change Event** | 实体变更的结构化描述。包含变更前后对比、变更类型（属性修改/关系修改/新增/删除）。 |
| **Impact Set** | 受变更影响的场景集合。通过 Entity Mention Index 反向查询得到。 |
| **Rewrite Segment** | 需要改写的正文片段。以段落为单位，包含原始文本、改写后文本、变更理由。 |
| **Cascade Task** | Task System 中的特殊任务类型，承载级联改写的完整生命周期。 |

---

## 3. 架构

```
Backstage 变更
    │
    ▼
State Sync Service ──emit──► Backstage UI (刷新)
    │                            │
    ▼                            ▼
Change Detector              用户点击"应用变更到正文"
    │                            │
    ▼                            ▼
Impact Analyzer ◄──Entity Mention Index──┘
    │
    ▼
Rewrite Engine ──LLM call──► LLM Service
    │
    ▼
Diff Preview Generator
    │
    ▼
Task System (Cascade Task)
    │
    ▼
Backstage Diff 预览面板 ◄──用户确认/拒绝──► 应用改写
```

---

## 4. 数据模型

### 4.1 Entity Mention Index (SQLite)

```sql
CREATE TABLE entity_mentions (
    id TEXT PRIMARY KEY,
    story_id TEXT NOT NULL,
    scene_id TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,      -- character / world_building / foreshadowing / etc.
    start_pos INTEGER NOT NULL,     -- 在 scene.content 中的字符偏移
    end_pos INTEGER NOT NULL,
    mention_text TEXT NOT NULL,     -- 实际出现的文本（可能与 entity_name 不同）
    confidence REAL NOT NULL DEFAULT 1.0,  -- mention 识别置信度
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES kg_entities(id) ON DELETE CASCADE
);

CREATE INDEX idx_mentions_entity ON entity_mentions(entity_id);
CREATE INDEX idx_mentions_scene ON entity_mentions(scene_id);
CREATE INDEX idx_mentions_story ON entity_mentions(story_id);
```

### 4.2 Change Event (内存结构，序列化为 Task payload)

```rust
pub struct EntityChangeEvent {
    pub story_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub entity_name: String,
    pub change_type: ChangeType,
    pub before_json: String,   // 变更前完整 JSON
    pub after_json: String,    // 变更后完整 JSON
    pub changed_fields: Vec<String>, // 具体变更的字段名
    pub timestamp: String,
}

pub enum ChangeType {
    AttributeModified,  // 属性修改（如角色性格、世界观设定）
    RelationModified,   // 关系修改（如角色间关系变化）
    Created,            // 新增实体
    Deleted,            // 删除实体
}
```

### 4.3 Rewrite Segment (Task result 的一部分)

```rust
pub struct RewriteSegment {
    pub scene_id: String,
    pub paragraph_index: i32,      // 段落索引（基于换行符分割）
    pub original_text: String,
    pub rewritten_text: String,
    pub change_reason: String,     // 为什么需要改写（如"角色性格从'内向'改为'外向'")
    pub user_decision: UserDecision, // pending / accepted / rejected
}

pub enum UserDecision {
    Pending,
    Accepted,
    Rejected,
}
```

---

## 5. 执行流程

### 5.1 触发

当用户在 Backstage 修改实体时，对应的 command handler 在写入数据库后：

1. 发射 `sync-event`（已完成）
2. **新增**: 调用 `ChangeDetector::record_change(entity_id, before, after)` 记录变更
3. **新增**: 如果变更字段属于"正文敏感字段"（如角色性格、世界观规则、故事线目标），则自动创建 Cascade Task

"正文敏感字段"白名单：
- Character: personality, motivation, goal, appearance, relationships
- WorldBuilding: rules, history, geography, magic_system
- Foreshadowing: status, payoff_plan
- StyleDNA: 任何维度变化（但 StyleDNA 变更通过 `styleDnaUpdated` 事件单独处理，不走 Cascade Rewriter）

### 5.2 影响分析 (Impact Analyzer)

```rust
impl ImpactAnalyzer {
    pub fn analyze(&self, change: &EntityChangeEvent) -> Vec<SceneImpact> {
        // 1. 查询所有引用该实体的场景
        let mentions = self.mention_repo.get_by_entity(&change.entity_id);

        // 2. 按场景分组，计算影响分数
        let mut scene_impacts: HashMap<String, SceneImpact> = HashMap::new();
        for mention in mentions {
            let impact = scene_impacts.entry(mention.scene_id.clone())
                .or_insert_with(|| SceneImpact::new(&mention.scene_id));
            impact.mention_count += 1;
            impact.confidence_sum += mention.confidence;
        }

        // 3. 排除最近已改写的场景（避免循环触发）
        // 通过 scene_versions 表检查该场景是否在最近 1 小时内被 Cascade Rewriter 修改过

        // 4. 按影响分数排序
        scene_impacts.into_values()
            .filter(|i| i.mention_count > 0)
            .sorted_by(|a, b| b.score().partial_cmp(&a.score()).unwrap())
            .collect()
    }
}
```

### 5.3 增量改写 (Rewrite Engine)

对每个受影响场景，Rewrite Engine 执行：

1. **段落提取**: 根据 `entity_mentions.start_pos/end_pos` 定位包含 mention 的段落
2. **上下文窗口**: 提取段落前后各 1 段作为上下文（保持连贯性）
3. **Prompt 构建**:
   ```
   你是一位小说编辑。以下段落中引用了角色「{entity_name}」，但该角色的设定已发生变更。

   【变更内容】
   {changed_fields}: {before} → {after}

   【原文段落】
   {paragraph_text}

   【上下文】
   {prev_paragraph}
   {next_paragraph}

   【约束】
   - 仅改写与变更设定直接冲突的句子
   - 保持原文风格、语气、节奏不变
   - 不要增加原文没有的新情节
   - 输出完整的改写后段落
   ```
4. **LLM 调用**: 使用 `LlmService::generate()`，temperature 0.3（低创造性，高忠实度）
5. **结果解析**: 提取改写后段落，与原始段落对比生成 diff

### 5.4 一致性验证

改写后执行轻量验证：

1. **长度检查**: 改写后段落长度不应超过原文的 150% 或低于 50%
2. **实体保留检查**: 改写后的段落仍应包含原实体 mention（避免 LLM 删除实体引用）
3. **风格一致性**: 计算改写段落的 StyleDNA 6 维向量，与故事当前 StyleDNA 的 cosine similarity 应 > 0.85

验证失败时，标记为 `RewriteStatus::NeedsReview`，任务不失败，但增加警告。

### 5.5 用户确认流程

1. Task 完成后，StateSync 发射 `taskCompleted` 事件
2. Backstage 收到事件，自动打开"级联改写预览"面板
3. 面板展示：
   - 变更实体摘要
   - 受影响场景列表（带 mention 数量）
   - 每个场景的 diff 视图（增删改高亮）
   - 批量操作："全部接受" / "全部拒绝" / "逐条确认"
4. 用户确认后，Backstage 调用 `apply_cascade_rewrite(task_id)` command
5. 应用时逐个写入 `scene.content`，生成 `scene_version` 记录，发射 `sceneUpdated` 事件

---

## 6. 与现有系统的集成

### 6.1 Task System (D2 前置)

Cascade Rewriter 作为 Task System 的 executor 注册：

```rust
// task_system/executor.rs
match task.task_type {
    TaskType::BookDeconstruction => { ... }
    TaskType::CascadeRewrite => {
        let payload: CascadeTaskPayload = serde_json::from_str(&task.payload)?;
        cascade_rewriter::execute(payload, &app_handle, progress_tx).await
    }
    _ => { ... }
}
```

### 6.2 State Sync

- Task 创建 → `StateSync::emit_task_created`
- Task 完成 → `StateSync::emit_task_completed`
- Scene 改写应用 → `StateSync::emit_scene_updated`

### 6.3 版本控制

应用改写前自动创建 `scene_version` 快照，用户可随时回滚。

---

## 7. 错误处理

| 错误场景 | 行为 |
|----------|------|
| Entity Mention Index 缺失 | 降级为全文搜索（模糊匹配 entity_name），记录 warn |
| LLM 调用失败 | Task 标记 Failed，保留已生成的 segments，允许重试 |
| 验证失败（风格漂移） | Task 标记 Completed 但结果状态为 `NeedsReview`，用户需手动检查 |
| 用户拒绝所有改写 | Task 标记 Completed，结果状态为 `AllRejected`，不写入任何场景 |

---

## 8. 实现计划

### Phase 1: 基础设施（2-3 天）
- [ ] 创建 `creative_engine::cascade_rewriter` 模块骨架
- [ ] 实现 `entity_mentions` 表 + Repository
- [ ] 实现 `ChangeDetector`（在 story_commands 中 hook 实体更新）
- [ ] 实现 `ImpactAnalyzer`

### Phase 2: 改写引擎（3-4 天）
- [ ] 实现段落提取 + Prompt 构建
- [ ] 集成 `LlmService` 调用
- [ ] 实现一致性验证（长度、实体保留、风格一致性）
- [ ] 实现 Diff 生成

### Phase 3: 用户界面（3-4 天）
- [ ] Backstage "级联改写预览"面板
- [ ] Diff 视图组件
- [ ] 批量确认/拒绝交互
- [ ] 调用 `apply_cascade_rewrite` command

### Phase 4: 集成与打磨（2-3 天）
- [ ] 注册 `TaskType::CascadeRewrite`
- [ ] 端到端测试
- [ ] 性能优化（mention index 批量查询）

---

## 9. 验收标准

- [ ] 修改角色性格设定后，系统能在 5 秒内识别出涉及该角色的 3 个场景
- [ ] 每个受影响场景生成改写预览，diff 视图正确高亮变更
- [ ] 用户接受改写后，场景内容更新，版本记录生成，幕前自动同步
- [ ] LLM 失败时可重试，验证失败时降级为人工审查
- [ ] 新增/删除实体也能触发级联改写（如新增角色需在前文埋伏笔）
