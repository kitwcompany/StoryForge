# StoryForge (草苔) v0.19.0 项目完成状态

> 最后更新: 2026-06-18（v0.19.0 提示词全面可配置化）
> GitHub: https://github.com/91zgaoge/StoryForge

---

## ✅ 已完成功能

### v0.19.0 提示词全面可配置化（2026-06-18）

彻底消灭所有硬编码 LLM 提示词，全部纳入统一注册表，支持前端完整查看、搜索、编辑和重置。

#### 提示词注册表（Prompt Registry）

| 提示词 | 类别 | 用途 | 变量 |
|--------|------|------|------|
| `writer_system` | 写作 | Writer Agent 系统提示词 | genre, style_dna, writing_strategy, methodology, tone, world_rules, foreshadowings, character_states, target_length, scene_contract, anti_ai_guidelines |
| `writer_continue` | 写作 | 续写用户提示词 | genre, style_dna, current_content, scene_structure, instruction, target_length |
| `writer_rewrite` | 写作 | 重写用户提示词 | genre, style_dna, selected_text, context, instruction, target_length |
| `inspector_system` | 审校 | Inspector Agent 系统提示词 | genre, style_dna, audit_dimensions, target_length |
| `style_checker_system` | 审校 | 风格检查系统提示词 | style_dna, sample_text |
| `commentator_system` | 评点 | 古典评点系统提示词 | genre, style_dna, text, paragraph_index |
| `outline_planner` | 规划 | 大纲规划提示词 | genre, style_dna, story_summary, chapter_count, target_length |
| `probe_model` | 探测 | 模型网关探测提示词 | — |
| `intent_detection` | 探测 | 意图识别系统提示词 | — |
| `input_hint` | 探测 | 输入提示系统提示词 | — |
| `memory_compressor` | 记忆 | 记忆压缩提示词 | story_summary, memory_items, max_tokens |
| `knowledge_distiller` | 知识 | 知识蒸馏提示词 | story_summary, entities, max_tokens |
| `narrative_event_extraction` | 叙事 | 叙事事件提取提示词 | text, story_summary |
| `multi_agent_coordinator` | 系统 | 多助手协调提示词 | agents, task_description |
| `multi_agent_writer` | 系统 | 多助手写作提示词 | genre, style_dna, current_content, instruction |
| `multi_agent_inspector` | 系统 | 多助手审校提示词 | genre, style_dna, text, audit_dimensions |
| `multi_agent_commentator` | 系统 | 多助手评点提示词 | genre, style_dna, text, paragraph_index |
| `multi_agent_planner` | 系统 | 多助手规划提示词 | genre, style_dna, story_summary, chapter_count |
| `methodology_snowflake_step1` | 方法论 | 雪花法第1步：一句话摘要 | genre, story_concept |
| `methodology_snowflake_step2` | 方法论 | 雪花法第2步：一段摘要 | genre, one_line_summary |
| ... | ... | ... | ... |
| `methodology_snowflake_step10` | 方法论 | 雪花法第10步：撰写初稿 | genre, story_summary, character_sheets, scene_list |
| `skill_style_enhancer` | 技能 | 风格增强技能提示词 | genre, style_dna, text |
| `skill_plot_twist` | 技能 | 剧情反转技能提示词 | genre, story_summary, current_content |
| `skill_text_formatter` | 技能 | 文本格式化技能提示词 | text, format_type |
| `skill_character_voice` | 技能 | 角色声音技能提示词 | character_name, personality, text |
| `skill_emotion_pacing` | 技能 | 情感节奏技能提示词 | genre, current_content, target_emotion |

#### 架构

| 变更 | 文件 | 说明 |
|------|------|------|
| 注册表扩展 | `src-tauri/src/prompts/registry.rs` | 从 8 个扩展到 35+ 内置 prompt，15 个分类 |
| 技能映射 | `src-tauri/src/skills/executor.rs` | `skill_id_to_prompt_id()` 映射 5 个内置技能 |
| 雪花法注入 | `src-tauri/src/creative_engine/methodology/snowflake.rs` | `prompt_instruction()` 优先查注册表 |
| 记忆系统 | `src-tauri/src/memory/ingest.rs` | `extract_narrative_events` 改用 `resolve_prompt()` |
| 多助手系统 | `src-tauri/src/memory/multi_agent.rs` | 5 个 Agent 提示词改用 `resolve_prompt_default()` |
| 前端面板 | `src-frontend/src/pages/settings/PromptsPanel.tsx` | 15 分类 + 搜索 + 批量重置 + 默认值预览 |
| 设置入口 | `src-frontend/src/pages/settings/GeneralSettings.tsx` | 精简为「提示词注册表」链接卡片 |
| 批量重置 | `src-tauri/src/prompts/commands.rs` | 新增 `reset_all_prompt_overrides` IPC |

#### 前端 UI

- **PromptsPanel**：15 分类折叠面板，支持实时搜索（ID/名称/描述/内容）、分类筛选、批量重置、默认内容预览、模板变量标签高亮
- **GeneralSettings**：移除旧版 2 个 textarea 覆盖，改为「提示词注册表」链接卡片，显示当前覆盖数量

#### 数据流

```
用户编辑 → 前端保存 → IPC save_prompt_override → SQLite prompt_overrides 表
LLM 调用 → resolve_prompt() → 优先查 DB 覆盖 → 无覆盖则回退内置默认
```

---

## 📊 版本历史

| 版本 | 日期 | 核心内容 |
|------|------|----------|
| v0.19.0 | 2026-06-18 | 提示词全面可配置化（35+ prompt，15 分类） |
| v0.18.1 | 2026-06-20 | 设置超时修复（数字输入 + 配置读取路径） |
| v0.18.0 | 2026-06-20 | 后台资产深度审计 × 智能创作流程优化 |
| v0.17.1 | 2026-06-19 | 提示词注册表 + 超时设置修复 + 健康报告刷新 |
| v0.17.0 | 2026-06-19 | 中文叙事增强（桥段卡/剧情引擎/高压关系/读者承诺） |
| v0.16.2 | 2026-06-18 | 修复后台审计 LLM 调用误导前端假超时 |
| v0.16.1 | 2026-06-18 | 修复「距上次响应」计数 Bug |
| v0.16.0 | 2026-06-18 | 智能创作参数全面可配置 |
| v0.15.2 | 2026-06-18 | 修复「已完成」事件在错误检测前发射 |
| v0.15.1 | 2026-06-18 | 生成阶段提示汉字化 |
| v0.15.0 | 2026-06-17 | 模型网关智能调度器 |
| v0.14.4 | 2026-06-18 | 修复应用启动后自动进入生成进程假象 |
| v0.14.3 | 2026-06-17 | 场景智能路由（TimeSliced 默认） |
| v0.14.2 | 2026-06-17 | 多层超时防线 |
| v0.14.1 | 2026-06-17 | 后台设置即时更新重构 |
| v0.13.3 | 2026-06-17 | 诊断卡片安全网 |
| v0.13.2 | 2026-06-17 | 诊断卡片增强 + 前端自救计时器 |
| v0.13.1 | 2026-06-15 | 修复智能创作卡死在「准备上下文」阶段 |
| v0.9.7 | 2026-06-13 | 技能与设置参数对智能创作真正生效 |
| v0.9.4 | 2026-06-12 | 智能创作进度感知与幕前界面精简 |
| v0.9.2 | 2026-06-11 | 自动创作性能优化 |
| v0.9.1 | 2026-06-10 | 架构拆分与全面测试覆盖 |
| v0.9.0 | 2026-06-08 | Brooks-Lint 代码质量重构 |

---

## 🎯 下一步计划

### v0.19.1（计划中）
- [ ] 提示词模板变量自动补全（编辑器内 `{{` 触发下拉）
- [ ] 提示词版本历史（每次保存生成版本快照）
- [ ] 提示词导入/导出（JSON 格式）
- [ ] 提示词 A/B 测试（对比不同提示词效果）

### v0.20.0（规划中）
- [ ] 用户自定义提示词（新增非内置 prompt）
- [ ] 提示词组合模板（多提示词组合成工作流）
- [ ] 提示词性能分析（统计各提示词调用次数/耗时/效果）

---

## 📈 当前统计

- **Rust 代码**: ~45,000 行（src-tauri/src/）
- **前端代码**: ~25,000 行（src-frontend/src/）
- **Rust 测试**: 392 例（全部通过）
- **前端测试**: 126 例（全部通过）
- **数据库迁移**: 94 个版本
- **内置提示词**: 35+ 个
- **技能系统**: 5 个内置技能
- **创作方法论**: 雪花法 10 步
