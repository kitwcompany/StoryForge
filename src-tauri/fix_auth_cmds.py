import re

with open('src/auth/commands.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Add AppError import
if 'use crate::error::AppError;' not in content:
    content = content.replace(
        'use crate::db::DbPool;',
        'use crate::db::DbPool;\nuse crate::error::AppError;'
    )

# 2. Replace Result<T, String> -> Result<T, AppError>
content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)

# 3. Replace .map_err(|e| e.to_string())
content = re.sub(r'\.map_err\(\|e\| e\.to_string\(\)\)', '.map_err(AppError::from)', content)

with open('src/auth/commands.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print('Done')
