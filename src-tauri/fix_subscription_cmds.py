import re

# Fix subscription/commands.rs
with open('src/subscription/commands.rs', 'r', encoding='utf-8') as f:
    content = f.read()

if 'use crate::error::AppError;' not in content:
    content = content.replace(
        'use crate::db::DbPool;',
        'use crate::db::DbPool;\nuse crate::error::AppError;'
    )

content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)

with open('src/subscription/commands.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# Fix agents/commands.rs: change 4 helper functions
with open('src/agents/commands.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace the 4 helper function signatures
content = content.replace(
    'fn check_auto_write_quota_sync(app_handle: &AppHandle, requested_chars: i32) -> Result<(), String> {',
    'fn check_auto_write_quota_sync(app_handle: &AppHandle, requested_chars: i32) -> Result<(), AppError> {'
)
content = content.replace(
    'fn consume_auto_write_quota_sync(app_handle: &AppHandle, _actual_chars: i32) -> Result<(), String> {',
    'fn consume_auto_write_quota_sync(app_handle: &AppHandle, _actual_chars: i32) -> Result<(), AppError> {'
)
content = content.replace(
    'fn check_auto_revise_quota_sync(app_handle: &AppHandle, requested_chars: i32) -> Result<(), String> {',
    'fn check_auto_revise_quota_sync(app_handle: &AppHandle, requested_chars: i32) -> Result<(), AppError> {'
)
content = content.replace(
    'fn consume_auto_revise_quota_sync(app_handle: &AppHandle, _actual_chars: i32) -> Result<(), String> {',
    'fn consume_auto_revise_quota_sync(app_handle: &AppHandle, _actual_chars: i32) -> Result<(), AppError> {'
)

# Replace Err(...) inside these helpers
content = content.replace(
    'return Err(result.message.unwrap_or_else(|| "今日自动续写次数已用完".to_string()));',
    'return Err(AppError::quota_exceeded("auto_write", result.message.unwrap_or_else(|| "今日自动续写次数已用完".to_string())));'
)
content = content.replace(
    'return Err(result.message.unwrap_or_else(|| "今日自动修改次数已用完".to_string()));',
    'return Err(AppError::quota_exceeded("auto_revise", result.message.unwrap_or_else(|| "今日自动修改次数已用完".to_string())));'
)

with open('src/agents/commands.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# Fix llm/service.rs: check_platform_quota function
with open('src/llm/service.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace(
    'fn check_platform_quota(&self) -> Result<bool, String> {',
    'fn check_platform_quota(&self) -> Result<bool, AppError> {'
)
content = content.replace(
    'let user_id = self.resolve_user_id()\n            .ok_or("无法识别用户身份".to_string())?;',
    'let user_id = self.resolve_user_id()\n            .ok_or_else(|| AppError::internal("无法识别用户身份"))?;'
)
content = content.replace(
    'let pool = self.app_handle.try_state::<crate::db::DbPool>()\n            .ok_or("数据库未初始化")?;',
    'let pool = self.app_handle.try_state::<crate::db::DbPool>()\n            .ok_or_else(|| AppError::internal("数据库未初始化"))?;'
)

with open('src/llm/service.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print('Done')
