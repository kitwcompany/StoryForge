import re
import sys

files = sys.argv[1:]

for path in files:
    with open(path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Add AppError import if missing
    if 'use crate::error::AppError;' not in content:
        # Try common patterns
        if 'use crate::db::' in content:
            content = content.replace(
                'use crate::db::',
                'use crate::error::AppError;\nuse crate::db::'
            )
        elif 'use super::' in content:
            content = content.replace(
                'use super::',
                'use crate::error::AppError;\nuse super::'
            )
        else:
            # Fallback: add after first use statement
            content = re.sub(r'(use [^;]+;\n)', r'\1use crate::error::AppError;\n', content, count=1)

    # Replace Result<T, String> -> Result<T, AppError>
    content = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', content)

    # Replace .map_err(|e| e.to_string())
    content = re.sub(r'\.map_err\(\|e\| e\.to_string\(\)\)', '.map_err(AppError::from)', content)

    # Replace Err("...".to_string())
    content = re.sub(r'Err\("([^"]*)"\.to_string\(\)\)', r'Err(AppError::internal("\1"))', content)

    # Replace Err(format!(...))
    content = re.sub(r'Err\(format!\(([^)]+)\)\)', r'Err(AppError::internal(format!(\1)))', content)

    with open(path, 'w', encoding='utf-8') as f:
        f.write(content)

    print(f'Done: {path}')
