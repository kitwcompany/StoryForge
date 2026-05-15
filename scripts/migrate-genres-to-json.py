#!/usr/bin/env python3
"""
一次性迁移脚本：将 templates/genres/*.md 转换为 templates/genres.json

解析 Markdown 结构：
  # 中文名 (EnglishName)
  ## 核心基调
  ## 节奏策略
  ## 反模式 (Anti-Patterns)
  ## 参考数据表
  ## 典型结构
"""

import json
import os
import re
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent.resolve()
REPO_ROOT = SCRIPT_DIR.parent
GENRES_DIR = REPO_ROOT / "templates" / "genres"
OUTPUT_PATH = REPO_ROOT / "templates" / "genres.json"


def parse_markdown(path: Path) -> dict:
    content = path.read_text(encoding="utf-8")
    lines = content.splitlines()

    # 解析标题: # 修仙 (Cultivation)
    title_match = re.match(r"^#\s+(.+?)\s+\((.+?)\)", lines[0].strip())
    if not title_match:
        title_match = re.match(r"^#\s+(.+)", lines[0].strip())
        chinese_name = title_match.group(1).strip() if title_match else path.stem
        canonical_name = path.stem
    else:
        chinese_name = title_match.group(1).strip()
        canonical_name = title_match.group(2).strip()

    # 按二级标题分段
    sections: dict[str, list[str]] = {}
    current_heading = None
    for line in lines[1:]:
        line = line.rstrip()
        if line.startswith("## "):
            current_heading = line[3:].strip()
            sections[current_heading] = []
        elif current_heading is not None:
            sections[current_heading].append(line)

    def extract_text(keywords: list[str]) -> str:
        for kw in keywords:
            if kw in sections:
                return "\n".join(sections[kw]).strip()
        return ""

    core_tone = extract_text(["核心基调"])
    pacing_strategy = extract_text(["节奏策略"])
    anti_patterns_raw = extract_text(["反模式 (Anti-Patterns)", "反模式"])
    reference_tables_raw = extract_text(["参考数据表"])
    typical_structure = extract_text(["典型结构"])

    # 解析反模式为列表
    anti_patterns = []
    for line in anti_patterns_raw.splitlines():
        m = re.match(r"-\s+\[\s*\]\s+(.+)", line.strip())
        if m:
            anti_patterns.append(m.group(1).strip())

    # 解析参考数据表为 Markdown 表格（保留原始文本）
    reference_tables = reference_tables_raw

    # 解析典型结构为列表
    typical_structure_list = []
    for line in typical_structure.splitlines():
        m = re.match(r"\d+\.\s+\*\*(.+?)\*\*\s*\((.+?)\)", line.strip())
        if m:
            typical_structure_list.append({
                "title": m.group(1).strip(),
                "description": m.group(2).strip(),
            })
        elif line.strip().startswith("-"):
            typical_structure_list.append({"title": line.strip().lstrip("- ").strip(), "description": ""})

    return {
        "id": path.stem,
        "genre_name": chinese_name,
        "canonical_name": canonical_name,
        "aliases": [canonical_name.lower(), path.stem.replace("-", " ")],
        "core_tone": core_tone,
        "pacing_strategy": pacing_strategy,
        "anti_patterns": anti_patterns,
        "reference_tables": reference_tables,
        "typical_structure": typical_structure_list,
        "is_builtin": True,
    }


def main():
    if not GENRES_DIR.exists():
        print(f"ERROR: {GENRES_DIR} does not exist")
        return 1

    profiles = []
    for md_path in sorted(GENRES_DIR.glob("*.md")):
        try:
            profile = parse_markdown(md_path)
            profiles.append(profile)
            print(f"  OK: {md_path.name} -> {profile['genre_name']} ({profile['canonical_name']})")
        except Exception as e:
            print(f"  FAIL: {md_path.name} -> {e}")
            return 1

    output = {
        "version": "1.0.0",
        "count": len(profiles),
        "profiles": profiles,
    }

    OUTPUT_PATH.write_text(json.dumps(output, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"\nWrote {len(profiles)} genre profiles to {OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    exit(main())
