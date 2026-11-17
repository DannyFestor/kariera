// PROTOTYPE, throwaway (branch prototype/cv). Variant B, "Sidebar": a tinted sidebar with contact, Skills, languages
// and qualifications beside a main column with the profile and the experience; sans type.
// Content rule: each Engagement with its first summary paragraph and its 3 most recent Projects (first achievement
// each), the rest counted; the 15 Skills with the most years among those used in the last 5 years, as bars.

#import "common.typ": *

#let ink = rgb("#1d2733")
#let accent = rgb("#0f766e")
#let side-fill = rgb("#eef4f3")
#let muted = luma(100)
#let side-width = 64mm
#let side-pad = 8mm
#let projects-per-engagement = 3
#let skills-shown = 15

#set document(title: cv.profile.name + " – CV", author: cv.profile.name)
#set page(
  paper: "a4",
  margin: (left: 0mm, right: 14mm, top: 14mm, bottom: 14mm),
  background: place(left + top, rect(
    width: side-width,
    height: 100%,
    fill: side-fill,
  )),
  footer: context {
    set text(7pt, fill: muted)
    h(side-width + 8mm)
    cv.profile.name
    h(1fr)
    str(here().page()) + "/" + str(counter(page).final().first())
  },
)
#set text(
  font: ("IBM Plex Sans", "IBM Plex Sans JP"),
  size: 9pt,
  fill: ink,
  lang: lang,
)
#set par(leading: 0.55em, spacing: 0.7em)
#set list(indent: 0pt, body-indent: 0.45em, marker: text(fill: accent)[–])

#let side-heading(title) = {
  v(5mm, weak: true)
  text(7.5pt, weight: "bold", fill: accent, tracking: 0.08em, upper(title))
  v(1.5mm, weak: true)
}

#let main-heading(title) = {
  v(6mm, weak: true)
  grid(
    columns: (auto, 1fr),
    column-gutter: 3mm,
    align: horizon,
    text(11pt, weight: "bold", fill: accent, upper(title)),
    line(length: 100%, stroke: 0.6pt + accent.lighten(50%)),
  )
  v(2mm, weak: true)
}

#let skill-bar(skill, longest) = {
  grid(
    columns: (1fr, auto),
    text(8.5pt, skill.name), text(7.5pt, fill: muted, years(skill.months)),
  )
  v(-1.6mm)
  box(width: 100%, height: 2.2pt, fill: white, radius: 1pt, box(
    width: int(skill.months) / longest * 100%,
    height: 2.2pt,
    fill: accent,
    radius: 1pt,
  ))
  v(0.6mm)
}

#let sidebar = {
  set text(8.5pt)
  set par(justify: false)
  if personal-details-shown {
    align(center, box(clip: true, radius: 50%, photo-placeholder(
      34mm,
      34mm,
      fill: white,
    )))
    v(2mm)
  }
  side-heading(t.contact)
  let contact = (
    personal.location,
    personal.phone,
    link("mailto:" + personal.email, personal.email),
    if cv.profile.portfolio_url != none {
      link(cv.profile.portfolio_url, strip-https(cv.profile.portfolio_url))
    },
    if cv.profile.github_url != none {
      link(cv.profile.github_url, strip-https(cv.profile.github_url))
    },
  )
  contact.filter(x => x != none).join(linebreak())
  if personal-details-shown {
    side-heading(t.personal)
    [#t.born: #date(personal.birth_date) \ #t.nationality: #personal.nationality.at(lang)]
  }
  if availability != none {
    v(3mm)
    block(fill: accent, inset: 2.5mm, radius: 2pt, width: 100%, text(
      fill: white,
      weight: "semibold",
      availability,
    ))
  }
  side-heading(t.skills)
  let skills = recent-skills(5).slice(0, calc.min(
    skills-shown,
    recent-skills(5).len(),
  ))
  let longest = if skills.len() > 0 { int(skills.first().months) } else { 1 }
  for skill in skills { skill-bar(skill, longest) }
  side-heading(t.languages)
  for language in cv.languages {
    [*#language.name* #h(1fr) #if language.level != none { language.level }]
    linebreak()
  }
  side-heading(t.education)
  for q in education {
    block(
      spacing: 2mm,
      [*#q.name* \ #if q.issuer != none [#q.issuer \ ]#text(fill: muted, month(q.date))],
    )
  }
  if certificates.len() > 0 {
    side-heading(t.qualifications)
    for q in certificates {
      block(spacing: 2mm, [#qualification-line(q) \ #text(fill: muted, month(
          q.date,
        ))])
    }
  }
}

#let project-entry(project) = block(spacing: 2.2mm, breakable: false, {
  grid(
    columns: (1fr, auto),
    column-gutter: 3mm,
    [*#project.title*],
    text(7.5pt, fill: muted, period(project.start, project.end)),
  )
  v(-1.2mm)
  text(7.5pt, fill: muted, project.skills.join(" · "))
  if project.achievements.len() > 0 {
    v(-1mm)
    list(text(8.5pt, project.achievements.first()))
  }
})

#let engagement-entry(engagement) = block(spacing: 4.5mm, {
  block(breakable: false, spacing: 1.6mm, {
    grid(
      columns: (1fr, auto),
      column-gutter: 3mm,
      text(10pt, weight: "bold", if engagement.role != none {
        engagement.role
      } else { engagement.type_label }),
      text(8pt, fill: muted, period(engagement.start, engagement.end)),
    )
    v(-1.2mm)
    text(fill: accent, weight: "medium", engagement.organization)
    text(fill: muted, " · " + engagement.type_label)
  })
  if engagement.summary.len() > 0 {
    par(justify: true, engagement.summary.first())
  }
  let projects = engagement.projects
  if projects.len() > 0 {
    pad(left: 3mm, {
      for project in projects.slice(0, calc.min(
        projects-per-engagement,
        projects.len(),
      )) {
        project-entry(project)
      }
      let hidden = projects.len() - projects-per-engagement
      if hidden > 0 {
        text(8pt, fill: muted, style: "italic", (t.more-projects)(hidden))
      }
    })
  }
})

#let main = {
  text(24pt, weight: "bold", cv.profile.name)
  v(-3mm)
  text(12pt, fill: accent, cv.profile.headline)
  main-heading(t.profile)
  set par(justify: true)
  cv.profile.summary.map(par).join()
  v(1mm)
  text(weight: "bold", t.strengths)
  list(..cv.profile.strengths)
  main-heading(t.experience)
  for engagement in cv.engagements { engagement-entry(engagement) }
}

#grid(
  columns: (side-width, 1fr),
  column-gutter: 8mm,
  pad(x: side-pad, sidebar), main,
)
