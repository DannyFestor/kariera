// The 職務経歴書, layout variant B「サマリー先行・カード型」 (issue #7, branch prototype/shokumu-keirekisho). The
// content, headings and labels come from the documents crate as JSON in sys.inputs.data, already in Japanese; this
// file holds the layout.

#let doc = json(sys.inputs.data)
#let t = doc.labels

#let accent = rgb("#1f3a5f")
#let muted = luma(110)

#set document(title: t.title, author: doc.name)
#set page(
  paper: "a4",
  margin: (x: 16mm, top: 18mm, bottom: 18mm),
  header: context if counter(page).get().first() > 1 {
    set text(size: 7.5pt, fill: muted)
    [#t.title　#doc.name #h(1fr) #doc.as_of]
  },
  footer: context align(right, text(
    size: 7.5pt,
    fill: muted,
    counter(page).display("1 / 1", both: true),
  )),
)
#set text(font: "BIZ UDPGothic", size: 9pt, lang: "ja", region: "JP")
#set par(justify: true, leading: 0.7em, spacing: 0.9em)
#set list(marker: text(fill: accent, "■"), indent: 0pt, body-indent: 0.4em)

#let section(title) = block(
  sticky: true,
  above: 1.4em,
  below: 0.6em,
  width: 100%,
  stroke: (bottom: 1pt + accent),
  inset: (bottom: 3pt),
)[#text(size: 11pt, weight: "bold", fill: accent, title)]

#let subheading(title) = text(weight: "bold", fill: accent, title)

#let paragraphs(list) = list.join(parbreak())

#let note(body) = block(sticky: true, text(size: 8pt, fill: muted, body))

#grid(
  columns: (1fr, auto),
  align: (left + bottom, right + bottom),
  [
    #text(size: 20pt, weight: "bold", fill: accent, t.title) \
    #text(size: 12pt, weight: "bold", doc.name)#h(0.8em)#text(
      fill: muted,
      doc.headline,
    )
  ],
  text(size: 8.5pt, fill: muted, doc.as_of),
)

#section(t.summary)
#paragraphs(doc.summary)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 8mm,
  [
    #section(t.strengths)
    #list(..doc.strengths)
  ],
  [
    #section(t.qualifications)
    #for qualification in doc.qualifications [
      #text(fill: muted, qualification.date) #h(0.5em) #qualification.title \
    ]
    #for language in doc.languages [
      #text(fill: muted, t.language) #h(0.5em)
      #language.name#if language.level != none [（#language.level）] \
    ]
  ],
)

#if doc.skills.len() > 0 {
  section(t.skills)
  let longest = calc.max(
    ..doc
      .skills
      .map(group => group.skills.map(skill => skill.months))
      .flatten(),
  )
  let bar(months) = box(
    width: 11mm,
    height: 5pt,
    fill: luma(232),
    radius: 1pt,
    box(
      width: 11mm * months / calc.max(longest, 1),
      height: 5pt,
      fill: accent,
      radius: 1pt,
    ),
  )
  let skill-row(skill) = (
    skill.name,
    bar(skill.months),
    text(size: 7.5pt, skill.experience),
  )
  note(doc.skills_note)
  let category-block(group) = block(breakable: false)[
    #subheading(group.category)
    #table(
      columns: (1fr, auto, 16mm),
      stroke: none,
      inset: (x: 0pt, y: 1.5pt),
      column-gutter: 4pt,
      align: (left + horizon, horizon, right + horizon),
      ..group.skills.map(skill-row).flatten(),
    )
  ]
  text(size: 8pt, grid(
    columns: (1fr, 1fr, 1fr),
    column-gutter: 6mm,
    row-gutter: 0.7em,
    ..doc.skills.map(category-block),
  ))
}

#section(t.history)
#note(doc.history_note)

#let phase-mark(phase) = if phase.done {
  text(fill: accent, weight: "bold")[●#phase.label]
} else { text(fill: luma(180))[○#phase.label] }

#let facts(project) = {
  let rows = (
    (
      t.role,
      [#project.role#if project.focus != none [（#project.focus）]],
    ),
  )
  if project.team != none { rows.push((t.team, project.team)) }
  if project.end_client != none {
    rows.push((t.end_client, project.end_client))
  }
  rows.push((t.phases, project.phases.map(phase-mark).join(h(0.8em))))
  rows.push((t.environment, project.skills.join(" / ")))
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
    text(weight: "bold", fill: accent, project.period),
    text(weight: "bold", project.title),
    box(inset: (x: 5pt, y: 2pt), radius: 6pt, fill: luma(225), text(
      size: 8pt,
      project.industry,
    )),
  ))
  #v(0.2em)
  #text(size: 8.5pt, facts(project))
  #v(0.2em)
  #paragraphs(project.description)
  #if project.achievements.len() > 0 [
    #block(above: 0.7em, below: 0.4em, subheading(t.achievements))
    #list(..project.achievements)
  ]
]

#let line-row(project) = (
  text(size: 8pt, fill: muted, project.period),
  [
    #text(weight: "bold", project.title)
    #text(size: 7.5pt, fill: muted)[（#project.industry）] \
    #text(size: 7.5pt, fill: muted, project.skills.join(" / "))
  ],
  text(size: 8pt, project.role),
)

#let older-table(projects) = table(
  columns: (24mm, 1fr, 34mm),
  stroke: (x, y) => (bottom: 0.4pt + luma(215)),
  inset: (x: 3pt, y: 3.5pt),
  ..projects.map(line-row).flatten(),
)

#let engagement-heading(engagement) = block(
  sticky: true,
  above: 1.1em,
  below: 0.5em,
)[
  #text(size: 10.5pt, weight: "bold", engagement.organization)
  #h(0.6em) #text(fill: muted, engagement.period)
  #h(1fr) #engagement.engagement_type
  #if engagement.role != none [ \ #text(
    size: 8.5pt,
  )[#t.position：#engagement.role]]
  #if engagement.business.len() > 0 [ \ #text(
    size: 8.5pt,
    fill: muted,
  )[#t.business：#engagement.business.join(" ")]]
]

#for engagement in doc.engagements {
  engagement-heading(engagement)
  for project in engagement.cards { card(project) }
  if engagement.older.len() > 0 { older-table(engagement.older) }
}

#if doc.self_pr.len() > 0 {
  section(t.self_pr)
  paragraphs(doc.self_pr)
}

#if doc.portfolio_url != none or doc.github_url != none {
  v(0.6em)
  text(size: 8.5pt)[
    #if (
      doc.portfolio_url != none
    ) [#t.portfolio：#link(doc.portfolio_url)#h(1.5em)]
    #if doc.github_url != none [#t.github：#link(doc.github_url)]
  ]
}

#align(right, t.closing)
