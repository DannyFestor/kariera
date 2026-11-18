// The Skill Sheet, layout C: the profile beside the Skill table, then one block per Project (issue #9, branch
// prototype/skill-sheet). The content comes from the documents crate as JSON in sys.inputs.data, already in
// Japanese; this file holds the headings and the layout.

#let doc = json(sys.inputs.data)

#let ink = rgb("#3b2f5c")
#let accent = rgb("#d9d2ea")
#let pale = rgb("#f4f1f9")
#let rule = 0.5pt + rgb("#8a7fa8")

#set document(title: "スキルシート", author: doc.name)
#set page(
  paper: "a4",
  margin: (x: 12mm, top: 12mm, bottom: 14mm),
  footer: context {
    set text(size: 7pt, fill: ink)
    align(center, counter(page).display("1 / 1", both: true))
  },
)
#set text(font: "BIZ UDPGothic", size: 8pt, lang: "ja", region: "JP")
#set par(leading: 0.5em, spacing: 0.75em)
#set table(stroke: rule, inset: (x: 4pt, y: 3pt))

#let key(body) = table.cell(fill: pale, text(weight: "bold", fill: ink, body))
#let lines(items) = items.join(linebreak())
#let dated(entries) = lines(entries.map(q => [#q.date #q.title]))

#block(width: 100%, fill: ink, inset: (x: 6pt, y: 4pt), text(
  fill: white,
  weight: "bold",
  size: 9.5pt,
)[スキルシート　　#doc.name　　#h(1fr) #text(size: 8pt, weight: "regular")[更新日 #doc.updated]])
#v(4pt)

#let availability = (
  if doc.available_from != none [#doc.available_from から],
  if doc.days_per_week != none [週#doc.days_per_week 日],
).filter(x => x != none)

#let profile-table = table(
  columns: (22mm, 1fr),
  key[得意分野], doc.headline,
  key[所属], doc.business,
  key[稼働], availability.join("　"),
  key[URL], lines((doc.portfolio_url, doc.github_url).filter(x => x != none)),
  key[資格], dated(doc.certificates),
  key[学歴], dated(doc.education),
  key[得意技術], lines(doc.strengths.map(s => [・#s])),
)

#let skill-table = table(
  columns: (auto, 1fr, auto, auto),
  table.header(key[分類], key[スキル], key[経験], key[最終使用]),
  ..doc
    .skills
    .map(group => (
      table.cell(rowspan: group.skills.len(), text(size: 7pt, group.category)),
      ..group
        .skills
        .map(skill => (
          skill.name,
          align(right, skill.experience),
          text(size: 7pt, skill.last_use),
        ))
        .flatten(),
    ))
    .flatten(),
)

#grid(
  columns: (1fr, 1fr),
  gutter: 8pt,
  profile-table, skill-table,
)
#v(4pt)
#table(
  columns: (22mm, 1fr),
  key[自己PR], doc.self_pr.join(parbreak()),
)

#let phase-strip(phases) = grid(
  columns: (1fr,) * phases.len(),
  gutter: 2pt,
  ..phases.map(phase => box(
    width: 100%,
    inset: 2.5pt,
    fill: if phase.done { ink } else { pale },
    align(center, text(
      size: 7pt,
      fill: if phase.done { white } else { luma(170) },
      phase.label,
    )),
  )),
)

#let technology(project) = lines(project.technology.map(group => [#text(
    fill: ink,
    weight: "bold",
    group.category,
  )：#group.skills.join("、")]))

#for project in doc.projects {
  v(6pt)
  block(breakable: false, width: 100%, stroke: rule, {
    block(width: 100%, fill: accent, inset: (x: 6pt, y: 4pt), below: 0pt, grid(
      columns: (auto, 1fr, auto),
      gutter: 6pt,
      text(weight: "bold", fill: ink)[No. #project.number],
      text(weight: "bold", size: 9.5pt, project.title),
      [#project.period（#project.duration）],
    ))
    block(inset: 6pt, width: 100%, {
      table(
        columns: (auto, 1fr, auto, 1fr, auto, 1fr),
        stroke: none,
        inset: (x: 2pt, y: 1.5pt),
        key[会社],
        [#project.client#if (
            project.end_client != none
          ) [ ／ #project.end_client]],
        key[業種],
        project.industry,
        key[契約],
        project.contract_type,

        key[役割],
        [#project.role#if project.focus != none [（#project.focus）]],
        key[体制],
        if project.team_size == none [—] else [#project.team_size 名],
        [],
        [],
      )
      v(3pt)
      phase-strip(project.phases)
      v(3pt)
      technology(project)
      v(2pt)
      [#text(
          weight: "bold",
          fill: ink,
        )[【業務内容】] \ #project.description.join(parbreak())]
      if project.achievements.len() > 0 {
        [\ #text(weight: "bold", fill: ink)[【実績】] \ #lines(
            project.achievements.map(a => [・#a]),
          )]
      }
    })
  })
}
