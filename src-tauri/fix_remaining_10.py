import re

# 1. planner/executor.rs: result.error is Option<String>
with open('src/planner/executor.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = content.replace(
    'return Err(result.error.unwrap_or(AppError::internal("Skill execution failed")));',
    'return Err(AppError::internal(result.error.unwrap_or("Skill execution failed".to_string())));'
)
with open('src/planner/executor.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 2. audit/commands.rs: change signature to AppError
with open('src/audit/commands.rs', 'r', encoding='utf-8') as f:
    content = f.read()
if 'use crate::error::AppError;' not in content:
    content = content.replace('use crate::db::DbPool;', 'use crate::db::DbPool;\nuse crate::error::AppError;')
content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)
with open('src/audit/commands.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 3. lib.rs: fix execute_skill body errors
with open('src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Fix format_text signature
content = content.replace(
    'async fn format_text(content: String, app_handle: AppHandle) -> Result<String, String> {',
    'async fn format_text(content: String, app_handle: AppHandle) -> Result<String, AppError> {'
)
# Fix get_skills and get_skills_by_category signatures
content = content.replace(
    'fn get_skills() -> Result<Vec<SkillInfo>, String> {',
    'fn get_skills() -> Result<Vec<SkillInfo>, AppError> {'
)
content = content.replace(
    'fn get_skills_by_category(category: String) -> Result<Vec<SkillInfo>, String> {',
    'fn get_skills_by_category(category: String) -> Result<Vec<SkillInfo>, AppError> {'
)
# Fix lib.rs execute_skill error handling
content = content.replace(
    'return Err(result.error.unwrap_or("Skill execution failed".to_string()));',
    'return Err(AppError::internal(result.error.unwrap_or("Skill execution failed".to_string())));'
)
content = content.replace(
    'return Err("Skill did not produce a valid prompt".to_string());',
    'return Err(AppError::internal("Skill did not produce a valid prompt"));'
)
with open('src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 4. creative_engine/payoff_ledger.rs: function signature still String
with open('src/creative_engine/payoff_ledger.rs', 'r', encoding='utf-8') as f:
    content = f.read()
# Check if there are still Result<..., String> signatures
content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)
with open('src/creative_engine/payoff_ledger.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 5. canonical_state/manager.rs: function signature still String
with open('src/canonical_state/manager.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)
with open('src/canonical_state/manager.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print('Done')
