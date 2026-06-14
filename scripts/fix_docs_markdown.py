#!/usr/bin/env python3
"""Normalize Markdown under docs/ for AGENTS.md markdown checks."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
WIDTH = 80

LIST_RE = re.compile(r"^(\s*)([-*+]|\d+\.)\s+")
BLOCKQUOTE_RE = re.compile(r"^(>\s*)")
HEADING_RE = re.compile(r"^(#{1,6}\s+)")
FOOTNOTE_RE = re.compile(r"^(\s*\[\^[^\]]+\]:\s*)")


def detect_fence_lang(lines: list[str], start: int) -> str:
    sample = "\n".join(lines[start + 1 : start + 12]).lower()
    if re.search(r"^\s*(select|create table|insert into)\b", sample, re.M):
        return "sql"
    if re.search(r"^\s*\{", sample, re.M) or '"type"' in sample:
        return "json"
    if re.search(r"^\s*(type:|apiversion:|tool:|---)", sample, re.M):
        return "yaml"
    if re.search(r"^\s*(curl |track |export |# )", sample, re.M):
        return "bash"
    if re.search(r"^\s*(package |interface |world )", sample, re.M):
        return "wit"
    return "text"


def strip_trailing_whitespace(line: str) -> str:
    stripped = line.rstrip(" \t")
    if line.endswith("  ") and not line.endswith("\\"):
        return stripped + "\\"
    return stripped


def wrap_text(text: str, width: int, prefix: str, continuation: str) -> list[str]:
    import textwrap

    available = width - len(prefix)
    if available <= 10:
        return [prefix + text]
    wrapped = textwrap.fill(
        text,
        width=available,
        break_long_words=False,
        break_on_hyphens=True,
    )
    parts = wrapped.split("\n")
    out = [prefix + parts[0]]
    for part in parts[1:]:
        out.append(continuation + part)
    return out


def wrap_line(line: str, width: int = WIDTH) -> list[str]:
    if len(line) <= width:
        return [line]

    m = BLOCKQUOTE_RE.match(line)
    if m:
        prefix = m.group(1)
        return wrap_text(line[len(prefix) :], width, prefix, prefix)

    m = FOOTNOTE_RE.match(line)
    if m:
        prefix = m.group(1)
        return wrap_text(line[len(prefix) :], width, prefix, " " * len(prefix))

    m = LIST_RE.match(line)
    if m:
        indent, marker = m.group(1), m.group(2)
        prefix = f"{indent}{marker} "
        cont = indent + " " * (len(marker) + 2)
        return wrap_text(line[len(prefix) :], width, prefix, cont)

    return wrap_text(line, width, "", "")


def process_content(content: str) -> str:
    lines = content.splitlines()
    out: list[str] = []
    in_fence = False
    i = 0

    while i < len(lines):
        raw = lines[i]
        line = strip_trailing_whitespace(raw)

        if line.strip().startswith("```"):
            if not in_fence and line.strip() == "```":
                lang = detect_fence_lang(lines, i)
                out.append(f"```{lang}")
            else:
                out.append(line)
            in_fence = not in_fence
            i += 1
            continue

        if in_fence:
            out.append(line)
            i += 1
            continue

        if not line.strip():
            out.append("")
            i += 1
            continue

        if line.lstrip().startswith("|") or line.strip() in {"---", "***", "___"}:
            out.append(line)
            i += 1
            continue

        if HEADING_RE.match(line):
            out.append(line)
            i += 1
            continue

        if len(line) <= WIDTH or re.search(r"\]\([^)]*\)", line):
            out.append(line)
            i += 1
            continue

        if i + 1 < len(lines):
            combined = line + lines[i + 1].lstrip()
            if re.search(r"\]\([^)]*\)", combined) and "](" in line:
                out.append(combined)
                i += 2
                continue

        out.extend(wrap_line(line, WIDTH))
        i += 1

    text = "\n".join(out)
    if not text.endswith("\n"):
        text += "\n"
    return text


def fix_emphasis_headings(content: str) -> str:
    replacements = {
        "**Phase 2 — Discover project root** (when required)": (
            "#### Phase 2 — Discover project root (when required)"
        ),
        "**A. UUID / hash exclusively**": "#### A. UUID / hash exclusively",
        "**B. Provisional `{KEY}-{number}` with node disambiguation (e.g. `KITCHEN-42@node-7`)**": (
            "#### B. Provisional `{KEY}-{number}` with node disambiguation "
            "(e.g. `KITCHEN-42@node-7`)"
        ),
        "**C. `{KEY}-{number}` as display-only; UUID canonical internally** *(recommended base)*": (
            "#### C. `{KEY}-{number}` as display-only; UUID canonical internally "
            "*(recommended base)*"
        ),
        "**D. Recommended: C + provisional display (B-lite)**": (
            "#### D. Recommended: C + provisional display (B-lite)"
        ),
    }
    for old, new in replacements.items():
        content = content.replace(old, new)
    return content


def main() -> int:
    paths = sorted(DOCS.rglob("*.md"))
    for path in paths:
        original = path.read_text(encoding="utf-8")
        updated = process_content(original)
        updated = fix_emphasis_headings(updated)
        if updated != original:
            path.write_text(updated, encoding="utf-8")
            print(f"updated {path.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
