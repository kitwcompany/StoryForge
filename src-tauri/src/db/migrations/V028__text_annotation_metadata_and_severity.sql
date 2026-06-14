-- V028: 为 text_annotations 表添加 metadata 和 severity 列
-- 支持分时架构时间线 2 的异步审计 annotation（ai_audit 类型）
-- metadata 存储 Inspector 的维度/评分/建议等结构化数据（JSON）
-- severity 标注问题严重程度（high/medium/low），用于前端颜色区分与债务指示器

ALTER TABLE text_annotations ADD COLUMN metadata TEXT;
ALTER TABLE text_annotations ADD COLUMN severity TEXT NOT NULL DEFAULT 'medium';
