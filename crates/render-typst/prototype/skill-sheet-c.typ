// PROTOTYPE (throwaway), Skill Sheet variant C: the project blocks (geechs style). A4 portrait; a two-column profile
// with the years per Skill as a table by category, then each Project as its own labelled block: a title band, a
// facts row, the phases as a strip, the technology by category, then the description and the achievements.
#let data = json(sys.inputs.data)
#let profile = data.profile
#let ink = rgb("#3b2f5c")
#let accent = rgb("#d9d2ea")
#let pale = rgb("#f4f1f9")
#let rule = 0.5pt + rgb("#8a7fa8")

#set page(
  paper: "a4",
  margin: (x: 12mm, top: 12mm, bottom: 14mm),
  footer: context {
    set text(size: 7pt, fill: ink)
    align(center, counter(page).display("1 / 1", both: true))
  },
)
#set text(font: "IBM Plex Sans JP", size: 8pt, lang: "ja")
#set par(leading: 0.5em, spacing: 0.75em)
#set table(stroke: rule, inset: (x: 4pt, y: 3pt))

#let key(body) = table.cell(fill: pale, text(weight: "bold", fill: ink, body))
#let heading-band(body) = block(
  width: 100%,
  fill: ink,
  inset: (x: 6pt, y: 4pt),
  text(
    fill: white,
    weight: "bold",
    size: 9.5pt,
    body,
  ),
)

#heading-band[スキルシート　　#profile.name　　#h(1fr) #text(size: 8pt, weight: "regular")[更新日 #data.build_month]]
#v(4pt)

#grid(
  columns: (1fr, 1fr),
  gutter: 8pt,
  table(
    columns: (22mm, 1fr),
    key[得意分野], profile.headline,
    key[所属], profile.business,
    key[稼働],
    [#profile.available_from #(
        profile.days_per_week.filter(d => d != none).map(str).join("〜")
      )],

    key[URL], [#profile.portfolio_url \ #profile.github_url],
    key[資格],
    profile
      .qualifications
      .filter(q => q.category != "degree")
      .map(q => [#q.date #q.label])
      .join(linebreak()),

    key[学歴],
    profile
      .qualifications
      .filter(q => q.category == "degree")
      .map(q => [#q.date #q.label])
      .join(linebreak()),

    key[得意技術], profile.strengths.map(s => [・#s]).join(linebreak()),
  ),
  {
    let used = data.skills.filter(s => s.months > 0)
    table(
      columns: (auto, 1fr, auto, auto),
      table.header(key[分類], key[スキル], key[経験], key[最終]),
      ..data
        .category_labels
        .pairs()
        .map(((category, name)) => {
          let rows = used.filter(s => s.category == category)
          if rows.len() == 0 { return () }
          (
            table.cell(rowspan: rows.len(), text(size: 7pt, name)),
            ..rows
              .map(s => (
                s.label,
                align(right, s.duration),
                text(size: 7pt, s.last_used),
              ))
              .flatten(),
          )
        })
        .flatten(),
    )
  },
)
#v(4pt)
#table(
  columns: (22mm, 1fr),
  key[自己PR], profile.summary + parbreak() + profile.self_pr,
)

#let phase-strip(phases) = grid(
  columns: (1fr,) * phases.len(),
  gutter: 2pt,
  ..data
    .phase_labels
    .zip(phases)
    .map(((name, on)) => box(
      width: 100%,
      inset: 2.5pt,
      fill: if on { ink } else { pale },
      align(center, text(
        size: 7pt,
        fill: if on { white } else { luma(170) },
        name,
      )),
    )),
)
#let technology(p) = {
  let present = data
    .category_labels
    .pairs()
    .filter(pair => pair.first() in p.skills)
  let line(
    pair,
  ) = [#text(fill: ink, weight: "bold", pair.last())：#p.skills.at(pair.first()).join("、")]
  present.map(line).join(linebreak())
}

#for p in data.projects {
  v(6pt)
  block(breakable: true, width: 100%, stroke: rule, {
    block(width: 100%, fill: accent, inset: (x: 6pt, y: 4pt), below: 0pt, grid(
      columns: (auto, 1fr, auto),
      gutter: 6pt,
      text(weight: "bold", fill: ink)[No. #p.number],
      text(weight: "bold", size: 9.5pt, p.title),
      [#p.start 〜 #p.end（#p.duration）],
    ))
    block(inset: 6pt, width: 100%, {
      table(
        columns: (auto, 1fr, auto, 1fr, auto, 1fr),
        stroke: none,
        inset: (x: 2pt, y: 1.5pt),
        key[会社],
        p.client + if p.end_client != "" [ ／ #p.end_client],
        key[業種],
        p.industry,
        key[契約],
        p.contract_type,

        key[役割],
        [#p.role#if p.focus != "" [（#p.focus）]],
        key[体制],
        if p.team_size == none [—] else [#p.team_size 名],
        [],
        [],
      )
      v(3pt)
      phase-strip(p.phases)
      v(3pt)
      technology(p)
      v(2pt)
      [#text(weight: "bold", fill: ink)[【業務内容】] \ #p.description]
      if p.achievements.len() > 0 {
        [\ #text(weight: "bold", fill: ink)[【実績】] \ #(
            p.achievements.map(a => [・#a]).join(linebreak())
          )]
      }
    })
  })
}
