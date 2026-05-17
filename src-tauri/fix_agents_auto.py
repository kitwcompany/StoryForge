import re

with open('src/agents/commands.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Change auto_write and auto_revise signatures
content = content.replace(
    'pub async fn auto_write(\n    request: AutoWriteRequest,\n    app_handle: AppHandle,\n) -> Result<AutoWriteResponse, String> {',
    'pub async fn auto_write(\n    request: AutoWriteRequest,\n    app_handle: AppHandle,\n) -> Result<AutoWriteResponse, AppError> {'
)
content = content.replace(
    ') -> Result<AutoReviseResponse, String> {\n    let task_id = Uuid::new_v4().to_string();\n\n    // 预估算文本长度用于配额检查',
    ') -> Result<AutoReviseResponse, AppError> {\n    let task_id = Uuid::new_v4().to_string();\n\n    // 预估算文本长度用于配额检查'
)

# 2. Replace map_err patterns in these functions
content = content.replace(
    '.map_err(|e| crate::error::AppError::from(e).to_string())',
    '.map_err(AppError::from)'
)

with open('src/agents/commands.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print('Done')
