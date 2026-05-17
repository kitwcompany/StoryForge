import re

# 1. skills/executor.rs: error: Some(e) where e is AppError but field expects String
with open('src/skills/executor.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = content.replace('error: Some(e),', 'error: Some(e.to_string()),')
with open('src/skills/executor.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 2. mcp/server.rs: McpError::RpcError(e) expects String
with open('src/mcp/server.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = content.replace('Ok(Err(e)) => Err(McpError::RpcError(e)),', 'Ok(Err(e)) => Err(McpError::RpcError(e.to_string())),')
with open('src/mcp/server.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 3. task_system/commands.rs: ok_or_else with String
with open('src/task_system/commands.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = content.replace(
    '.and_then(|opt| opt.ok_or_else(|| "Task not found".to_string()))',
    '.and_then(|opt| opt.ok_or_else(|| AppError::not_found("Task", &id)))'
)
with open('src/task_system/commands.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 4. planner/executor.rs: Err(result.error.unwrap_or_else(...))
with open('src/planner/executor.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = content.replace(
    'return Err(result.error.unwrap_or_else(|| "Skill execution failed".to_string()));',
    'return Err(result.error.unwrap_or(AppError::internal("Skill execution failed")));'
)
# Also fix line 374: builder.build returns String error but function expects AppError
content = content.replace(
    'builder.build(story_id, scene_number, current_content, selected_text)',
    'Ok(builder.build(story_id, scene_number, current_content, selected_text)?)'
)
with open('src/planner/executor.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 5. skills/mod.rs: enable/disable return AppError but registry returns String
with open('src/skills/mod.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = content.replace(
    'self.registry.lock().unwrap().enable(skill_id)',
    'Ok(self.registry.lock().unwrap().enable(skill_id)?)'
)
content = content.replace(
    'self.registry.lock().unwrap().disable(skill_id)',
    'Ok(self.registry.lock().unwrap().disable(skill_id)?)'
)
with open('src/skills/mod.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 6. creative_engine/payoff_ledger.rs: function returns String but constructs AppError
with open('src/creative_engine/payoff_ledger.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)
if 'use crate::error::AppError;' not in content:
    content = content.replace('use super::', 'use crate::error::AppError;\nuse super::')
with open('src/creative_engine/payoff_ledger.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 7. creative_engine/adaptive/mod.rs: functions return String but internal calls return AppError
with open('src/creative_engine/adaptive/mod.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)
if 'use crate::error::AppError;' not in content:
    content = content.replace('use super::', 'use crate::error::AppError;\nuse super::')
with open('src/creative_engine/adaptive/mod.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 8. canonical_state/manager.rs: function returns String but ledger returns AppError
with open('src/canonical_state/manager.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)
if 'use crate::error::AppError;' not in content:
    content = content.replace('use crate::db::DbPool;', 'use crate::db::DbPool;\nuse crate::error::AppError;')
with open('src/canonical_state/manager.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 9. audit/mod.rs: function returns String but ledger returns AppError
with open('src/audit/mod.rs', 'r', encoding='utf-8') as f:
    content = f.read()
content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)
if 'use crate::error::AppError;' not in content:
    content = content.replace('use crate::db::DbPool;', 'use crate::db::DbPool;\nuse crate::error::AppError;')
with open('src/audit/mod.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# 10. lib.rs skill functions: change signatures to AppError
with open('src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Change skill command signatures
for old, new in [
    ('fn import_skill(path: String) -> Result<SkillInfo, String> {', 'fn import_skill(path: String) -> Result<SkillInfo, AppError> {'),
    ('fn enable_skill(skill_id: String) -> Result<(), String> {', 'fn enable_skill(skill_id: String) -> Result<(), AppError> {'),
    ('fn disable_skill(skill_id: String) -> Result<(), String> {', 'fn disable_skill(skill_id: String) -> Result<(), AppError> {'),
    ('fn uninstall_skill(skill_id: String) -> Result<(), String> {', 'fn uninstall_skill(skill_id: String) -> Result<(), AppError> {'),
    ('fn get_skill(skill_id: String) -> Result<SkillInfo, String> {', 'fn get_skill(skill_id: String) -> Result<SkillInfo, AppError> {'),
    ('fn update_skill(skill_id: String, manifest: skills::SkillManifest) -> Result<(), String> {', 'fn update_skill(skill_id: String, manifest: skills::SkillManifest) -> Result<(), AppError> {'),
    ('async fn execute_skill(\n    skill_id: String,\n    params: HashMap<String, serde_json::Value>,\n    app_handle: AppHandle,\n) -> Result<serde_json::Value, String> {', 'async fn execute_skill(\n    skill_id: String,\n    params: HashMap<String, serde_json::Value>,\n    app_handle: AppHandle,\n) -> Result<serde_json::Value, AppError> {'),
    ('async fn get_canonical_state(story_id: String) -> Result<canonical_state::CanonicalStateSnapshot, String> {', 'async fn get_canonical_state(story_id: String) -> Result<canonical_state::CanonicalStateSnapshot, AppError> {'),
]:
    content = content.replace(old, new)

# Fix ok_or and ok_or_else in these functions
content = content.replace('SKILL_MANAGER.get().ok_or("Skills not initialized")?', 'SKILL_MANAGER.get().ok_or(AppError::internal("Skills not initialized"))?')
content = content.replace('.ok_or_else(|| "Skill not found".to_string())', '.ok_or_else(|| AppError::not_found("Skill", &skill_id))')

# Fix record_feedback
content = content.replace(
    'async fn record_feedback(request: RecordFeedbackRequest) -> Result<Vec<LearningPoint>, String> {',
    'async fn record_feedback(request: RecordFeedbackRequest) -> Result<Vec<LearningPoint>, AppError> {'
)
content = content.replace(
    '_ => Err("Unknown feedback type".to_string()),',
    '_ => Err(AppError::validation_failed("Unknown feedback type", None::<String>)),'
)

with open('src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print('Done')
