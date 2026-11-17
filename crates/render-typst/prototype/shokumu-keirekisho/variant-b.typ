// PROTOTYPE, throwaway. Variant B「サマリー先行・カード型」: page 1 is an overview a reader takes in at a glance
// (要約, 得意分野, スキル with experience bars, 資格), then one card per Project grouped under its Engagement. Sans serif.

#import "common.typ": *

#let accent = rgb("#1f3a5f")
#let muted = luma(110)

#set document(title: "職務経歴書")
#set page(
  paper: "a4",
  margin: (x: 16mm, top: 18mm, bottom: 18mm),
  header: context if counter(page).get().first() > 1 {
    set text(size: 7.5pt, fill: muted)
    [職務経歴書　#profile.name #h(1fr) #format-date(today)現在]
  },
  footer: context align(right, text(
    size: 7.5pt,
    fill: muted,
    counter(page).display("1 / 1", both: true),
  )),
)
#set text(
  font: sys.inputs.at("font", default: "BIZ UDPGothic"),
  size: 9pt,
  lang: "ja",
  region: "JP",
)
#set par(justify: true, leading: 0.7em, spacing: 0.9em)
#set list(marker: text(fill: accent, "■"), indent: 0pt, body-indent: 0.4em)

#let section(title) = block(
  sticky: true,
  above: 1.4em,
  below: 0.6em,
  width: 100%,
  stroke: (bottom: 1pt + accent),
  inset: (
    bottom: 3pt,
  ),
)[#text(size: 11pt, weight: "bold", fill: accent, title)]

#let subheading(title) = text(weight: "bold", fill: accent, title)

#grid(
  columns: (1fr, auto),
  align: (left + bottom, right + bottom),
  [
    #text(size: 20pt, weight: "bold", fill: accent)[職務経歴書] \
    #text(size: 12pt, weight: "bold", profile.name)#h(0.8em)#text(
      fill: muted,
      ja(profile.headline),
    )
  ],
  text(
    size: 8.5pt,
    fill: muted,
  )[#format-date(today)現在],
)

#section[職務要約]
#ja-paragraphs(profile.summary)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 8mm,
  [
    #section[得意分野]
    #list(..ja-bullets(profile.strengths))
  ],
  [
    #section[資格・語学]
    #for qualification in qualifications [
      #text(fill: muted, format-month(qualification.date)) #h(
        0.5em,
      ) #qualification-label(qualification) \
    ]
    #for language in human-languages [
      #text(fill: muted)[語学] #h(0.5em)
      #if language.ja != none { language.ja } else { language.name }（#ja(language.level)） \
    ]
  ],
)

#section[テクニカルスキル]
#let longest = calc.max(..shown-skills.map(stat => stat.months))
#let bar(months) = box(
  width: 11mm,
  height: 5pt,
  fill: luma(232),
  radius: 1pt,
  box(
    width: 11mm * months / longest,
    height: 5pt,
    fill: accent,
    radius: 1pt,
  ),
)
#let skill-row(stat) = (
  skill-label(stat.name),
  bar(stat.months),
  text(size: 7.5pt, format-duration(stat.months)),
)
#text(
  size: 8pt,
  fill: muted,
)[直近#skill-window 年以内に案件で使用したスキル。バーは案件での使用期間の合計。]
#let category-block(category) = block(breakable: false)[
  #subheading(category.ja)
  #table(
    columns: (1fr, auto, 16mm),
    stroke: none,
    inset: (x: 0pt, y: 1.5pt),
    column-gutter: 4pt,
    align: (left + horizon, horizon, right + horizon),
    ..skills-in(category).map(skill-row).flatten(),
  )
]
#text(size: 8pt, grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 6mm,
  row-gutter: 0.7em,
  ..skill-categories.map(category-block),
))

#section[職務経歴]
#block(sticky: true, text(
  size: 8pt,
  fill: muted,
)[直近#detail-years 年以内の案件は詳細を、それ以前の案件は概要を記載。])

#let phase-mark(project, slug, label) = if project.phases.any(phase => (
  phase.slug == slug
)) {
  text(fill: accent, weight: "bold")[●#label]
} else { text(fill: luma(180))[○#label] }

#let phase-marks(project) = {
  all-phases
    .map(((slug, label)) => phase-mark(project, slug, label))
    .join(h(0.8em))
}

#let facts(project) = {
  let rows = (
    (
      [役割],
      [#ja(project.role)#if project.focus != none [（#ja(project.focus)）]],
    ),
  )
  if project.team_size != none { rows.push(([体制], [#project.team_size 名])) }
  if project.end_client != none {
    rows.push(([エンドクライアント], project.end_client))
  }
  rows.push(([担当工程], phase-marks(project)))
  rows.push(([環境], project.skills.map(skill-label).join(" / ")))
  grid(
    columns: (29mm, 1fr),
    row-gutter: 5pt,
    ..rows.map(((key, value)) => (text(fill: muted, key), value)).flatten(),
  )
}

#let card(project) = block(
  width: 100%,
  fill: luma(247),
  stroke: (left: 2pt + accent),
  inset: (x: 10pt, y: 8pt),
  below: 0.8em,
)[
  #block(sticky: true, grid(
    columns: (auto, 1fr, auto),
    column-gutter: 8pt,
    text(
      weight: "bold",
      fill: accent,
    )[#format-month(project.start)～#format-month(project.end)],
    text(weight: "bold", ja(project.title)),
    box(inset: (x: 5pt, y: 2pt), radius: 6pt, fill: luma(225), text(
      size: 8pt,
      ja(project.industry),
    )),
  ))
  #v(0.2em)
  #text(size: 8.5pt, facts(project))
  #v(0.2em)
  #ja-paragraphs(project.description)
  #let achievements = ja-bullets(project.achievements)
  #if achievements.len() > 0 [
    #block(above: 0.7em, below: 0.4em, subheading[実績])
    #list(..achievements)
  ]
]

#let compact-period(project) = if project.start == project.end {
  format-month(project.start)
} else [
  #format-month(project.start)～ \ #format-month(project.end)
]

#let compact-row(project) = (
  text(size: 8pt, fill: muted, compact-period(project)),
  [
    #text(weight: "bold", ja(project.title))
    #text(size: 7.5pt, fill: muted)[（#ja(project.industry)）] \
    #text(size: 7.5pt, fill: muted, project.skills.map(skill-label).join(" / "))
  ],
  text(size: 8pt, ja(project.role)),
)

#let compact-table(projects) = table(
  columns: (24mm, 1fr, 34mm),
  stroke: (x, y) => (bottom: 0.4pt + luma(215)),
  inset: (x: 3pt, y: 3.5pt),
  ..projects.map(compact-row).flatten(),
)

#let engagement-heading(engagement, detailed) = {
  let organization = organization-of(engagement)
  block(sticky: true, above: 1.1em, below: 0.5em)[
    #text(size: 10.5pt, weight: "bold", organization.name)
    #h(0.6em) #text(
      fill: muted,
    )[#format-month(engagement.start)～#format-month(engagement.end)]
    #h(1fr) #engagement.type.ja#if (
      engagement.agencies.len() > 0
    ) [（#engagement.agencies.join("、")経由）]
    #if engagement.role != none [ \ #text(
      size: 8.5pt,
    )[役職：#ja(engagement.role)]]
    #if detailed and organization.description.en != none [ \ #text(
      size: 8.5pt,
      fill: muted,
    )[事業内容：#ja-paragraphs(organization.description)]]
  ]
}

#for engagement in engagements {
  let projects = projects-of(engagement)
  let detailed = projects.filter(is-detailed)
  let brief = projects.filter(project => not is-detailed(project))
  engagement-heading(engagement, detailed.len() > 0)
  for project in detailed { card(project) }
  if brief.len() > 0 { compact-table(brief) }
}

#section[自己PR]
#ja-paragraphs(profile.self_pr)

#if profile.portfolio_url != none or profile.github_url != none {
  v(0.6em)
  text(size: 8.5pt)[
    #if (
      profile.portfolio_url != none
    ) [ポートフォリオ：#link(profile.portfolio_url)#h(1.5em)]
    #if profile.github_url != none [GitHub：#link(profile.github_url)]
  ]
}

#align(right)[以上]
