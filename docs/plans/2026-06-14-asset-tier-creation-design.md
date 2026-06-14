# 文思资产分级与三模式创作系统设计文档（已废弃）

> 创建日期: 2026-06-14
> 状态: **❌ 已废弃，被 [`2026-06-14-time-sliced-intervention-design.md`](./2026-06-14-time-sliced-intervention-design.md) 取代**
> 废弃原因: 本设计假设"慢的根源是资产过多导致 prompt 太长"，但 Phase 0 实验数据（prompt 长 160% 仅耗时多 7%）否证了这一假设。正确解法是**分时介入**——改变资产介入时机而非减少资产深度。

---

## ⚠️ 替代文档

**请使用 [`2026-06-14-time-sliced-intervention-design.md`](./2026-06-14-time-sliced-intervention-design.md)（分时介入架构）作为本项目的实施设计。**

---

## 被取代的设计决策（保留为历史参考）

以下决策在数据层面上**仍然部分正确**，但已被 time-sliced 架构以更好的方式实现：

### 保留部分（作为数据字典）

| 决策 | 原方案 | time-sliced 中的对应 |
|------|--------|---------------------|
| P0 防错级内容 | 世界观+角色+大纲+故事线+GenreProfile | WriteTimeBundle 的资产来源 |
| Preflight 拆分 | QuickPreflight / FullPreflight | QuickPreflightChecker（time-sliced 时间线 1）|
| 资产分类 | P0/P1/P2/P3 四级 | 转为资产归属时间线的数据字典 |

### 被取代部分（不应再使用）

| 原决策 | 被取代原因 |
|--------|-----------|
| AssetLoader 按 tier 注入 | 被 WriteTimeBundle 取代（不是"按需加载"而是"改变时机"） |
| GenerationTier Standard/Pro | 被 GenerationMode TimeSliced/Full 取代 |
| LightInspector / DeepInspector 分层 | 被单一 Inspector + 异步执行取代 |
| 三级模式静态行为 | 被三条时间线的动态行为取代 |
| "/" 是 Pro 专属命令 | 被 TimeSliced 默认 + Full 可选取代 |
| 标准模式跳过 Inspector | 被 Inspector 走时间线 2 后台异步取代 |

---

## 原设计核心错误

**错误假设**：资产越多 → prompt 越长 → 越慢，所以减少资产 = 加速。

**Phase 0 实验数据**：
- B 组（全量资产）prompt 比 A 组（最小约束）长 160%，但生成耗时只多 7%
- **结论**：慢的根源不是 prompt 长度（资产量），而是同步链路堆叠的 Inspector/Rewrite/auto_contract

**正确解法**：不是削弱资产，而是改变资产介入时机——让每个资产在它该发力的那一刻独立介入。详见 time-sliced 设计文档。
