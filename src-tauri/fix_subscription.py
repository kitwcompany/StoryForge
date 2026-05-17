import re

with open('src/subscription/mod.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Add AppError import
if 'use crate::error::AppError;' not in content:
    content = content.replace(
        'use crate::db::DbPool;',
        'use crate::db::DbPool;\nuse crate::error::AppError;'
    )

# 2. Replace function return types: Result<T, String> -> Result<T, AppError>
# But NOT inside FromStr impl where Err = String
lines = content.split('\n')
new_lines = []
in_from_str_impl = False
for i, line in enumerate(lines):
    stripped = line.strip()
    if 'impl std::str::FromStr for SubscriptionTier' in stripped:
        in_from_str_impl = True
    if in_from_str_impl and stripped.startswith('}'):
        in_from_str_impl = False
        new_lines.append(line)
        continue
    if in_from_str_impl:
        new_lines.append(line)
        continue
    # Replace Result<..., String> in function signatures and other places
    line = re.sub(r'Result<([^>]+(?:<[^>]+>)?), String>', r'Result<\1, AppError>', line)
    new_lines.append(line)

content = '\n'.join(new_lines)

# 3. Remove .map_err(|e| e.to_string())
content = re.sub(r'\.map_err\(\|e\| e\.to_string\(\)\)', '', content)

with open('src/subscription/mod.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print('Done')
