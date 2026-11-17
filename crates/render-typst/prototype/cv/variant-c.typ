// PROTOTYPE, throwaway (branch prototype/cv). Variant C, "Projects first": a dark header band, then the profile, then
// the most recent Projects in detail as the centre of the CV, the career as a compact table, and a Skills matrix.
// Content rule: the 5 most recent Projects across all Engagements with description and up to 3 achievements; every
// Engagement as one table row; Skills used in the last 3 years, grouped by category, as chips with years.

#import "common.typ": *

#let ink = rgb("#18181b")
#let band = rgb("#232a3b")
#let accent = rgb("#c2410c")
#let chip-fill = rgb("#f1f1f4")
#let muted = luma(105)
#let projects-shown = 5
#let achievements-per-project = 3

#set document(title: cv.profile.name + " – CV", author: cv.profile.name)
#set page(
  paper: "a4",
  margin: (x: 16mm, top: 14mm, bottom: 16mm),
  footer: context {
    set text(7pt, fill: muted)
    cv.profile.name + " · " + cv.profile.headline
    h(1fr)
    str(here().page()) + " / " + str(counter(page).final().first())
  },
)
#set text(font: ("Inter", "Noto Sans JP"), size: 8.8pt, fill: ink, lang: lang)
#set par(leading: 0.58em, spacing: 0.75em, justify: true)
#set list(indent: 0pt, body-indent: 0.45em, marker: text(fill: accent)[•])

#let heading-c(title) = {
  v(6mm, weak: true)
  block(spacing: 2.5mm, text(14pt, weight: "bold", title))
}

#let chip(body) = box(
  fill: chip-fill,
  inset: (x: 1.6mm, y: 0.9mm),
  radius: 1.5mm,
  text(7.6pt, body),
)

#let fact(label, value) = if value != none [#text(fill: muted, label) #value]

// Header band, full width
#block(
  width: 100%,
  inset: (top: 9mm, bottom: 7mm),
  outset: (top: 14mm, x: 16mm),
  fill: band,
  spacing: 0mm,
  {
    set text(fill: white)
    grid(
      columns: (1fr, auto, auto),
      column-gutter: 6mm,
      align: bottom,
      {
        text(28pt, weight: "bold", tracking: -0.02em, cv.profile.name)
        v(-3mm)
        text(12pt, fill: rgb("#fdba74"), cv.profile.headline)
      },
      {
        set text(8pt)
        set align(right)
        (
          personal.location,
          personal.phone,
          link("mailto:" + personal.email, personal.email),
          if cv.profile.portfolio_url != none {
            link(cv.profile.portfolio_url, strip-https(
              cv.profile.portfolio_url,
            ))
          },
          if cv.profile.github_url != none {
            link(cv.profile.github_url, strip-https(cv.profile.github_url))
          },
        )
          .filter(x => x != none)
          .join(linebreak())
      },
      if personal-details-shown {
        box(radius: 2mm, clip: true, photo-placeholder(
          24mm,
          30mm,
          fill: rgb("#3a4358"),
          figure: rgb("#5b6680"),
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
    let items = (
      if availability != none {
        text(weight: "semibold", fill: accent, availability)
      },
      cv
        .languages
        .map(l => l.name + if l.level != none { " (" + l.level + ")" })
        .join(" · "),
      if personal-details-shown {
        [#t.born #date(personal.birth_date) · #personal.nationality.at(lang)]
      },
    ).filter(x => x != none)
    items.join(h(1fr))
  },
)

#heading-c(t.profile)
#grid(
  columns: (1.7fr, 1fr),
  column-gutter: 7mm,
  cv.profile.summary.map(par).join(),
  block(inset: (left: 4mm), stroke: (left: 1.5pt + accent), {
    text(weight: "bold", t.strengths)
    set par(justify: false)
    list(..cv.profile.strengths)
  }),
)

// Ongoing Projects first, then by end month.
#let all-projects = {
  let projects = cv.engagements.map(engagement => engagement.projects.map(
    project => (..project, engagement: engagement),
  ))
  projects
    .flatten()
    .sorted(key: project => (
      project.end == none,
      if project.end == none { project.start } else { project.end },
    ))
    .rev()
}

#let project-card(project) = block(breakable: false, spacing: 4.5mm, {
  grid(
    columns: (1fr, auto),
    column-gutter: 4mm,
    text(10.5pt, weight: "bold", project.title),
    text(8pt, weight: "semibold", fill: accent, period(
      project.start,
      project.end,
    )),
  )
  v(-1.5mm)
  {
    set text(7.8pt, fill: muted)
    (
      project.role + if project.focus != none { " (" + project.focus + ")" },
      t.client + " " + project.engagement.organization,
      if project.end_client != none { t.end-client + " " + project.end_client },
      project.industry,
      if project.team_size != none { t.team + " " + project.team_size },
    )
      .filter(x => x != none)
      .join("  ·  ")
  }
  v(-0.5mm)
  if project.description.len() > 0 { par(project.description.first()) }
  if project.achievements.len() > 0 {
    list(..project.achievements.slice(0, calc.min(
      achievements-per-project,
      project.achievements.len(),
    )))
  }
  project.skills.map(chip).join(h(1mm))
})

#heading-c(t.selected-projects)
#for project in all-projects.slice(0, calc.min(
  projects-shown,
  all-projects.len(),
)) { project-card(project) }

#heading-c(t.career)
#table(
  columns: (auto, 1fr, auto, auto),
  stroke: (x, y) => (bottom: 0.4pt + luma(215)),
  inset: (x: 0mm, y: 1.8mm),
  column-gutter: 5mm,
  ..cv
    .engagements
    .map(engagement => (
      text(fill: muted, period(engagement.start, engagement.end)),
      text(weight: "semibold", if engagement.role != none {
        engagement.role
      } else { engagement.type_label }),
      engagement.organization,
      text(
        fill: muted,
        engagement.type_label
          + if engagement.projects.len() > 1 {
            " · " + str(engagement.projects.len()) + " " + t.projects
          },
      ),
    ))
    .flatten(),
)

#heading-c(t.skills)
#table(
  columns: (32mm, 1fr),
  stroke: none,
  inset: (x: 0mm, y: 1.2mm),
  column-gutter: 3mm,
  ..skills-by-category(recent-skills(3))
    .pairs()
    .map(((category, skills)) => (
      text(weight: "semibold", category),
      skills
        .map(skill => chip[#skill.name #text(fill: muted, years(skill.months))])
        .join(h(1mm)),
    ))
    .flatten(),
)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 8mm,
  {
    heading-c(t.education)
    for q in education {
      block(
        spacing: 2mm,
        [*#q.name* \ #text(fill: muted)[#if q.issuer != none [#q.issuer · ]#month(q.date)]],
      )
    }
  },
  if certificates.len() > 0 {
    heading-c(t.qualifications)
    for q in certificates {
      block(
        spacing: 2mm,
        [*#qualification-line(q)* \ #text(fill: muted)[#if q.issuer != none [#q.issuer · ]#month(q.date)]],
      )
    }
  },
)
