// PROTOTYPE, throwaway. Variant C「一覧表・コンパクト」: one continuous table for the whole career, one row per
// Project, Engagements as separator rows, the six phases as ● columns like a Skill Sheet. Aims at two to four pages.

#import "common.typ": *

#let line-color = luma(150)
#let band = luma(228)

#set document(title: "職務経歴書")
#set page(
  paper: "a4",
  margin: (x: 12mm, top: 14mm, bottom: 16mm),
  footer: context align(center, text(
    size: 7.5pt,
    counter(page).display("1 / 1", both: true),
  )),
)
#set text(font: "BIZ UDPGothic", size: 8.5pt, lang: "ja", region: "JP")
#set par(justify: false, leading: 0.55em, spacing: 0.7em)
#set list(marker: "・", indent: 0pt, body-indent: 0pt)

#let section(title) = block(
  above: 1.3em,
  below: 0.5em,
  fill: band,
  width: 100%,
  inset: (x: 5pt, y: 4pt),
  text(
    size: 9.5pt,
    weight: "bold",
    title,
  ),
)

#let info-table(..rows) = table(
  columns: (22mm, 1fr),
  stroke: 0.5pt + line-color,
  inset: 4pt,
  fill: (x, y) => if x == 0 { band },
  ..rows.pos().flatten(),
)

#let availability = {
  let (low, high) = profile.days_per_week
  if profile.available_from != none [#format-date(profile.available_from)～]
  if low != none and high != none [　週#low～#high 日]
}

#grid(
  columns: (1fr, 78mm),
  column-gutter: 6mm,
  [
    #text(size: 18pt, weight: "bold")[職務経歴書] \
    #v(0.3em)
    #text(size: 12pt, weight: "bold", profile.name) \
    #ja(profile.headline)
  ],
  info-table(
    ([作成日], [#format-date(today)]),
    ([稼働可能], availability),
    ..if profile.portfolio_url != none {
      (([ポートフォリオ], link(profile.portfolio_url)),)
    },
    ..if profile.github_url != none { (([GitHub], link(profile.github_url)),) },
  ),
)

#section[職務要約]
#ja-paragraphs(profile.summary)

#section[得意分野]
#list(..ja-bullets(profile.strengths))

#section[職務経歴]

#let phase-header(label) = align(center, stack(dir: ttb, spacing: 1pt, ..label
  .clusters()
  .map(
    text.with(
      size: 7pt,
    ),
  )))

#let project-number = counter("project")

#let period(project) = align(center)[
  #format-month(project.start) \ ～ \ #format-month(project.end) \
  #text(
    size: 7pt,
    fill: luma(90),
  )[（#format-duration(months-of(project).len())）]
]

#let summary(project) = {
  text(weight: "bold", ja(project.title))
  linebreak()
  text(
    size: 7.5pt,
    fill: luma(90),
  )[業界：#ja(project.industry)#if project.end_client != none [　エンドクライアント：#project.end_client]]
  block(above: 0.4em, below: 0.3em, ja-paragraphs(project.description))
  list(..ja-bullets(project.achievements))
}

#let role(project) = {
  ja(project.role)
  if project.focus != none [（#ja(project.focus)）]
  if project.team_size != none [ \ #text(
    fill: luma(90),
  )[体制 #project.team_size 名]]
}

#let project-row(project) = (
  align(center, project-number.step() + context project-number.display()),
  text(size: 7.5pt, period(project)),
  summary(project),
  ..all-phases.map(((slug, label)) => align(
    center + horizon,
    if project.phases.any(phase => phase.slug == slug) {
      "●"
    },
  )),
  text(size: 7.5pt, project.skills.map(skill-label).join(linebreak())),
  text(size: 7.5pt, role(project)),
)

#let engagement-row(engagement) = {
  let organization = organization-of(engagement)
  table.cell(colspan: 11, fill: band)[
    #text(weight: "bold", organization.name)
    #h(0.6em) #format-month(engagement.start)～#format-month(engagement.end)
    #h(0.6em) #engagement.type.ja#if (
      engagement.agencies.len() > 0
    ) [（#engagement.agencies.join("、")経由）]
    #if organization.description.en != none [#h(0.6em) #text(
        size: 7.5pt,
      )[事業内容：#ja-paragraphs(organization.description)]]
  ]
}

#table(
  columns: (5mm, 22mm, 1fr, ..all-phases.map(_ => 4.6mm), 25mm, 22mm),
  stroke: 0.5pt + line-color,
  inset: (x: 3pt, y: 4pt),
  table.header(
    ..([No], [期間], [案件概要]).map(title => align(center + horizon, title)),
    ..all-phases.map(((slug, label)) => phase-header(label)),
    ..([環境], [役割・体制]).map(title => align(center + horizon, title)),
  ),
  ..engagements
    .map(engagement => (
      engagement-row(engagement),
      ..projects-of(engagement).map(project-row).flatten(),
    ))
    .flatten(),
)

#section[テクニカルスキル]
#text(
  size: 7.5pt,
  fill: luma(90),
)[直近#skill-window 年以内に案件で使用したもの。数字は案件での使用年数（月単位で合算）。]
#table(
  columns: (30mm, 1fr),
  stroke: 0.5pt + line-color,
  inset: 4pt,
  fill: (x, y) => if x == 0 { band },
  ..skill-categories
    .map(category => (
      category.ja,
      skills-in(category)
        .map(stat => [#skill-label(stat.name) #text(fill: luma(90), str(
            calc.round(stat.months / 12, digits: 1),
          ))])
        .join("　/　"),
    ))
    .flatten(),
)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 6mm,
  [
    #section[資格]
    #for qualification in qualifications [
      #box(width: 22mm, format-month(qualification.date))#qualification-label(
        qualification,
      ) \
    ]
  ],
  [
    #section[語学]
    #for language in human-languages [
      #box(width: 22mm, if language.ja != none { language.ja } else {
        language.name
      })#ja(language.level) \
    ]
  ],
)

#section[自己PR]
#ja-paragraphs(profile.self_pr)

#align(right)[以上]
