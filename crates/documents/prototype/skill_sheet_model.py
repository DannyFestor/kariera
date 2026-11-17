# PROTOTYPE (throwaway): the Skill Sheet document model, read straight from a vault's Markdown so the layout
# prototypes can be judged on real content before `documents` exists. The real model is built in Rust from the
# domain types (issue #7); this file only shows which fields the layout needs.
#
#   uv run --with pyyaml python -I crates/documents/prototype/skill_sheet_model.py <vault> <build-month> <out.json>

import json
import re
import sys
import unicodedata
from pathlib import Path

import yaml

LANG = "ja"
PHASES = [
    ("requirements", "要件定義"),
    ("basic-design", "基本設計"),
    ("detailed-design", "詳細設計"),
    ("implementation", "製造"),
    ("testing", "テスト"),
    ("operations", "運用・保守"),
]
CATEGORIES = {
    "language": "プログラミング言語",
    "framework": "フレームワーク",
    "library": "ライブラリ",
    "database": "データベース",
    "cloud": "クラウド",
    "platform": "プラットフォーム",
    "tool": "ツール",
    "ai-tool": "AIツール",
    "method": "手法",
    "human-language": "語学",
}
ENGAGEMENT_TYPES = {
    "employment": "直接雇用",
    "dispatch": "派遣",
    "internship": "インターンシップ",
    "freelance": "フリーランス",
    "own-business": "個人事業",
}
CONTRACT_TYPES = {
    "quasi-mandate": "業務委託",
    "employment": "雇用契約",
    "dispatch": "派遣",
    "internship": "インターン",
}


def read_note(path):
    text = path.read_text(encoding="utf-8")
    _, front, body = text.split("---\n", 2)
    return yaml.safe_load(front) or {}, body


def link_target(link):
    return unicodedata.normalize("NFC", re.sub(r"^\[\[|\]\]$", "", link)) if link else None


def sections(body):
    """{(language, heading): text} with language None for the English part."""
    found, language, heading, lines = {}, None, None, []

    def flush():
        if heading:
            found[(language, heading)] = "\n".join(lines).strip()

    for line in body.splitlines():
        if line.startswith("## ") and line[3:] in ("日本語", "Deutsch"):
            flush()
            language, heading, lines = line[3:], None, []
        elif line.startswith("## ") or line.startswith("### "):
            flush()
            heading, lines = line.split(" ", 1)[1], []
        else:
            lines.append(line)
    flush()
    return found


def localized(body_sections, heading):
    return body_sections.get(("日本語", heading)) or body_sections.get((None, heading)) or ""


def bullets(text):
    return [line[2:].strip() for line in text.splitlines() if line.startswith("- ")]


def map3(value):
    if not value:
        return ""
    return value.get(LANG) or value.get("en") or ""


def month_index(text):
    year, month = str(text).split("-")[:2]
    return int(year) * 12 + int(month) - 1


def month_text(index):
    return f"{index // 12}年{index % 12 + 1}月"


def duration_text(months):
    years, rest = divmod(months, 12)
    return f"{years}年{rest}ヶ月" if years else f"{rest}ヶ月"


def load(root, folder):
    return {unicodedata.normalize("NFC", path.stem): read_note(path) for path in sorted((root / folder).rglob("*.md")) if not path.name.startswith(".")}


def skill_rows(skills, projects, build_month):
    months_by_skill, last_by_skill, count_by_skill = {}, {}, {}
    for front, _ in projects.values():
        start = month_index(front["start"])
        end = month_index(front["end"]) if front.get("end") else build_month
        for link in front["skills"]:
            name = link_target(link)
            months_by_skill.setdefault(name, set()).update(range(start, end + 1))
            last_by_skill[name] = max(last_by_skill.get(name, 0), end)
            count_by_skill[name] = count_by_skill.get(name, 0) + 1
    rows = []
    for name, (front, _) in skills.items():
        labels = front.get("labels") or {}
        months = len(months_by_skill.get(name, ()))
        rows.append(
            {
                "name": name,
                "label": labels.get(LANG) or front["name"],
                "category": front["category"],
                "category_label": CATEGORIES[front["category"]],
                "level": map3(front.get("level")),
                "months": months,
                "years": round(months / 12, 1),
                "duration": duration_text(months) if months else "",
                "last_used": month_text(last_by_skill[name]) if name in last_by_skill else "",
                "last_used_index": last_by_skill.get(name),
                "projects": count_by_skill.get(name, 0),
            }
        )
    order = list(CATEGORIES)
    rows.sort(key=lambda row: (order.index(row["category"]), -row["months"], row["label"].lower()))
    return rows


def project_rows(projects, engagements, contracts, organizations, skills, build_month):
    rows = []
    for name, (front, body) in projects.items():
        body_sections = sections(body)
        engagement, _ = engagements[link_target(front["engagement"])]
        client_name = link_target(engagement.get("client") or engagement.get("employer"))
        contract_types = [
            CONTRACT_TYPES[contracts[link_target(link)][0]["contract_type"]]
            for link in engagement.get("contracts") or []
        ]
        start = month_index(front["start"])
        end = month_index(front["end"]) if front.get("end") else None
        confidential = front["confidential"]
        by_category = {}
        for link in front["skills"]:
            skill, _ = skills[link_target(link)]
            label = (skill.get("labels") or {}).get(LANG) or skill["name"]
            by_category.setdefault(skill["category"], []).append(label)
        rows.append(
            {
                "note": name,
                "start": month_text(start),
                "end": month_text(end) if end is not None else "現在",
                "start_index": start,
                "duration": duration_text((end if end is not None else build_month) - start + 1),
                "title": map3(front["public_title"] if confidential else front["title"]),
                "client": organizations[client_name][0]["name"] if client_name else "",
                "end_client": "" if confidential or not front.get("end_client")
                else organizations[link_target(front["end_client"])][0]["name"],
                "confidential": confidential,
                "industry": map3(front["industry"]),
                "role": map3(front["role"]),
                "focus": map3(front.get("focus")),
                "team_size": front.get("team_size"),
                "engagement_type": ENGAGEMENT_TYPES[engagement["engagement_type"]],
                "contract_type": "・".join(dict.fromkeys(contract_types)),
                "phases": [slug in (front.get("phases") or []) for slug, _ in PHASES],
                "skills": by_category,
                "description": localized(body_sections, "Description"),
                "achievements": bullets(localized(body_sections, "Achievements")),
            }
        )
    rows.sort(key=lambda row: -row["start_index"])
    for number, row in enumerate(rows, 1):
        row["number"] = number
    return rows


def profile_block(root, organizations, qualifications):
    front, body = read_note(root / "Profile.md")
    body_sections = sections(body)
    business = link_target(front["business"])
    return {
        "name": front["name"],
        "headline": map3(front["headline"]),
        "business": organizations[business][0]["name"],
        "available_from": str(front.get("available_from") or ""),
        "days_per_week": [front.get("days_per_week_min"), front.get("days_per_week_max")],
        "portfolio_url": front.get("portfolio_url") or "",
        "github_url": front.get("github_url") or "",
        "summary": localized(body_sections, "Summary"),
        "strengths": bullets(localized(body_sections, "Strengths")),
        "self_pr": localized(body_sections, "Self-PR"),
        "qualifications": [
            {
                "label": (q.get("labels") or {}).get(LANG) or q["name"],
                "category": q["category"],
                "date": month_text(month_index(q["date"])),
            }
            for q, _ in sorted(qualifications.values(), key=lambda note: str(note[0]["date"]))
        ],
    }


def main(vault, build_month, out):
    root = Path(vault).expanduser()
    organizations = load(root, "Organizations")
    projects = load(root, "Projects")
    skills = load(root, "Skills")
    month = month_index(build_month)
    model = {
        "language": LANG,
        "build_month": month_text(month),
        "phase_labels": [label for _, label in PHASES],
        "category_labels": CATEGORIES,
        "profile": profile_block(root, organizations, load(root, "Qualifications")),
        "skills": skill_rows(skills, projects, month),
        "projects": project_rows(
            projects, load(root, "Engagements"), load(root, "Contracts"), organizations, skills, month
        ),
    }
    Path(out).write_text(json.dumps(model, ensure_ascii=False, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main(*sys.argv[1:])
