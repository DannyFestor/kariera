# PROTOTYPE (throwaway): the Skill Sheet layout variants as XLSX. Written with Python's XlsxWriter because its
# layout API is the one rust_xlsxwriter ports (same author, same method names): only features rust_xlsxwriter has
# are used here (formats, merge_range, row heights, stacked text, A4 print setup with fit-to-width, repeat rows,
# horizontal page breaks, freeze panes, autofilter, data bars).
#
#   uv run --with xlsxwriter python -I crates/render-xlsx/prototype/skill_sheet_xlsx.py <model.json> <a|b|c> <out.xlsx>

import json
import math
import sys
import unicodedata

import xlsxwriter

A4 = 9
# Excel cannot fit row heights to wrapped text on its own (it does not recalculate them on open), and a row is at
# most 409 pt high, so every height is estimated here from the text and the column width.
MAX_ROW_HEIGHT = 409
PRINTABLE_PORTRAIT = (720, 770)
PRINTABLE_LANDSCAPE = (1060, 520)


def text_width_px(text, size):
    em = size * 96 / 72
    return sum(em if unicodedata.east_asian_width(c) in "WF" else em * 0.55 for c in text)


def column_px(units):
    return units * 7 + 5


def lines_needed(text, width_units, size):
    usable = max(column_px(width_units) - 8, 10)
    return sum(max(1, math.ceil(text_width_px(line, size) / usable)) for line in str(text).split("\n"))


def height_for(text, width_units, size):
    return min(MAX_ROW_HEIGHT, lines_needed(text, width_units, size) * size * 1.45 + 5)


class Sheet:
    """A worksheet on a grid of equal columns, so every block is a merge over a span of columns."""

    def __init__(self, workbook, name, columns, unit, font, landscape=False):
        self.book, self.font, self.unit = workbook, font, unit
        self.ws = workbook.add_worksheet(name)
        self.ws.set_column(0, columns - 1, unit)
        self.ws.set_paper(A4)
        self.ws.fit_to_pages(1, 0)
        self.ws.set_margins(left=0.4, right=0.4, top=0.5, bottom=0.6)
        self.ws.center_horizontally()
        self.ws.set_footer("&C&8&P / &N")
        self.ws.hide_gridlines(0)
        if landscape:
            self.ws.set_landscape()
        self.formats = {}
        self.row = 0
        self.heights = {}
        self.page_width, self.page_height = PRINTABLE_LANDSCAPE if landscape else PRINTABLE_PORTRAIT
        self.scale = min(1.0, self.page_width / (columns * column_px(unit)))

    def fmt(self, **props):
        key = tuple(sorted(props.items()))
        if key not in self.formats:
            base = {"font_name": self.font, "font_size": 9, "valign": "top", "text_wrap": True}
            self.formats[key] = self.book.add_format({**base, **props})
        return self.formats[key]

    def put(self, row, first, last, value, last_row=None, **props):
        last_row = row if last_row is None else last_row
        if (first, row) == (last, last_row):
            self.ws.write(row, first, value, self.fmt(**props))
        else:
            self.ws.merge_range(row, first, last_row, last, value, self.fmt(**props))

    def fit(self, row, text, span, size=9):
        height = height_for(text, span * self.unit, size)
        self.heights[row] = max(self.heights.get(row, 15), height)

    def grow_to_fit(self, first_row, last_row, text, span, size=9):
        """Taller last row when a cell merged over several rows needs more lines than the rows give."""
        needed = height_for(text, span * self.unit, size)
        have = sum(self.heights.get(r, 15) for r in range(first_row, last_row + 1))
        if needed > have:
            self.heights[last_row] = min(MAX_ROW_HEIGHT, self.heights.get(last_row, 15) + needed - have)

    def finish(self, block_starts=()):
        for row, height in self.heights.items():
            self.ws.set_row(row, height)
        self.ws.set_h_pagebreaks(self.page_breaks(block_starts))

    def page_breaks(self, block_starts):
        """A break before each block that would not fit on the rest of the page, so no block is split."""
        capacity = self.page_height / self.scale
        starts = sorted(block_starts) + [self.row]
        breaks, used = [], sum(self.heights.get(r, 15) for r in range(0, starts[0]))
        for start, end in zip(starts, starts[1:]):
            block = sum(self.heights.get(r, 15) for r in range(start, end))
            if used + block > capacity and used > 0:
                breaks.append(start)
                used = 0
            used += block
        return breaks


def availability(profile):
    low, high = profile["days_per_week"]
    days = "" if low is None else f"週{low}日" if low == high else f"週{low}〜{high}日"
    return " 〜 ".join(part for part in (profile["available_from"], days) if part)


def qualifications(profile, *categories):
    return "\n".join(f"{q['date']} {q['label']}" for q in profile["qualifications"] if q["category"] in categories)


def used_skills(model, *categories):
    return [s for s in model["skills"] if s["months"] and s["category"] in categories]


def skills_of(project, *categories):
    return [label for c in categories for label in project["skills"].get(c, [])]


def variant_a(book, model):
    """The agency format: a grid of 38 narrow columns (Excel 方眼紙), merges for every block, one Project per group."""
    ink, pale = "#1F3A5F", "#E8EEF5"
    sheet = Sheet(book, "スキルシート", 38, 3.2, "BIZ UDPゴシック")
    ws, profile = sheet.ws, model["profile"]
    border = {"border": 1, "border_color": ink}
    label = {**border, "bold": True, "bg_color": pale, "align": "center", "valign": "vcenter"}
    head = {**border, "bold": True, "bg_color": ink, "font_color": "white", "align": "center", "valign": "vcenter"}
    cell = {**border}

    sheet.put(0, 0, 37, "スキルシート", font_size=16, bold=True, align="center", valign="vcenter")
    sheet.heights[0] = 28
    sheet.put(1, 28, 37, f"更新日 {model['build_month']}", align="right")
    rows = [
        ("氏　　名", profile["name"], "所　　属", profile["business"]),
        ("稼　　働", availability(profile), "語　　学",
         "\n".join(f"{s['label']} {s['level']}" for s in model["skills"] if s["category"] == "human-language")),
        ("ポートフォリオ", profile["portfolio_url"], "GitHub", profile["github_url"]),
        ("資　　格", qualifications(profile, "certificate", "test-result"), "学　　歴", qualifications(profile, "degree")),
    ]
    sheet.row = 2
    for left_label, left, right_label, right in rows:
        sheet.put(sheet.row, 0, 4, left_label, **label)
        sheet.put(sheet.row, 5, 18, left, **cell)
        sheet.put(sheet.row, 19, 23, right_label, **label)
        sheet.put(sheet.row, 24, 37, right, **cell)
        sheet.fit(sheet.row, left, 14)
        sheet.fit(sheet.row, right, 14)
        sheet.row += 1
    sheet.row += 1
    for title, text in (
        ("得意分野", profile["headline"]),
        ("得意技術", "\n".join(f"・{s}" for s in profile["strengths"])),
        ("自己PR", profile["summary"] + "\n\n" + profile["self_pr"]),
    ):
        sheet.put(sheet.row, 0, 4, title, **label)
        sheet.put(sheet.row, 5, 37, text, **cell)
        sheet.fit(sheet.row, text, 33)
        sheet.row += 1
    sheet.row += 1

    blocks = [
        ("開発言語", used_skills(model, "language")),
        ("フレームワーク・ライブラリ", used_skills(model, "framework", "library")),
        ("DB・インフラ・ツール", used_skills(model, "database", "cloud", "platform", "tool", "ai-tool", "method")),
    ]
    depth = max(len(skills) for _, skills in blocks)
    top = sheet.row
    sheet.put(top, 0, 1, "実務開発経験", top + depth + 1, **label, rotation=270)
    for index, (title, skills) in enumerate(blocks):
        first = 2 + index * 12
        sheet.put(top, first, first + 11, title, **head)
        sheet.put(top + 1, first, first + 7, "名称", **label)
        sheet.put(top + 1, first + 8, first + 11, "期間", **label)
        for offset in range(depth):
            skill = skills[offset] if offset < len(skills) else None
            sheet.put(top + 2 + offset, first, first + 7, skill["label"] if skill else "", **cell)
            sheet.put(top + 2 + offset, first + 8, first + 11, skill["duration"] if skill else "", **cell, align="right")
    sheet.row = top + depth + 3

    columns = [("No", 0, 1), ("期間", 2, 5), ("業種", 6, 8), ("会社名／プロジェクト名・業務内容", 9, 23),
               ("契約形態", 24, 25), ("規模人数", 26, 27), ("使用技術", 28, 31)]
    header = sheet.row
    for title, first, last in columns:
        sheet.put(header, first, last, title, **head)
    for offset, phase in enumerate(model["phase_labels"]):
        sheet.put(header, 32 + offset, 32 + offset, phase, **head, rotation=270, font_size=8)
    sheet.heights[header] = 62
    ws.repeat_rows(header)
    sheet.row = header + 1

    block_starts = []
    for project in model["projects"]:
        first_row = sheet.row
        block_starts.append(first_row)
        client = project["client"] + (f" ／ {project['end_client']}" if project["end_client"] else "")
        role = project["role"] + (f"（{project['focus']}）" if project["focus"] else "")
        lines = [
            (project["title"], {"bold": True, "font_size": 10}),
            (client, {"font_color": "#4A6A8F"}),
            (f"＜担当＞ {role}", {}),
            ("＜業務内容＞\n" + project["description"], {}),
        ]
        if project["achievements"]:
            lines.append(("＜実績＞\n" + "\n".join(f"・{a}" for a in project["achievements"]), {}))
        for offset, (text, props) in enumerate(lines):
            row = first_row + offset
            edges = {"left": 1, "right": 1, "border_color": ink, "top": int(offset == 0),
                     "bottom": int(offset == len(lines) - 1)}
            sheet.put(row, 9, 23, text, **edges, **props)
            sheet.fit(row, text, 15, props.get("font_size", 9))
        last_row = first_row + len(lines) - 1
        period = f"{project['duration']}\n\n{project['start']}\n〜\n{project['end']}"
        technology = "\n".join(label for labels in project["skills"].values() for label in labels)
        team = f"{project['team_size']}名" if project["team_size"] else ""
        merged = [(0, 1, str(project["number"])), (2, 5, period), (6, 8, project["industry"]),
                  (24, 25, project["contract_type"]), (26, 27, team), (28, 31, technology)]
        for first, last, value in merged:
            sheet.put(first_row, first, last, value, last_row, **cell, align="center", valign="vcenter")
            sheet.grow_to_fit(first_row, last_row, value, last - first + 1)
        for offset, on in enumerate(project["phases"]):
            col = 32 + offset
            sheet.put(first_row, col, col, "●" if on else "", last_row, **cell, align="center", valign="vcenter")
        sheet.row = last_row + 1
    ws.print_area(0, 0, sheet.row - 1, 37)
    sheet.finish(block_starts)


def variant_b(book, model):
    """The overview list: real columns without merges in the body, so agencies can filter, sort and edit."""
    ink, pale = "#2D4A3E", "#EDF3EF"
    font = "游ゴシック"
    profile = model["profile"]

    skills = Sheet(book, "プロフィール・スキル", 6, 1, font, landscape=True)
    ws = skills.ws
    widths = [22, 26, 11, 10, 13, 9]
    for col, width in enumerate(widths):
        ws.set_column(col, col, width)
    skills.scale = min(1.0, skills.page_width / sum(column_px(w) for w in widths))
    head = {"bold": True, "bg_color": ink, "font_color": "white", "valign": "vcenter", "border": 1,
            "border_color": ink}
    cell = {"border": 1, "border_color": "#9FB5AA"}
    skills.put(0, 0, 5, "スキルシート", font_size=14, bold=True, font_color=ink)
    skills.heights[0] = 24
    facts = [
        ("氏名", profile["name"]), ("得意分野", profile["headline"]), ("所属", profile["business"]),
        ("稼働", availability(profile)), ("ポートフォリオ", profile["portfolio_url"]), ("GitHub", profile["github_url"]),
        ("概要", profile["summary"]), ("得意技術", "\n".join(f"・{s}" for s in profile["strengths"])),
        ("資格", qualifications(profile, "certificate", "test-result")), ("学歴", qualifications(profile, "degree")),
        ("更新日", model["build_month"]),
    ]
    skills.row = 1
    for title, value in facts:
        skills.put(skills.row, 0, 0, title, **cell, bold=True, bg_color=pale)
        skills.put(skills.row, 1, 5, value, **cell)
        skills.heights[skills.row] = height_for(value, sum(widths[1:]), 9)
        skills.row += 1
    skills.row += 1
    top = skills.row
    for col, title in enumerate(["分類", "スキル", "経験年数", "経験月数", "最終使用", "案件数"]):
        skills.put(top, col, col, title, **head)
    rows = [s for s in model["skills"] if s["months"]]
    for offset, skill in enumerate(rows, 1):
        values = [skill["category_label"], skill["label"], skill["years"], skill["months"], skill["last_used"],
                  skill["projects"]]
        for col, value in enumerate(values):
            numeric = {"num_format": "0.0"} if col == 2 else {}
            skills.put(top + offset, col, col, value, **cell, **numeric)
    ws.conditional_format(top + 1, 2, top + len(rows), 2,
                          {"type": "data_bar", "bar_color": "#5E8C78", "bar_solid": True})
    ws.autofilter(top, 0, top + len(rows), 5)
    skills.row = top + len(rows) + 1
    skills.finish()

    sheet = Sheet(book, "経歴", 1, 1, font, landscape=True)
    ws = sheet.ws
    columns = [("No", 4), ("開始", 9), ("終了", 9), ("期間", 8), ("プロジェクト", 20), ("会社", 13), ("業種", 10),
               ("役割", 13), ("契約", 7), ("体制", 5), ("言語", 12), ("FW・ライブラリ", 14), ("DB・クラウド・基盤", 14),
               ("ツール・手法", 14)]
    columns += [(phase, 3.4) for phase in model["phase_labels"]]
    columns += [("業務内容", 60)]
    for col, (_, width) in enumerate(columns):
        ws.set_column(col, col, width)
    sheet.scale = min(1.0, sheet.page_width / sum(column_px(w) for _, w in columns))
    for col, (title, _) in enumerate(columns):
        rotated = {"rotation": 270, "align": "center", "font_size": 8} if 14 <= col < 20 else {}
        sheet.put(0, col, col, title, **head, **rotated)
    sheet.heights[0] = 58
    ws.freeze_panes(1, 5)
    ws.repeat_rows(0)
    for row, project in enumerate(model["projects"], 1):
        values = [
            project["number"], project["start"], project["end"], project["duration"], project["title"],
            project["client"] + (f" ／ {project['end_client']}" if project["end_client"] else ""),
            project["industry"], project["role"] + (f"（{project['focus']}）" if project["focus"] else ""),
            project["contract_type"], project["team_size"] or "",
            ", ".join(skills_of(project, "language")),
            ", ".join(skills_of(project, "framework", "library")),
            ", ".join(skills_of(project, "database", "cloud", "platform")),
            ", ".join(skills_of(project, "tool", "ai-tool", "method")),
        ]
        values += ["●" if on else "" for on in project["phases"]]
        values += [project["description"]]
        for col, value in enumerate(values):
            centred = {"align": "center"} if col in (0, 9) or 14 <= col < 20 else {}
            sheet.put(row, col, col, value, **cell, **centred)
            sheet.fit(row, value, columns[col][1] / sheet.unit)
    sheet.row = len(model["projects"]) + 1
    ws.autofilter(0, 0, sheet.row - 1, len(columns) - 1)
    sheet.finish()


def variant_c(book, model):
    """The project blocks: a 12-column grid, one labelled block per Project, never split across pages."""
    ink, accent, pale = "#3B2F5C", "#D9D2EA", "#F4F1F9"
    sheet = Sheet(book, "スキルシート", 12, 7.5, "游ゴシック")
    ws, profile = sheet.ws, model["profile"]
    rule = {"border_color": "#8A7FA8"}
    key = {"bold": True, "font_color": ink, "bg_color": pale, "border": 1, **rule}
    cell = {"border": 1, **rule}

    sheet.put(0, 0, 8, f"スキルシート　　{profile['name']}", bold=True, font_size=12, font_color="white",
              bg_color=ink, valign="vcenter")
    sheet.put(0, 9, 11, f"更新日 {model['build_month']}", font_color="white", bg_color=ink, align="right",
              valign="vcenter")
    sheet.heights[0] = 24
    facts = [
        ("得意分野", profile["headline"]), ("所属", profile["business"]), ("稼働", availability(profile)),
        ("URL", "\n".join(u for u in (profile["portfolio_url"], profile["github_url"]) if u)),
        ("資格", qualifications(profile, "certificate", "test-result")), ("学歴", qualifications(profile, "degree")),
        ("得意技術", "\n".join(f"・{s}" for s in profile["strengths"])),
        ("自己PR", profile["summary"] + "\n\n" + profile["self_pr"]),
    ]
    sheet.row = 2
    for title, value in facts:
        sheet.put(sheet.row, 0, 1, title, **key)
        sheet.put(sheet.row, 2, 11, value, **cell)
        sheet.fit(sheet.row, value, 10)
        sheet.row += 1
    sheet.row += 1
    for col, (first, last, title) in enumerate([(0, 2, "分類"), (3, 6, "スキル"), (7, 8, "経験"), (9, 11, "最終使用")]):
        sheet.put(sheet.row, first, last, title, **key)
    sheet.row += 1
    for category, name in model["category_labels"].items():
        rows = used_skills(model, category)
        if not rows:
            continue
        sheet.put(sheet.row, 0, 2, name, sheet.row + len(rows) - 1, **cell, font_size=8)
        for skill in rows:
            sheet.put(sheet.row, 3, 6, skill["label"], **cell)
            sheet.put(sheet.row, 7, 8, skill["duration"], **cell, align="right")
            sheet.put(sheet.row, 9, 11, skill["last_used"], **cell, font_size=8)
            sheet.row += 1

    block_starts = []
    for project in model["projects"]:
        sheet.row += 1
        top = sheet.row
        block_starts.append(top)
        sheet.put(top, 0, 0, f"No.{project['number']}", bold=True, font_color=ink, bg_color=accent, valign="vcenter")
        sheet.put(top, 1, 7, project["title"], bold=True, font_size=10, bg_color=accent, valign="vcenter")
        sheet.put(top, 8, 11, f"{project['start']} 〜 {project['end']}（{project['duration']}）", bg_color=accent,
                  align="right", valign="vcenter")
        sheet.heights[top] = 20
        client = project["client"] + (f" ／ {project['end_client']}" if project["end_client"] else "")
        role = project["role"] + (f"（{project['focus']}）" if project["focus"] else "")
        team = f"{project['team_size']}名" if project["team_size"] else "—"
        facts = [[("会社", client), ("業種", project["industry"]), ("契約", project["contract_type"])],
                 [("役割", role), ("体制", team), ("", "")]]
        for line in facts:
            sheet.row += 1
            for index, (title, value) in enumerate(line):
                sheet.put(sheet.row, index * 4, index * 4, title, bold=True, font_color=ink)
                sheet.put(sheet.row, index * 4 + 1, index * 4 + 3, value)
                sheet.fit(sheet.row, value, 3)
        sheet.row += 1
        for index, (phase, on) in enumerate(zip(model["phase_labels"], project["phases"])):
            colours = {"bg_color": ink, "font_color": "white"} if on else {"bg_color": pale, "font_color": "#AAAAAA"}
            sheet.put(sheet.row, index * 2, index * 2 + 1, phase, **colours, align="center", font_size=8,
                      border=1, border_color="white")
        for category, name in model["category_labels"].items():
            if category in project["skills"]:
                sheet.row += 1
                text = "、".join(project["skills"][category])
                sheet.put(sheet.row, 0, 1, name, bold=True, font_color=ink, font_size=8)
                sheet.put(sheet.row, 2, 11, text)
                sheet.fit(sheet.row, text, 10)
        sheet.row += 1
        sheet.put(sheet.row, 0, 11, "【業務内容】", bold=True, font_color=ink)
        sheet.row += 1
        sheet.put(sheet.row, 0, 11, project["description"])
        sheet.fit(sheet.row, project["description"], 12)
        if project["achievements"]:
            sheet.row += 1
            sheet.put(sheet.row, 0, 11, "【実績】", bold=True, font_color=ink)
            for achievement in project["achievements"]:
                sheet.row += 1
                sheet.put(sheet.row, 0, 11, f"・{achievement}")
                sheet.fit(sheet.row, f"・{achievement}", 12)
        sheet.row += 1
    ws.print_area(0, 0, sheet.row - 1, 11)
    sheet.finish(block_starts)


def main(model_path, variant, out):
    with open(model_path, encoding="utf-8") as file:
        model = json.load(file)
    book = xlsxwriter.Workbook(out)
    {"a": variant_a, "b": variant_b, "c": variant_c}[variant](book, model)
    book.close()


if __name__ == "__main__":
    main(*sys.argv[1:])
