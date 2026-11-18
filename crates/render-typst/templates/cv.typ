// The CV, layout variant C "Projects first" (issue #7, branch prototype/cv). The content, headings and labels come
// from the documents crate as JSON in sys.inputs.data, already in the CV's language; this file holds the layout.

#let cv = json(sys.inputs.data)
#let lang = cv.language

#let t = cv.labels

#let ink = rgb("#18181b")
#let band = rgb("#232a3b")
#let accent = rgb("#c2410c")
#let chip-fill = rgb("#f1f1f4")
#let muted = luma(105)

#set document(title: t.title + " – " + cv.name, author: cv.name)
#set page(
  paper: "a4",
  margin: (x: 16mm, top: 14mm, bottom: 16mm),
  footer: context {
    set text(7pt, fill: muted)
    cv.name + " · " + cv.headline
    h(1fr)
    str(here().page()) + " / " + str(counter(page).final().first())
  },
)
#set text(font: ("Inter", "BIZ UDPGothic"), size: 8.8pt, fill: ink, lang: lang)
#set par(leading: 0.58em, spacing: 0.75em, justify: true)
#set list(indent: 0pt, body-indent: 0.45em, marker: text(fill: accent)[•])

#let section-heading(title) = {
  v(6mm, weak: true)
  block(spacing: 2.5mm, sticky: true, text(14pt, weight: "bold", title))
}

#let chip(body) = box(
  fill: chip-fill,
  inset: (x: 1.6mm, y: 0.9mm),
  radius: 1.5mm,
  text(7.6pt, body),
)

#let strip-https(url) = url.replace("https://", "")


// Header band, full width
#block(
  width: 100%,
  inset: (top: 9mm, bottom: 7mm),
  outset: (top: 14mm, x: 16mm),
  fill: band,
  spacing: 0mm,
  {
    set text(fill: white)
    let contact = cv.contact
    grid(
      columns: (1fr, auto, auto),
      column-gutter: 6mm,
      align: bottom,
      {
        text(28pt, weight: "bold", tracking: -0.02em, cv.name)
        v(-3mm)
        text(12pt, fill: rgb("#fdba74"), cv.headline)
      },
      {
        set text(8pt)
        set align(right)
        (
          contact.location,
          contact.phone,
          if contact.email != none {
            link("mailto:" + contact.email, contact.email)
          },
          if contact.portfolio_url != none {
            link(contact.portfolio_url, strip-https(contact.portfolio_url))
          },
          if contact.github_url != none {
            link(contact.github_url, strip-https(contact.github_url))
          },
        )
          .filter(x => x != none)
          .join(linebreak())
      },
      if "photo" in sys.inputs {
        box(radius: 2mm, clip: true, image(
          sys.inputs.photo,
          width: 24mm,
          height: 30mm,
          fit: "cover",
        ))
      },
    )
  },
)

// At a glance
#block(
  width: 100%,
  inset: (y: 3mm),
  outset: (x: 16mm),
  fill: chip-fill,
  spacing: 0mm,
  {
    set text(8pt)
    let details = cv.personal_details
    let items = (
      if cv.availability != none {
        text(weight: "semibold", fill: accent, cv.availability)
      },
      cv
        .languages
        .map(l => l.name + if l.level != none { " (" + l.level + ")" })
        .join(" · "),
      if details != none {
        (
          if details.birth_date != none [#t.born #details.birth_date],
          details.nationality,
        )
          .filter(x => x != none)
          .join(" · ")
      },
    ).filter(x => x != none)
    items.join(h(1fr))
  },
)

#section-heading(t.profile)
#grid(
  columns: (1.7fr, 1fr),
  column-gutter: 7mm,
  cv.summary.map(par).join(),
  if cv.strengths.len() > 0 {
    block(inset: (left: 4mm), stroke: (left: 1.5pt + accent), {
      text(weight: "bold", t.strengths)
      set par(justify: false)
      list(..cv.strengths)
    })
  },
)

#let project-card(project) = block(breakable: false, spacing: 4.5mm, {
  grid(
    columns: (1fr, auto),
    column-gutter: 4mm,
    text(10.5pt, weight: "bold", project.title),
    text(8pt, weight: "semibold", fill: accent, project.period),
  )
  v(-1.5mm)
  {
    set text(7.8pt, fill: muted)
    (
      project.role + if project.focus != none { " (" + project.focus + ")" },
      t.client + " " + project.client,
      if project.end_client != none { t.end_client + " " + project.end_client },
      project.industry,
      project.team,
    )
      .filter(x => x != none)
      .join("  ·  ")
  }
  v(-0.5mm)
  if project.description != none { par(project.description) }
  if project.achievements.len() > 0 { list(..project.achievements) }
  project.skills.map(chip).join(h(1mm))
})

#if cv.projects.len() > 0 {
  section-heading(t.selected_projects)
  for project in cv.projects { project-card(project) }
}

#if cv.career.len() > 0 {
  section-heading(t.career)
  table(
    columns: (auto, 1fr, auto, auto),
    stroke: (x, y) => (bottom: 0.4pt + luma(215)),
    inset: (x: 0mm, y: 1.8mm),
    column-gutter: 5mm,
    ..cv
      .career
      .map(entry => (
        text(fill: muted, entry.period),
        text(weight: "semibold", entry.role),
        entry.organization,
        text(
          fill: muted,
          entry.engagement_type
            + if entry.projects != none { " · " + entry.projects },
        ),
      ))
      .flatten(),
  )
}

#if cv.skills.len() > 0 {
  section-heading(t.skills)
  table(
    columns: (32mm, 1fr),
    stroke: none,
    inset: (x: 0mm, y: 1.2mm),
    column-gutter: 3mm,
    ..cv
      .skills
      .map(group => (
        text(weight: "semibold", group.category),
        group
          .skills
          .map(skill => chip[#skill.name #text(fill: muted, skill.experience)])
          .join(h(1mm)),
      ))
      .flatten(),
  )
}

#let qualification(q) = block(
  spacing: 2mm,
  [*#q.title* \ #text(fill: muted)[#if q.issuer != none [#q.issuer · ]#q.date]],
)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 8mm,
  if cv.education.len() > 0 {
    section-heading(t.education)
    for q in cv.education { qualification(q) }
  },
  if cv.certificates.len() > 0 {
    section-heading(t.certificates)
    for q in cv.certificates { qualification(q) }
  },
)
