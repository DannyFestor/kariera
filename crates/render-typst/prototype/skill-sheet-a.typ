// PROTOTYPE (throwaway), Skill Sheet variant A: the agency format (Findy style). A4 portrait; profile, strengths and
// years per Skill in three columns, then one tall row per Project with the phase ● columns.
#let data = json(sys.inputs.data)
#let profile = data.profile
#let ink = rgb("#1f3a5f")
#let pale = rgb("#e8eef5")

#set page(
  paper: "a4",
  margin: (x: 10mm, top: 12mm, bottom: 14mm),
  footer: context align(center, text(size: 7pt)[#counter(page).display(
    "1 / 1",
    both: true,
  )]),
)
#set text(font: "BIZ UDPGothic", size: 8pt, lang: "ja")
#set par(leading: 0.45em, spacing: 0.7em)
#set table(stroke: 0.5pt + ink, inset: (x: 4pt, y: 3.5pt))

#let label(..span, body) = table.cell(
  ..span,
  fill: pale,
  align: center + horizon,
  text(
    weight: "bold",
    body,
  ),
)
#let head(..span, body) = table.cell(
  ..span,
  fill: ink,
  align: center + horizon,
  text(
    fill: white,
    weight: "bold",
    body,
  ),
)
#let vertical(body) = stack(dir: ttb, spacing: 1pt, ..body
  .clusters()
  .map(c => [#c]))
#let availability = {
  let (low, high) = profile.days_per_week
  let days = if low == none { "" } else if low == high { [週#low 日] } else {
    [週#low 〜#high 日]
  }
  if profile.available_from == "" { days } else {
    [#profile.available_from 〜 #days]
  }
}
#let in-category(..names) = data.skills.filter(s => (
  s.months > 0 and names.pos().contains(s.category)
))
#let qualifications(category) = {
  let matching = profile.qualifications.filter(q => q.category == category)
  matching.map(q => [#q.date #q.label]).join(linebreak())
}

#grid(
  columns: (1fr, auto),
  align(center, text(
    size: 15pt,
    weight: "bold",
    tracking: 0.3em,
  )[スキルシート]),
  align(right + bottom)[更新日 #data.build_month],
)
#v(4pt)

#table(
  columns: (20mm, 1fr, 20mm, 1fr),
  label[氏　　名], profile.name, label[所　　属], profile.business,
  label[稼　　働],
  availability,
  label[語　　学],
  data
    .skills
    .filter(s => s.category == "human-language")
    .map(s => [#s.label #s.level])
    .join(linebreak()),

  label[ポートフォリオ],
  profile.portfolio_url,
  label[GitHub],
  profile.github_url,

  label[資　　格],
  qualifications("certificate") + linebreak() + qualifications("test-result"),
  label[学　　歴],
  qualifications("degree"),
)
#v(4pt)
#table(
  columns: (20mm, 1fr),
  label[得意分野], profile.headline,
  label[得意技術], profile.strengths.map(s => [・#s]).join(linebreak()),
  label[自己PR], profile.summary + parbreak() + profile.self_pr,
)
#v(4pt)

#let skill-columns = (
  ([開発言語], in-category("language")),
  ([フレームワーク・ライブラリ], in-category("framework", "library")),
  (
    [DB・インフラ・ツール],
    in-category("database", "cloud", "platform", "tool", "ai-tool", "method"),
  ),
)
#let depth = calc.max(..skill-columns.map(c => c.at(1).len()))
#table(
  columns: (7mm,) + (1fr, 19mm) * 3,
  label(rowspan: 2 + depth, vertical("実務開発経験")),
  ..skill-columns.map(c => head(colspan: 2, c.at(0))),
  ..(label[名称], label[期間]) * 3,
  ..range(depth)
    .map(i => skill-columns.map(c => {
      let skill = c.at(1).at(i, default: none)
      if skill == none { ([], []) } else {
        (skill.label, align(right, skill.duration))
      }
    }))
    .flatten(),
)
#v(4pt)

#let phase-count = data.phase_labels.len()
#table(
  columns: (6mm, 17mm, 12mm, 1fr, 12mm, 10mm, 24mm) + (5mm,) * phase-count,
  table.header(
    repeat: true,
    head[No],
    head[期間],
    head[業種],
    head[会社名／プロジェクト名・業務内容],
    head[契約\ 形態],
    head[規模\ 人数],
    head[使用技術],
    ..data.phase_labels.map(p => head(text(size: 7pt, vertical(p)))),
  ),
  ..data
    .projects
    .map(p => (
      align(center + horizon, str(p.number)),
      align(center + horizon)[#p.duration\ \ #p.start\ 〜\ #p.end],
      align(center + horizon, p.industry),
      [
        #text(weight: "bold", size: 9pt, p.title) \
        #text(
          fill: ink.lighten(20%),
          p.client + if p.end_client != "" { [ ／ #p.end_client] },
        )

        ＜担当＞ #p.role#if p.focus != "" [（#p.focus）]

        ＜業務内容＞ \
        #p.description

        #if (
          p.achievements.len() > 0
        ) [＜実績＞ \ #p.achievements.map(a => [・#a]).join(linebreak())]
      ],
      align(center + horizon, p.contract_type),
      align(center + horizon, if p.team_size == none { [] } else {
        [#p.team_size 名]
      }),
      align(horizon, p.skills.values().flatten().join(linebreak())),
      ..p.phases.map(on => align(center + horizon, if on [●])),
    ))
    .flatten(),
)
