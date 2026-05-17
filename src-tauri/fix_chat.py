import re

with open('src/chat/mod.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Add AppError import
if 'use crate::error::AppError;' not in content:
    content = content.replace(
        'use crate::db::DbPool;',
        'use crate::db::DbPool;\nuse crate::error::AppError;'
    )

# Replace Result<T, String> -> Result<T, AppError>
content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)

# Replace .map_err(|e| e.to_string())
content = re.sub(r'\.map_err\(\|e\| e\.to_string\(\)\)', '.map_err(AppError::from)', content)

# Replace Err("...".to_string())
content = re.sub(r'Err\("([^"]*)"\.to_string\(\)\)', r'Err(AppError::internal("\1"))', content)

# Replace Err(format!(...))
content = re.sub(r'Err\(format!\(([^)]+)\)\)', r'Err(AppError::internal(format!(\1)))', content)

with open('src/chat/mod.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print('Done')
