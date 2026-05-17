with open('errors.txt', 'r', encoding='utf-8') as f:
    text = f.read()
import re
blocks = re.split(r'error\[E\d+\]:', text)
files = set()
for block in blocks:
    m = re.search(r'--> src-tauri\\\\src\\\\([^:]+)', block)
    if m:
        files.add(m.group(1))
for f in sorted(files):
    print(f)
