// PROTOTYPE, throwaway (branch prototype/cv). Variant A, "Timeline": the tabular Lebenslauf. A narrow date column on
// the left and the content on the right, one column throughout; serif type.
// Content rule: every Engagement with its full summary, every Project as one line with its Skills; Skills used in the
// last 5 years, grouped by category.

#import "common.typ": *

#let accent = rgb("#1f4e79")
#let muted = luma(105)
#let label-width = 31mm
#let gap = 5mm

#set document(title: cv.profile.name + " – CV", author: cv.profile.name)
#set page(
  paper: "a4",
  margin: (x: 18mm, top: 16mm, bottom: 18mm),
  footer: context {
    set text(7.5pt, fill: muted)
    cv.profile.name
    h(1fr)
    t.page + " " + str(here().page()) + "/" + str(counter(page).final().first())
  },
)
#set text(font: ("IBM Plex Serif", "Noto Serif JP"), size: 9.5pt, lang: lang)
#set par(justify: true, leading: 0.55em, spacing: 0.75em)
#set list(indent: 0pt, body-indent: 0.5em, marker: text(fill: accent)[•])

#let row(label, body) = grid(
  columns: (label-width, 1fr),
  column-gutter: gap,
  align(right, text(fill: muted, size: 8.5pt, hyphenate: false, label)), body,
)

#let section(title) = {
  v(4mm, weak: true)
  grid(
    columns: (label-width, 1fr),
    column-gutter: gap,
    align(right + horizon, line(length: 100%, stroke: 2.5pt + accent)),
    text(13pt, fill: accent, weight: "regular", smallcaps(title)),
  )
  v(2mm)
}

// Header
#grid(
  columns: (1fr, auto, auto),
  column-gutter: 6mm,
  align: bottom,
  {
    text(26pt, weight: "light", cv.profile.name)
    v(-2mm)
    text(12pt, fill: accent, style: "italic", cv.profile.headline)
  },
  {
    set text(8.5pt)
    set align(right)
    let lines = (
      personal.location,
      personal.phone,
      link("mailto:" + personal.email, personal.email),
      cv.profile.portfolio_url,
      cv.profile.github_url,
    )
    lines
      .filter(x => x != none)
      .map(x => if type(x) == str and x.starts-with("https://") {
        link(x, strip-https(x))
      } else { x })
      .join(linebreak())
  },
  if personal-details-shown { photo-placeholder(26mm, 32mm) },
)
#v(3mm)

#if personal-details-shown {
  section(t.personal)
  row(t.born, date(personal.birth_date))
  row(t.nationality, personal.nationality.at(lang))
}

#section(t.profile)
#row([], {
  cv.profile.summary.map(par).join()
  if availability != none {
    text(fill: accent, weight: "semibold", availability)
  }
})
#v(1mm)
#row(t.strengths, list(..cv.profile.strengths))

#section(t.experience)
#for engagement in cv.engagements {
  row(period(engagement.start, engagement.end), {
    block(spacing: 1.2mm, {
      text(weight: "bold", if engagement.role != none { engagement.role } else {
        engagement.type_label
      })
      [, ]
      text(style: "italic", engagement.organization)
      h(1fr)
      text(8pt, fill: muted, smallcaps(engagement.type_label))
    })
    set text(9pt)
    engagement.summary.map(par).join()
    if engagement.projects.len() > 0 {
      set text(8.5pt)
      set par(justify: false)
      grid(
        columns: (auto, 1fr),
        column-gutter: 3mm,
        row-gutter: 1.6mm,
        ..engagement
          .projects
          .map(project => (
            text(fill: muted, period(project.start, project.end)),
            [*#project.title* #h(0.3em) #text(fill: muted, project.skills.join(", "))],
          ))
          .flatten(),
      )
    }
  })
  v(3mm)
}

#section(t.skills)
#for (category, skills) in skills-by-category(recent-skills(5)) {
  row(
    category,
    skills
      .map(skill => [#skill.name #text(
          fill: muted,
          size: 7.5pt,
          "(" + years(skill.months) + ")",
        )])
      .join(", "),
  )
  v(1.2mm)
}

#section(t.languages)
#for language in cv.languages {
  row(language.name, if language.level != none { language.level })
  v(1.2mm)
}

#section(t.education)
#for q in education {
  row(
    if q.start != none { month(q.start) + " – " + month(q.date) } else {
      month(q.date)
    },
    [
      *#q.name*#if q.issuer != none [, #emph(q.issuer)]
    ],
  )
  v(1.2mm)
}

#if certificates.len() > 0 {
  section(t.qualifications)
  for q in certificates {
    row(month(q.date), [
      #qualification-line(q)#if q.issuer != none [, #emph(q.issuer)]
    ])
    v(1.2mm)
  }
}
