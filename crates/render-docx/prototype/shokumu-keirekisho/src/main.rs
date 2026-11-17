//! PROTOTYPE, throwaway: the 職務経歴書 variant B (crates/render-typst/prototype/shokumu-keirekisho/variant-b.typ)
//! as DOCX, built only from what docx-rs 0.4.22 can produce. Reads the JSON of the prototype exporter.
//!
//! `cargo run -- <data.json> <today YYYY-MM-DD> <out.docx>`

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;

use docx_rs::{
    AlignmentType, BorderType, Docx, Footer, Header, LineSpacing, NumPages,
    PageMargin, PageNum, Paragraph, ParagraphBorder, ParagraphBorderPosition, Run, RunFonts,
    Shading, SpecialIndentType, Tab, TabValueType, Table, TableCell, TableCellBorder,
    TableCellBorderPosition, TableCellBorders, TableCellMargins, TableLayoutType, TableRow,
    WidthType,
};
use serde_json::Value;

const FONT: &str = "BIZ UDPGothic";
const ACCENT: &str = "1F3A5F";
const MUTED: &str = "6E6E6E";
const CARD: &str = "F7F7F7";
const UNTRANSLATED: &str = "C0392B";
// A4 in twentieths of a point; the text width is the page minus 16 mm on each side.
const PAGE: (u32, u32) = (11906, 16838);
const MARGIN_X: i32 = 907;
const TEXT_WIDTH: usize = 11906 - 2 * 907;
const DETAIL_YEARS: i32 = 2;
const SKILL_WINDOW: i32 = 5;
const PHASES: [(&str, &str); 6] = [
    ("requirements", "要件定義"),
    ("basic-design", "基本設計"),
    ("detailed-design", "詳細設計"),
    ("implementation", "製造"),
    ("testing", "テスト"),
    ("operations", "運用・保守"),
];
const CATEGORY_ORDER: [&str; 9] = [
    "language",
    "framework",
    "library",
    "database",
    "cloud",
    "platform",
    "tool",
    "ai-tool",
    "method",
];

/// A piece of text in Japanese, or the English with a visible marker when the translation is missing.
struct Ja {
    text: String,
    translated: bool,
}

fn ja(value: &Value) -> Option<Ja> {
    match (value["ja"].as_str(), value["en"].as_str()) {
        (Some(text), _) => Some(Ja {
            text: text.to_owned(),
            translated: true,
        }),
        (None, Some(text)) => Some(Ja {
            text: format!("［未翻訳］{text}"),
            translated: false,
        }),
        (None, None) => None,
    }
}

fn ja_paragraphs(value: &Value) -> Vec<Ja> {
    match (value["ja"].as_str(), value["en"].as_str()) {
        (Some(text), _) => text
            .split("\n\n")
            .map(|paragraph| Ja {
                text: paragraph.replace('\n', ""),
                translated: true,
            })
            .collect(),
        (None, Some(text)) => vec![Ja {
            text: format!("［未翻訳］{}", text.replace("\n\n", " ")),
            translated: false,
        }],
        (None, None) => Vec::new(),
    }
}

fn ja_bullets(value: &Value) -> Vec<Ja> {
    let (list, translated) = match (value["ja"].as_array(), value["en"].as_array()) {
        (Some(list), _) => (list, true),
        (None, Some(list)) => (list, false),
        (None, None) => return Vec::new(),
    };
    list.iter()
        .filter_map(Value::as_str)
        .map(|text| Ja {
            text: if translated {
                text.to_owned()
            } else {
                format!("［未翻訳］{text}")
            },
            translated,
        })
        .collect()
}

fn text_of(value: &Value) -> String {
    ja(value).map(|text| text.text).unwrap_or_default()
}

fn month_index(month: &str) -> i32 {
    let (year, number) = month.split_once('-').unwrap_or(("0", "1"));
    year.parse::<i32>().unwrap_or(0) * 12 + number.parse::<i32>().unwrap_or(1) - 1
}

fn format_month(month: &Value) -> String {
    month.as_str().map_or_else(
        || "現在".to_owned(),
        |month| {
            let index = month_index(month);
            format!("{}年{}月", index / 12, index % 12 + 1)
        },
    )
}

fn format_date(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    format!(
        "{}年{}月{}日",
        parts[0],
        parts[1].trim_start_matches('0'),
        parts[2].trim_start_matches('0')
    )
}

fn period(item: &Value) -> String {
    format!(
        "{}～{}",
        format_month(&item["start"]),
        format_month(&item["end"])
    )
}

fn format_duration(months: usize) -> String {
    let (years, rest) = (months / 12, months % 12);
    match (years, rest) {
        (0, rest) => format!("{rest}ヶ月"),
        (years, 0) => format!("{years}年"),
        (years, rest) => format!("{years}年{rest}ヶ月"),
    }
}

struct Data {
    json: Value,
    today: i32,
}

impl Data {
    fn end_index(&self, item: &Value) -> i32 {
        item["end"].as_str().map_or(self.today, month_index)
    }

    fn months_of(&self, item: &Value) -> std::ops::RangeInclusive<i32> {
        month_index(item["start"].as_str().unwrap_or_default())..=self.end_index(item)
    }

    fn is_detailed(&self, project: &Value) -> bool {
        self.end_index(project) > self.today - 12 * DETAIL_YEARS
    }

    fn list(&self, key: &str) -> &[Value] {
        self.json[key].as_array().map_or(&[], Vec::as_slice)
    }

    fn find<'a>(&'a self, key: &str, name: &str) -> &'a Value {
        self.list(key)
            .iter()
            .find(|item| item["name"] == name)
            .unwrap_or(&Value::Null)
    }

    fn skill_label(&self, name: &str) -> String {
        self.find("skills", name)["ja"]
            .as_str()
            .unwrap_or(name)
            .to_owned()
    }

    fn newest_first<'a>(&self, items: impl Iterator<Item = &'a Value>) -> Vec<&'a Value> {
        let mut items: Vec<&Value> = items.collect();
        items.sort_by_key(|item| {
            std::cmp::Reverse(month_index(item["start"].as_str().unwrap_or_default()))
        });
        items
    }

    fn projects_of(&self, engagement: &Value) -> Vec<&Value> {
        self.newest_first(
            self.list("projects")
                .iter()
                .filter(|project| project["engagement"] == engagement["name"]),
        )
    }

    /// Per Skill: (months used in Projects, overlaps counted once; last month used).
    fn skill_stats(&self) -> BTreeMap<String, (usize, i32)> {
        let mut months: BTreeMap<String, Vec<i32>> = BTreeMap::new();
        for project in self.list("projects") {
            for skill in project["skills"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                months
                    .entry(skill.to_owned())
                    .or_default()
                    .extend(self.months_of(project));
            }
        }
        months
            .into_iter()
            .map(|(name, mut list)| {
                list.sort_unstable();
                list.dedup();
                let last = list.last().copied().unwrap_or_default();
                (name, (list.len(), last))
            })
            .collect()
    }
}

fn run(text: &str) -> Run {
    Run::new().add_text(text)
}

fn ja_run(text: &Ja) -> Run {
    let run = run(&text.text);
    if text.translated {
        run
    } else {
        run.color(UNTRANSLATED)
    }
}

fn spacing(after: u32) -> LineSpacing {
    LineSpacing::new().after(after).before(0)
}

fn paragraph(text: &Ja) -> Paragraph {
    Paragraph::new()
        .add_run(ja_run(text))
        .line_spacing(spacing(80))
}

fn bullet(text: &Ja) -> Paragraph {
    Paragraph::new()
        .add_run(run("■").color(ACCENT))
        .add_run(run(" "))
        .add_run(ja_run(text))
        .indent(Some(240), Some(SpecialIndentType::Hanging(240)), None, None)
        .line_spacing(spacing(20))
}

fn section(title: &str) -> Paragraph {
    let mut heading = Paragraph::new()
        .add_run(run(title).bold().size(22).color(ACCENT))
        .keep_next(true)
        .line_spacing(spacing(100).before(240));
    heading.property = heading.property.set_border(
        ParagraphBorder::new(ParagraphBorderPosition::Bottom)
            .val(BorderType::Single)
            .size(8)
            .color(ACCENT)
            .space(2),
    );
    heading
}

fn subheading(title: &str) -> Paragraph {
    Paragraph::new()
        .add_run(run(title).bold().color(ACCENT))
        .keep_next(true)
        .line_spacing(spacing(40))
}

fn no_borders() -> TableCellBorders {
    TableCellBorders::with_empty()
}

fn plain_cell(width: usize, paragraphs: Vec<Paragraph>) -> TableCell {
    paragraphs.into_iter().fold(
        TableCell::new()
            .width(width, WidthType::Dxa)
            .set_borders(no_borders()),
        TableCell::add_paragraph,
    )
}

fn plain_table(widths: &[usize], rows: Vec<Vec<TableCell>>) -> Table {
    Table::new(rows.into_iter().map(TableRow::new).collect())
        .set_grid(widths.to_vec())
        .layout(TableLayoutType::Fixed)
        .width(widths.iter().sum(), WidthType::Dxa)
        .margins(TableCellMargins::new().margin(0, 60, 0, 0))
        .clear_all_border()
}

fn title_block(data: &Data, today: &str) -> Vec<Paragraph> {
    let profile = &data.json["profile"];
    vec![
        Paragraph::new()
            .add_run(run("職務経歴書").bold().size(40).color(ACCENT))
            .line_spacing(spacing(0)),
        Paragraph::new()
            .add_run(
                run(profile["name"].as_str().unwrap_or_default())
                    .bold()
                    .size(24),
            )
            .add_run(run("　"))
            .add_run(run(&text_of(&profile["headline"])).color(MUTED))
            .add_run(
                Run::new()
                    .add_tab()
                    .add_text(format!("{}現在", format_date(today)))
                    .size(17)
                    .color(MUTED),
            )
            .add_tab(Tab::new().val(TabValueType::Right).pos(TEXT_WIDTH))
            .line_spacing(spacing(120)),
    ]
}

fn qualification_lines(data: &Data) -> Vec<Paragraph> {
    let mut qualifications: Vec<&Value> = data
        .list("qualifications")
        .iter()
        .filter(|qualification| qualification["category"]["slug"] != "degree")
        .collect();
    qualifications.sort_by_key(|qualification| {
        month_index(qualification["date"].as_str().unwrap_or_default())
    });
    let mut lines: Vec<Paragraph> = qualifications
        .into_iter()
        .map(|qualification| {
            let label = qualification["ja"]
                .as_str()
                .or(qualification["name"].as_str())
                .unwrap_or_default();
            let mut text = label.to_owned();
            if let Some(level) = qualification["level"]
                .as_str()
                .filter(|level| !label.contains(level))
            {
                text.push_str(&format!(" {level}"));
            }
            if let Some(score) = qualification["score"].as_str() {
                text.push_str(&format!("（{score}）"));
            }
            Paragraph::new()
                .add_run(run(&format_month(&qualification["date"])).color(MUTED))
                .add_run(run("　"))
                .add_run(run(&text))
                .line_spacing(spacing(20))
        })
        .collect();
    for language in data
        .list("skills")
        .iter()
        .filter(|skill| skill["category"]["slug"] == "human-language")
    {
        let name = language["ja"]
            .as_str()
            .or(language["name"].as_str())
            .unwrap_or_default();
        lines.push(
            Paragraph::new()
                .add_run(run("語学").color(MUTED))
                .add_run(run("　"))
                .add_run(run(&format!("{name}（{}）", text_of(&language["level"]))))
                .line_spacing(spacing(20)),
        );
    }
    lines
}

fn overview(data: &Data) -> Table {
    let half = TEXT_WIDTH / 2;
    let strengths: Vec<Paragraph> = std::iter::once(section("得意分野"))
        .chain(
            ja_bullets(&data.json["profile"]["strengths"])
                .iter()
                .map(bullet),
        )
        .collect();
    let qualifications: Vec<Paragraph> = std::iter::once(section("資格・語学"))
        .chain(qualification_lines(data))
        .collect();
    plain_table(
        &[half, half],
        vec![vec![
            plain_cell(half, strengths),
            plain_cell(half, qualifications),
        ]],
    )
}

fn skill_block(
    data: &Data,
    category: &Value,
    skills: &[(String, usize)],
    width: usize,
) -> TableCell {
    let name_width = width - 1100;
    let rows = skills
        .iter()
        .map(|(name, months)| {
            vec![
                plain_cell(
                    name_width,
                    vec![Paragraph::new().add_run(run(&data.skill_label(name)).size(16))],
                ),
                plain_cell(
                    1100,
                    vec![
                        Paragraph::new()
                            .add_run(run(&format_duration(*months)).size(15))
                            .align(AlignmentType::Right),
                    ],
                ),
            ]
        })
        .collect();
    TableCell::new()
        .width(width, WidthType::Dxa)
        .set_borders(no_borders())
        .add_paragraph(subheading(category["ja"].as_str().unwrap_or_default()).size(16))
        .add_table(plain_table(&[name_width, 1100], rows))
        .add_paragraph(Paragraph::new().line_spacing(spacing(0)).size(8))
}

fn skills_table(data: &Data) -> Table {
    let stats = data.skill_stats();
    let width = TEXT_WIDTH / 3;
    let blocks: Vec<TableCell> = CATEGORY_ORDER
        .iter()
        .filter_map(|slug| {
            let mut skills: Vec<(String, usize)> = data
                .list("skills")
                .iter()
                .filter(|skill| skill["category"]["slug"] == *slug)
                .filter_map(|skill| {
                    let name = skill["name"].as_str()?;
                    let (months, last) = stats.get(name)?;
                    (*last > data.today - 12 * SKILL_WINDOW).then(|| (name.to_owned(), *months))
                })
                .collect();
            skills.sort_by_key(|(_, months)| std::cmp::Reverse(*months));
            let category = &data
                .list("skills")
                .iter()
                .find(|skill| skill["category"]["slug"] == *slug)?["category"];
            (!skills.is_empty()).then(|| skill_block(data, category, &skills, width))
        })
        .collect();
    let mut rows: Vec<Vec<TableCell>> = Vec::new();
    let mut blocks = blocks.into_iter().peekable();
    while blocks.peek().is_some() {
        let mut row: Vec<TableCell> = blocks.by_ref().take(3).collect();
        while row.len() < 3 {
            row.push(plain_cell(width, vec![Paragraph::new()]));
        }
        rows.push(row);
    }
    plain_table(&[width, width, width], rows)
}

fn phase_marks(project: &Value) -> Paragraph {
    let used: Vec<&str> = project["phases"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|phase| phase["slug"].as_str())
        .collect();
    PHASES
        .iter()
        .fold(Paragraph::new(), |paragraph, (slug, label)| {
            if used.contains(slug) {
                paragraph.add_run(run(&format!("●{label}　")).bold().color(ACCENT))
            } else {
                paragraph.add_run(run(&format!("○{label}　")).color("B4B4B4"))
            }
        })
}

fn facts(data: &Data, project: &Value, width: usize) -> Table {
    let mut role = text_of(&project["role"]);
    if let Some(focus) = ja(&project["focus"]) {
        role.push_str(&format!("（{}）", focus.text));
    }
    let mut rows = vec![("役割", Paragraph::new().add_run(run(&role)))];
    if let Some(size) = project["team_size"].as_u64() {
        rows.push(("体制", Paragraph::new().add_run(run(&format!("{size}名")))));
    }
    if let Some(client) = project["end_client"].as_str() {
        rows.push(("エンドクライアント", Paragraph::new().add_run(run(client))));
    }
    rows.push(("担当工程", phase_marks(project)));
    let skills: Vec<String> = project["skills"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|name| data.skill_label(name))
        .collect();
    rows.push(("環境", Paragraph::new().add_run(run(&skills.join(" / ")))));
    let label_width = 1650;
    plain_table(
        &[label_width, width - label_width],
        rows.into_iter()
            .map(|(label, value)| {
                vec![
                    plain_cell(
                        label_width,
                        vec![Paragraph::new().add_run(run(label).color(MUTED)).size(17)],
                    ),
                    plain_cell(width - label_width, vec![value.size(17)]),
                ]
            })
            .collect(),
    )
}

fn card(data: &Data, project: &Value) -> Table {
    let inner = TEXT_WIDTH - 400;
    let header = Paragraph::new()
        .add_run(run(&period(project)).bold().color(ACCENT))
        .add_run(run("　"))
        .add_run(run(&text_of(&project["title"])).bold())
        .add_run(
            Run::new()
                .add_tab()
                .add_text(format!(" {} ", text_of(&project["industry"])))
                .size(16)
                .shading(Shading::new().fill("E1E1E1")),
        )
        .add_tab(Tab::new().val(TabValueType::Right).pos(inner))
        .keep_next(true)
        .line_spacing(spacing(80));
    let mut cell = TableCell::new()
        .width(TEXT_WIDTH, WidthType::Dxa)
        .shading(Shading::new().fill(CARD))
        .set_borders(
            no_borders().set(
                TableCellBorder::new(TableCellBorderPosition::Left)
                    .border_type(BorderType::Single)
                    .size(16)
                    .color(ACCENT),
            ),
        )
        .add_paragraph(header)
        .add_table(facts(data, project, inner))
        .add_paragraph(Paragraph::new().line_spacing(spacing(0)).size(8));
    for text in ja_paragraphs(&project["description"]) {
        cell = cell.add_paragraph(paragraph(&text));
    }
    let achievements = ja_bullets(&project["achievements"]);
    if !achievements.is_empty() {
        cell = cell.add_paragraph(subheading("実績"));
        for text in &achievements {
            cell = cell.add_paragraph(bullet(text));
        }
    }
    Table::new(vec![TableRow::new(vec![cell])])
        .set_grid(vec![TEXT_WIDTH])
        .layout(TableLayoutType::Fixed)
        .width(TEXT_WIDTH, WidthType::Dxa)
        .margins(TableCellMargins::new().margin(140, 200, 140, 200))
        .clear_all_border()
}

fn compact_table(data: &Data, projects: &[&Value]) -> Table {
    let widths = [1360, TEXT_WIDTH - 1360 - 1930, 1930];
    let rule = || {
        no_borders().set(
            TableCellBorder::new(TableCellBorderPosition::Bottom)
                .size(4)
                .color("D7D7D7"),
        )
    };
    let rows = projects
        .iter()
        .map(|project| {
            let start = format_month(&project["start"]);
            let end = format_month(&project["end"]);
            let period = if project["start"] == project["end"] {
                start
            } else {
                format!("{start}～\n{end}")
            };
            let period_cell = period.split('\n').fold(TableCell::new(), |cell, line| {
                cell.add_paragraph(Paragraph::new().add_run(run(line).size(16).color(MUTED)))
            });
            let skills: Vec<String> = project["skills"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(|name| data.skill_label(name))
                .collect();
            let summary = TableCell::new()
                .add_paragraph(
                    Paragraph::new()
                        .add_run(run(&text_of(&project["title"])).bold())
                        .add_run(
                            run(&format!("（{}）", text_of(&project["industry"])))
                                .size(15)
                                .color(MUTED),
                        ),
                )
                .add_paragraph(
                    Paragraph::new().add_run(run(&skills.join(" / ")).size(15).color(MUTED)),
                );
            let role = TableCell::new()
                .add_paragraph(Paragraph::new().add_run(run(&text_of(&project["role"])).size(16)));
            TableRow::new(
                [period_cell, summary, role]
                    .into_iter()
                    .zip(widths)
                    .map(|(cell, width)| cell.width(width, WidthType::Dxa).set_borders(rule()))
                    .collect(),
            )
            .cant_split()
        })
        .collect();
    Table::new(rows)
        .set_grid(widths.to_vec())
        .layout(TableLayoutType::Fixed)
        .width(TEXT_WIDTH, WidthType::Dxa)
        .margins(TableCellMargins::new().margin(50, 60, 50, 60))
        .clear_all_border()
}

fn engagement_heading(data: &Data, engagement: &Value, detailed: bool) -> Vec<Paragraph> {
    let organization = data.find(
        "organizations",
        engagement["organization"].as_str().unwrap_or_default(),
    );
    let mut kind = engagement["type"]["ja"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let agencies: Vec<&str> = engagement["agencies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    if !agencies.is_empty() {
        kind.push_str(&format!("（{}経由）", agencies.join("、")));
    }
    let mut lines = vec![
        Paragraph::new()
            .add_run(
                run(organization["name"].as_str().unwrap_or_default())
                    .bold()
                    .size(21),
            )
            .add_run(run(&format!("　{}", period(engagement))).color(MUTED))
            .add_run(Run::new().add_tab().add_text(kind))
            .add_tab(Tab::new().val(TabValueType::Right).pos(TEXT_WIDTH))
            .keep_next(true)
            .line_spacing(spacing(20).before(200)),
    ];
    if let Some(role) = ja(&engagement["role"]) {
        lines.push(
            Paragraph::new()
                .add_run(run(&format!("役職：{}", role.text)).size(17))
                .keep_next(true),
        );
    }
    if detailed {
        for text in ja_paragraphs(&organization["description"]) {
            lines.push(
                Paragraph::new()
                    .add_run(
                        run(&format!("事業内容：{}", text.text))
                            .size(17)
                            .color(MUTED),
                    )
                    .keep_next(true),
            );
        }
    }
    lines.push(
        Paragraph::new()
            .line_spacing(spacing(0))
            .size(8)
            .keep_next(true),
    );
    lines
}

fn career(mut docx: Docx, data: &Data) -> Docx {
    let mut engagements: Vec<&Value> = data.newest_first(data.list("engagements").iter());
    engagements.dedup();
    for engagement in engagements {
        let projects = data.projects_of(engagement);
        let (detailed, brief): (Vec<&Value>, Vec<&Value>) = projects
            .into_iter()
            .partition(|project| data.is_detailed(project));
        for line in engagement_heading(data, engagement, !detailed.is_empty()) {
            docx = docx.add_paragraph(line);
        }
        for project in detailed {
            docx = docx
                .add_table(card(data, project))
                .add_paragraph(Paragraph::new().size(8));
        }
        if !brief.is_empty() {
            docx = docx.add_table(compact_table(data, &brief));
        }
    }
    docx
}

fn page_frame(docx: Docx, data: &Data, today: &str) -> Docx {
    let name = data.json["profile"]["name"].as_str().unwrap_or_default();
    let header = Header::new().add_paragraph(
        Paragraph::new()
            .add_run(run(&format!("職務経歴書　{name}")).size(15).color(MUTED))
            .add_run(
                Run::new()
                    .add_tab()
                    .add_text(format!("{}現在", format_date(today)))
                    .size(15)
                    .color(MUTED),
            )
            .add_tab(Tab::new().val(TabValueType::Right).pos(TEXT_WIDTH)),
    );
    let footer = Footer::new().add_paragraph(
        Paragraph::new()
            .add_page_num(PageNum::new())
            .add_run(run(" / "))
            .add_num_pages(NumPages::new())
            .size(15)
            .color(MUTED)
            .align(AlignmentType::Right),
    );
    docx.header(header)
        .first_header(Header::new())
        .footer(footer.clone())
        .first_footer(footer)
        .title_pg()
}

fn document(data: &Data, today: &str) -> Docx {
    let profile = &data.json["profile"];
    let fonts = RunFonts::new()
        .ascii(FONT)
        .hi_ansi(FONT)
        .east_asia(FONT)
        .cs(FONT);
    let mut docx = Docx::new()
        .page_size(PAGE.0, PAGE.1)
        .page_margin(
            PageMargin::new()
                .top(1020)
                .bottom(1020)
                .left(MARGIN_X)
                .right(MARGIN_X)
                .header(500)
                .footer(500),
        )
        .default_fonts(fonts)
        .default_size(18);
    docx = page_frame(docx, data, today);
    for line in title_block(data, today) {
        docx = docx.add_paragraph(line);
    }
    docx = docx.add_paragraph(section("職務要約"));
    for text in ja_paragraphs(&profile["summary"]) {
        docx = docx.add_paragraph(paragraph(&text));
    }
    docx = docx
        .add_table(overview(data))
        .add_paragraph(section("テクニカルスキル"))
        .add_paragraph(
            Paragraph::new().add_run(
                run(&format!(
                    "直近{SKILL_WINDOW}年以内に案件で使用したスキル。期間は案件での使用期間の合計。"
                ))
                .size(16)
                .color(MUTED),
            ),
        )
        .add_table(skills_table(data))
        .add_paragraph(section("職務経歴"))
        .add_paragraph(
            Paragraph::new()
                .add_run(
                    run(&format!(
                        "直近{DETAIL_YEARS}年以内の案件は詳細を、それ以前の案件は概要を記載。"
                    ))
                    .size(16)
                    .color(MUTED),
                )
                .keep_next(true),
        );
    docx = career(docx, data).add_paragraph(section("自己PR"));
    for text in ja_paragraphs(&profile["self_pr"]) {
        docx = docx.add_paragraph(paragraph(&text));
    }
    let links: Vec<String> = [
        ("ポートフォリオ", "portfolio_url"),
        ("GitHub", "github_url"),
    ]
    .iter()
    .filter_map(|(label, key)| profile[*key].as_str().map(|url| format!("{label}：{url}")))
    .collect();
    if !links.is_empty() {
        docx = docx.add_paragraph(Paragraph::new().add_run(run(&links.join("　　")).size(17)));
    }
    docx.add_paragraph(
        Paragraph::new()
            .add_run(run("以上"))
            .align(AlignmentType::Right),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [data_path, today, output] = arguments.as_slice() else {
        return Err("usage: <data.json> <today YYYY-MM-DD> <out.docx>".into());
    };
    let data = Data {
        json: serde_json::from_str(&std::fs::read_to_string(data_path)?)?,
        today: month_index(&today[..7]),
    };
    document(&data, today).build().pack(File::create(output)?)?;
    Ok(())
}
