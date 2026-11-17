// PROTOTYPE (throwaway), Skill Sheet variant B: the overview list. A4 landscape; a one-line profile, the years per
// Skill as a dense matrix by category, then one compact row per Project with the technology split into category
// columns and the phase ● columns. Of the prose only the description is printed, without the achievements.
#let data = json(sys.inputs.data)
#let profile = data.profile
#let ink = rgb("#2d4a3e")
#let pale = rgb("#edf3ef")
#let rule = 0.4pt + rgb("#9fb5aa")

#set page(
  paper: "a4",
  flipped: true,
  margin: (x: 10mm, top: 10mm, bottom: 12mm),
  footer: context {
    set text(size: 7pt, fill: ink)
    grid(
      columns: (1fr, 1fr),
      [スキルシート　#profile.name],
      align(right, counter(page).display("1 / 1", both: true)),
    )
  },
)
#set text(font: "Noto Sans JP", size: 7.5pt, lang: "ja")
#set par(leading: 0.4em, spacing: 0.6em)
#set table(stroke: rule, inset: (x: 3pt, y: 3pt))

#let head(..args, body) = table.cell(
  ..args,
  fill: ink,
  align: center + horizon,
  text(
    fill: white,
    weight: "bold",
    body,
  ),
)
#let years-bar(months) = box(width: 18mm, height: 4pt, {
  let max = calc.max(..data.skills.map(s => s.months))
  place(rect(width: 100%, height: 100%, fill: pale, stroke: none))
  place(rect(width: 100% * months / max, height: 100%, fill: ink, stroke: none))
})

#grid(
  columns: (auto, 1fr, auto),
  gutter: 8pt,
  text(size: 14pt, weight: "bold", fill: ink)[スキルシート],
  align(bottom, text(
    size: 9pt,
  )[#profile.name　｜　#profile.headline　｜　#profile.business]),
  align(right + bottom)[更新日 #data.build_month],
)
#line(length: 100%, stroke: 1.2pt + ink)

#grid(
  columns: (1fr, 1fr),
  gutter: 10pt,
  [
    #text(weight: "bold", fill: ink)[概要] \
    #profile.summary
  ],
  [
    #text(weight: "bold", fill: ink)[得意技術] \
    #profile.strengths.map(s => [・#s]).join(linebreak())
    #v(2pt)
    #text(weight: "bold", fill: ink)[稼働・連絡先] \
    #profile.available_from #(
      profile.days_per_week.filter(d => d != none).map(str).join("〜")
    )
    #profile.portfolio_url　#profile.github_url
  ],
)
#v(4pt)

#let groups = (
  ("language",),
  ("framework", "library"),
  ("database", "cloud", "platform"),
  ("tool", "ai-tool", "method"),
)
#let used(categories) = data.skills.filter(s => (
  s.months > 0 and categories.contains(s.category)
))
#grid(
  columns: (1fr,) * groups.len(),
  gutter: 6pt,
  ..groups.map(categories => table(
    columns: (1fr, auto, auto),
    stroke: none,
    inset: (x: 2pt, y: 1.6pt),
    table.header(
      head(
        colspan: 3,
        categories.map(c => data.category_labels.at(c)).join("・"),
      ),
    ),
    ..used(categories)
      .map(s => (
        s.label,
        years-bar(s.months),
        align(right, str(s.years) + "年"),
      ))
      .flatten(),
  )),
)

#pagebreak()

#let short(categories, p) = (
  categories.map(c => p.skills.at(c, default: ())).flatten().join(", ")
)
#table(
  columns: (5mm, 19mm, 1fr, 16mm, 20mm, 9mm, 22mm, 26mm, 24mm, 24mm)
    + (5.5mm,) * data.phase_labels.len(),
  table.header(
    repeat: true,
    head[No],
    head[期間],
    head[プロジェクト・業務内容],
    head[業種],
    head[役割],
    head[体制],
    head[言語],
    head[FW・ライブラリ],
    head[DB・クラウド・基盤],
    head[ツール・手法],
    ..data.phase_labels.map(p => head(text(size: 6.5pt, p))),
  ),
  ..data
    .projects
    .map(p => (
      align(center, str(p.number)),
      [#p.start〜\ #p.end\ (#p.duration)],
      [
        #text(weight: "bold", p.title)　#text(fill: ink.lighten(30%), size: 6.5pt, p.client) \
        #p.description
      ],
      p.industry,
      [#p.role \ #p.contract_type],
      align(center, if p.team_size == none { [] } else { [#p.team_size 名] }),
      short(("language",), p),
      short(("framework", "library"), p),
      short(("database", "cloud", "platform"), p),
      short(("tool", "ai-tool", "method"), p),
      ..p.phases.map(on => align(center + horizon, if on [●])),
    ))
    .flatten(),
)
