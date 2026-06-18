-- v0.17.1 提示词注册表 —— 用户对内置提示词的覆盖
-- 设计：prompt_id 是 PromptRegistry 中的稳定 ID（如 "writer_system"），
-- overridden_content 为用户自定义版本；空则回退到内置默认。

CREATE TABLE IF NOT EXISTS prompt_overrides (
    prompt_id           TEXT PRIMARY KEY,
    overridden_content  TEXT NOT NULL,
    updated_at          INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_prompt_overrides_updated
    ON prompt_overrides(updated_at);
