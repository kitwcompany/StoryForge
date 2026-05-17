import re

with open('errors.txt', 'r', encoding='utf-8') as f:
    text = f.read()

files = re.findall(r'--> src-tauri\\\\src\\\\([^:]+)', text)
from collections import Counter
c = Counter(files)
for f, n in c.most_common(20):
    print(f'{n:3d} {f}')
