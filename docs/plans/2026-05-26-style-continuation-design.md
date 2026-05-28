# 续写功能加固设计方案

> 状态：待审批
> 目标：在现有组件基础上增强，实现风格一致续写（古典仿写 7-8 分 / 网文续写 8.5-9 分）

---

## 一、方案总览

### 核心架构

```
参考文本 ──→ StyleMimicAgent（增强）──→ 增强版 StyleDNA + 锚点片段 + N-gram 白名单
                                                          │
    ┌─────────────────────────────────────────────────────┘
    ▼
Writer Agent（prompt 注入风格约束）──→ 生成候选（3 选 1）
    │
    ▼
后处理替换层 ──→ 跨段 StyleChecker ──→ Inspector ──→ Orchestrator 双轨平衡
    │
    ▼
输出（风格一致性评分 + 叙事推进评分）
```

### 双模式切换

| 模式 | 输入 | 流程差异 |
|------|------|---------|
| **经典仿写** | 任意参考文本 | 强风格约束、古风后处理、弱叙事扩展 |
| **小说续写** | 当前故事前文 | 世界观上下文注入、强叙事约束、自适应风格 |

---

## 二、组件级详细设计

### Phase 1：StyleDNA 增强（`creative_engine/style/dna.rs`）

**新增字段**：在现有六维度下增加 `metrics` 数值字段

```rust
/// 新增：词汇量化指标
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct VocabularyMetrics {
    pub four_char_density: f32,           // 四字格密度（%）
    pub function_word_ratio: f32,         // 虚词占比（%）
    pub avg_word_length: f32,             // 平均词长（字）
    pub signature_word_freq: HashMap<String, u32>, // 标志性词汇频率 TOP20
}

/// 新增：句法量化指标
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SyntaxMetrics {
    pub avg_sentence_length: f32,         // 平均句长（字）
    pub sentence_length_std: f32,         // 句长标准差
    pub short_sentence_ratio: f32,        // 短句占比（<10字）
    pub long_sentence_ratio: f32,         // 长句占比（>30字）
    pub comma_density: f32,               // 逗号密度（每百字）
}

/// 新增：短语特征
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PhraseProfile {
    pub bigram_whitelist: Vec<String>,    // 高频双字搭配白名单 TOP30
    pub four_char_whitelist: Vec<String>, // 高频四字词 TOP20
    pub transition_patterns: Vec<String>, // 衔接词模式 TOP15
}
```

**工作量**：~60 行

---

### Phase 2：StyleMimicAgent 改造（`agents/style_mimic.rs`）

**现状问题**：输出独立的 `StyleAnalysis`（定性 JSON），与 `StyleDNA` 不互通

**改造目标**：让 StyleMimic 直接输出**增强版 StyleDNA**（定性 + 定量），彻底取代 `StyleAnalysis`

**新增方法**：

```rust
impl StyleMimicAgent {
    /// 分析参考文本，输出增强版 StyleDNA
    pub async fn analyze_to_dna(&self, reference_text: &str) -> Result<StyleDNA, Error> {
        // 1. 复用 Anti-AI Review 文本统计计算 metrics
        let metrics = Self::compute_metrics(reference_text);
        // 2. 复用现有 LLM 分析生成定性描述
        let qualitative = self.analyze_qualitative(reference_text).await?;
        // 3. 提取锚点片段
        let anchors = Self::sample_style_anchors(reference_text, 5);
        // 4. 提取 N-gram 白名单
        let phrases = Self::extract_phrase_whitelist(reference_text);
        // 合并为完整 StyleDNA
        Ok(StyleDNA { meta, vocabulary, syntax, rhetoric, perspective, emotion, dialogue, metrics, anchors, phrases })
    }

    /// 从文本中采样风格代表性片段（用于少样本注入）
    fn sample_style_anchors(text: &str, count: usize) -> Vec<String> {
        // 按风格强度排序（四字格密度 × 修辞复杂度），取最典型片段
        // 每段 50-100 字，不截断句子
    }

    /// 提取高频短语白名单
    fn extract_phrase_whitelist(text: &str) -> PhraseProfile {
        // 统计双字/四字搭配频率，取 TOP N
    }
}
```

**工作量**：~80 行

---

### Phase 3：Writer Agent Prompt 增强（`prompts/` 新增）

**新增续写专用 prompt 模板**：

```
【角色】你是一位精通中国古典文学风格的写作大师。

【任务】基于以下参考文本的风格，续写一段内容。

【参考文本锚点片段】（少样本示例）
{anchors}

【风格约束 - 硬性指标】
- 平均句长: {avg_sentence_length}±3 字
- 四字格密度: ≥{four_char_density}%
- 短句占比: {short_sentence_ratio}%
- 虚词偏好: 多用 {preferred_function_words}
- 对话标签: {dialogue_tag_pattern}

【短语偏好 - 优先使用】
{phrase_whitelist}

【叙事约束】
- 续写长度: {target_length} 字
- 世界观设定: {world_setting}（不得改变已有设定）
- 需引入: {narrative_requirements}

【禁止】
{avoided_patterns}

【输出要求】
直接输出续写正文，不要解释，不要标注。
```

**工作量**：~100 行（prompt 模板 + 参数注入逻辑）

---

### Phase 4：后处理替换层（`utils/` 新增）

**轻量后处理模块**：不改变句法，只做词汇对齐

```rust
/// 根据 StyleDNA 对生成文本做古风化微调
pub fn align_style(text: &str, dna: &StyleDNA) -> String {
    let mut result = text.to_string();
    
    // 1. 通用虚词替换（从原文统计出的高频映射）
    for (modern, archaic) in &dna.vocabulary.avoided_patterns {
        result = result.replace(modern, archaic);
    }
    
    // 2. 对话标签对齐
    result = align_dialogue_tags(&result, &dna.dialogue);
    
    // 3. 四字格密度补偿（密度不足时，用同义四字词替换）
    if compute_four_char_density(&result) < dna.metrics.four_char_density * 0.8 {
        result = inject_four_char_phrases(&result, &dna.phrase_profile.four_char_whitelist);
    }
    
    result
}
```

**关键原则**：只替换虚词/衔接词，不替换名词动词（避免改变语义）。这是"润色"不是"重写"。

**工作量**：~50 行

---

### Phase 5：StyleChecker 数值偏离（`creative_engine/style/drift_checker.rs`）

**增强现有 StyleChecker**，增加数值偏离计算：

```rust
pub struct NumericDrift {
    pub dimension: String,
    pub original_value: f32,
    pub generated_value: f32,
    pub deviation_percent: f32,
    pub severity: DriftSeverity,
}

impl StyleChecker {
    /// 对比原文和生成文本的数值风格指标
    pub fn check_numeric_drift(
        &self,
        original_dna: &StyleDNA,
        generated_text: &str,
    ) -> Vec<NumericDrift> {
        let generated_dna = StyleMimicAgent::compute_metrics(generated_text);
        vec![
            NumericDrift {
                dimension: "平均句长".to_string(),
                original_value: original_dna.metrics.syntax.avg_sentence_length,
                generated_value: generated_dna.syntax.avg_sentence_length,
                deviation_percent: calculate_deviation(...),
                severity: classify_severity(...),
            },
            // ... 其他维度
        ]
    }
}
```

**工作量**：~40 行

---

### Phase 6：生成-评分-选优（`agents/orchestrator.rs`）

**在现有 Orchestrator 中新增候选模式**：

```rust
/// 生成多个候选，选风格分最高的一版
async fn generate_candidates(
    &self,
    context: &WriterContext,
    style_dna: &StyleDNA,
    count: usize,
) -> Result<String, Error> {
    let candidates = futures::future::join_all(
        (0..count).map(|i| self.writer.generate(context, style_dna, i as u64))
    ).await;
    
    let best = candidates.into_iter()
        .map(|text| {
            let style_score = self.style_checker.score(style_dna, &text);
            (text, style_score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(text, _)| text)
        .ok_or_else(|| Error::NoValidCandidate)?;
    
    Ok(best)
}
```

**短续写**：3 候选并行生成，选最优
**长续写**：每段 2 候选，降低 LLM 调用成本

**工作量**：~30 行

---

### Phase 7：Orchestrator 双轨平衡（`agents/orchestrator.rs`）

**在现有 WorkflowResult 中增加双轨评分**：

```rust
pub struct WorkflowResult {
    // 现有字段...
    pub style_score: f32,              // 新增：风格一致性 (0-1)
    pub narrative_score: f32,          // 新增：叙事推进 (0-1)
    pub drift_details: Vec<NumericDrift>, // 新增：具体偏离项
}

/// 双轨平衡逻辑
fn balance_style_narrative(
    style_score: f32,
    narrative_score: f32,
    config: &mut WorkflowConfig,
) {
    if style_score < 0.7 && narrative_score > 0.85 {
        // 风格漂移严重，叙事优秀 → 降低叙事复杂度，优先保风格
        config.narrative_complexity *= 0.7;
        config.style_weight = 0.7;
        config.narrative_weight = 0.3;
    } else if narrative_score < 0.6 {
        // 叙事薄弱 → 允许适度风格漂移
        config.style_strictness *= 0.8;
        config.style_weight = 0.4;
        config.narrative_weight = 0.6;
    }
}
```

**用户可调节**：`GeneralSettings.tsx` 中新增「风格-叙事平衡滑块」（0=纯风格优先 → 100=纯叙事优先）

**工作量**：~40 行

---

### Phase 8：前端交互（`frontstage/`）

**新增续写模式入口**：

1. **幕前 `/` 命令菜单**：新增「续写」命令
2. **续写设置弹窗**：
   - 输入参考文本（或自动取前文）
   - 选择模式：「经典仿写」/「小说续写」
   - 目标长度：短（500字）/ 中（2000字）/ 长（5000字）
   - 风格-叙事平衡滑块
3. **生成预览面板**：
   - 显示风格一致性评分
   - 显示具体偏离项（如"句长偏长 +23%"）
   - 「重新生成」按钮（换候选）
   - 「接受」/「放弃」按钮

**工作量**：~120 行

---

## 三、实施计划

### 优先级排序

| 阶段 | 内容 | 预估工时 | 依赖 |
|------|------|---------|------|
| **P0** | Phase 1 StyleDNA metrics 增强 | 2h | 无 |
| **P0** | Phase 2 StyleMimicAgent 改造 | 3h | Phase 1 |
| **P0** | Phase 3 续写 Prompt 模板 | 2h | Phase 2 |
| **P1** | Phase 5 StyleChecker 数值偏离 | 2h | Phase 1 |
| **P1** | Phase 6 生成-选优 | 2h | Phase 3 |
| **P1** | Phase 7 Orchestrator 双轨平衡 | 2h | Phase 5,6 |
| **P2** | Phase 4 后处理替换层 | 2h | Phase 1 |
| **P2** | Phase 8 前端交互 | 3h | Phase 7 |

**总计**：~18 小时，7 个文件，0 个新增 module

### 验收标准

| 测试项 | 通过标准 |
|--------|---------|
| 红楼梦 500 字仿写 | 3 位非专业读者中 ≥2 人认为"风格很像" |
| 网文 2000 字续写 | StyleChecker 风格分 ≥ 0.8 |
| 长续写跨段一致性 | 第 1/2/3 段风格分差异 < 0.15 |
| 双轨平衡 | 滑块调至"风格优先"时，叙事分下降但风格分 ≥ 0.85 |

---

## 四、风险与备选

| 风险 | 概率 | 应对 |
|------|------|------|
| LLM 对量化约束响应不稳定 | 中 | 增加 prompt 中少样本锚点的权重，降低纯数字约束权重 |
| 长续写 3 候选成本过高 | 低 | 长续写降为 2 候选，或只在首段用 3 候选 |
| 后处理替换改变语义 | 低 | 白名单严格限定虚词/衔接词，名词动词不替换 |
| 与现有 Pipeline 冲突 | 低 | 新增续写模式为独立入口，不改动现有 auto_write/smart_execute 路径 |

---

**等待审批。批准后开始 Phase 0 实施。**
