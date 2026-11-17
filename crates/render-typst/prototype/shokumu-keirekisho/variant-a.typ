// PROTOTYPE, throwaway. Variant A「定番の表組み」: the form Japanese agencies know. One bordered table per
// Engagement (期間 | 業務内容 | 開発環境 | 役割・体制), ■ section marks, Mincho body, Western calendar.

#import "common.typ": *

#set document(title: "職務経歴書")
#set page(
  paper: "a4",
  margin: (x: 17mm, top: 16mm, bottom: 18mm),
  footer: context align(center, text(
    size: 8pt,
    counter(page).display("1 / 1", both: true),
  )),
)
#set text(font: "BIZ UDPMincho", size: 9.5pt, lang: "ja", region: "JP")
#set par(justify: true, leading: 0.6em, spacing: 0.9em)
#set list(marker: "・", indent: 0pt, body-indent: 0pt)

#let gothic(body) = text(font: "BIZ UDPGothic", body)
#let section(title) = block(above: 1.6em, below: 0.7em, gothic(text(
  weight: "bold",
  size: 10.5pt,
)[■#title]))

#align(center, gothic(text(
  size: 16pt,
  weight: "bold",
  tracking: 0.5em,
)[職務経歴書]))
#align(right)[
  #format-date(today)現在 \
  氏名：#profile.name
]

#section[職務要約]
#ja-paragraphs(profile.summary)

#section[得意分野・活かせる経験]
#list(..ja-bullets(profile.strengths))

#section[職務経歴]

#let rule = 0.6pt
#let dotted = (thickness: 0.4pt, dash: "dotted")

#let period-cell(project) = align(center)[
  #format-month(project.start) \
  ｜ \
  #format-month(project.end)
]

#let work-cell(project) = {
  [■#ja(project.title) \ ■業界：#ja(project.industry)]
  if project.end_client != none [ \ ■エンドクライアント：#project.end_client]
  v(0.4em)
  [【業務内容】 \ ]
  ja-paragraphs(project.description)
  let achievements = ja-bullets(project.achievements)
  if achievements.len() > 0 {
    v(0.2em)
    [【実績】]
    list(..achievements)
  }
  if project.phases.len() > 0 {
    v(0.2em)
    [【担当フェーズ】 \ #phase-labels(project)]
  }
}

#let role-cell(project) = {
  [役割：\ #ja(project.role)]
  if project.focus != none [（#ja(project.focus)）]
  if project.team_size != none [ \ 体制：#project.team_size 名]
}

#let engagement-table(engagement) = {
  set par(justify: false)
  table(
    columns: (21mm, 1fr, 31mm, 25mm),
    inset: (x: 4pt, y: 5pt),
    stroke: (x, y) => (
      left: if x == 0 { rule } else { dotted },
      right: if x == 3 { rule } else { none },
      top: rule,
      bottom: rule,
    ),
    fill: (x, y) => if y == 0 { luma(235) },
    align: left,
    table.header(
      ..("期間", "業務内容", "開発環境", "役割・体制").map(title => align(
        center,
        gothic(title),
      )),
    ),
    ..projects-of(engagement)
      .map(project => (
        text(size: 8.5pt, period-cell(project)),
        work-cell(project),
        text(size: 8.5pt, project.skills.map(skill-label).join(linebreak())),
        text(size: 8.5pt, role-cell(project)),
      ))
      .flatten(),
  )
}

#for engagement in engagements {
  let organization = organization-of(engagement)
  block(breakable: false, below: 0.4em)[
    ◇#organization.name（#format-month(engagement.start)～#format-month(engagement.end)）
    #engagement.type.ja
    #if (
      engagement.agencies.len() > 0
    ) [（#engagement.agencies.join("、")経由）] \
    #if organization.description.en != none {
      text(size: 8.5pt)[事業内容：#ja-paragraphs(organization.description)]
    }
  ]
  if projects-of(engagement).len() > 0 { engagement-table(engagement) } else {
    block(inset: (left: 1em), ja-paragraphs(engagement.summary))
  }
  v(1em)
}

#section[テクニカルスキル]
#text(
  size: 8.5pt,
)[直近#skill-window 年以内に案件で使用したもの。（ ）内は案件での使用期間の合計。]
#table(
  columns: (36mm, 1fr),
  inset: 5pt,
  stroke: rule,
  fill: (x, y) => if x == 0 { luma(245) },
  ..skill-categories
    .map(category => (
      gothic(category.ja),
      skills-in(category)
        .map(stat => (
          skill-label(stat.name) + "（" + format-duration(stat.months) + "）"
        ))
        .join("、"),
    ))
    .flatten(),
)

#section[資格]
#for qualification in qualifications [
  ・#box(width: 28mm, format-month(qualification.date))#qualification-label(qualification) \
]

#section[語学]
#for language in human-languages [
  ・#if language.ja != none { language.ja } else { language.name }：#ja(language.level) \
]

#if profile.portfolio_url != none or profile.github_url != none {
  section[ポートフォリオ]
  if profile.portfolio_url != none [・#link(profile.portfolio_url) \ ]
  if profile.github_url != none [・GitHub：#link(profile.github_url)]
}

#section[自己PR]
#ja-paragraphs(profile.self_pr)

#align(right)[以上]
