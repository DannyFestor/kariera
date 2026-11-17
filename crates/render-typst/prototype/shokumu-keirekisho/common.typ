// PROTOTYPE, throwaway. Data access and Japanese date conventions shared by the layout variants. The layout itself
// is deliberately not shared: each variant-*.typ owns its whole page.
//
// Inputs (typst --input): data (root-relative path of the JSON the exporter wrote), today (YYYY-MM-DD),
// era (seireki | wareki), skill-window (years; Skills used in a Project within this window are shown),
// detail-years (years; see is-detailed).

#let data = json(sys.inputs.at("data"))
#let today = sys.inputs.at("today")
#let era = sys.inputs.at("era", default: "seireki")
#let skill-window = int(sys.inputs.at("skill-window", default: "5"))
#let detail-years = int(sys.inputs.at("detail-years", default: "2"))

#let profile = data.profile

#let all-phases = (
  ("requirements", "要件定義"),
  ("basic-design", "基本設計"),
  ("detailed-design", "詳細設計"),
  ("implementation", "製造"),
  ("testing", "テスト"),
  ("operations", "運用・保守"),
)

// Untranslated text stays visible, so the owner sees what the vault still lacks.
#let untranslated(english) = text(fill: rgb("#c0392b"))[［未翻訳］#english]

#let ja(value) = if value == none { none } else if value.ja != none {
  value.ja
} else if value.en != none {
  untranslated(value.en)
}

#let ja-paragraphs(value) = if value == none { none } else if value.ja != none {
  value
    .ja
    .split("\n\n")
    .map(paragraph => paragraph.replace("\n", ""))
    .join(parbreak())
} else if value.en != none { untranslated(value.en.split("\n\n").join(" ")) }

#let ja-bullets(value) = if value == none { () } else if value.ja != none {
  value.ja
} else if value.en != none {
  value.en.map(untranslated)
} else { () }

#let parse-month(month) = month.split("-").map(int)

#let wareki-year(year, month) = {
  let (name, first) = if year * 100 + month >= 201905 {
    ("令和", 2019)
  } else if year >= 1989 {
    ("平成", 1989)
  } else { ("昭和", 1926) }
  let number = year - first + 1
  name + if number == 1 { "元" } else { str(number) } + "年"
}

#let format-month(month) = if month == none { "現在" } else {
  let (year, number) = parse-month(month)
  if era == "wareki" { wareki-year(year, number) + str(number) + "月" } else {
    str(year) + "年" + str(number) + "月"
  }
}

#let format-date(date) = {
  let (year, month, day) = date.split("-").map(int)
  if era == "wareki" { wareki-year(year, month) } else { str(year) + "年" }
  str(month) + "月" + str(day) + "日"
}

#let month-index(month) = {
  let (year, number) = parse-month(month)
  year * 12 + number - 1
}

#let today-index = month-index(today.slice(0, 7))

#let months-of(item) = range(
  month-index(item.start),
  (if item.end == none { today-index } else { month-index(item.end) }) + 1,
)

#let format-duration(months) = {
  let years = calc.quo(months, 12)
  let rest = calc.rem(months, 12)
  if years > 0 { str(years) + "年" }
  if rest > 0 or years == 0 { str(rest) + "ヶ月" }
}

#let organizations = data.organizations.fold((:), (all, organization) => {
  all.insert(organization.name, organization)
  all
})

#let skills-by-name = data.skills.fold((:), (all, skill) => {
  all.insert(skill.name, skill)
  all
})

#let skill-label(name) = {
  let skill = skills-by-name.at(name, default: none)
  if skill == none or skill.ja == none { name } else { skill.ja }
}

// Projects that ended within the last detail-years years, or are ongoing, are printed in full; older ones briefly.
#let is-detailed(project) = (
  project.end == none
    or month-index(project.end) > today-index - 12 * detail-years
)

// Years per Skill: the union of the months of the Projects that link it, overlaps counted once (schema §4).
#let skill-stats = {
  let months = (:)
  for project in data.projects {
    for name in project.skills {
      months.insert(name, months.at(name, default: ()) + months-of(project))
    }
  }
  months
    .pairs()
    .map(((name, list)) => (
      name: name,
      months: list.dedup().len(),
      last: calc.max(..list),
      category: skills-by-name.at(name).category,
    ))
}

#let shown-skills = {
  skill-stats
    .filter(stat => stat.category.slug != "human-language")
    .filter(stat => stat.last > today-index - 12 * skill-window)
    .sorted(key: stat => -stat.months)
}

// The schema's order of Skill categories (§1.12).
#let category-order = (
  "language",
  "framework",
  "library",
  "database",
  "cloud",
  "platform",
  "tool",
  "ai-tool",
  "method",
)

#let skill-categories = {
  category-order
    .map(slug => shown-skills.find(stat => stat.category.slug == slug))
    .filter(stat => stat != none)
    .map(stat => stat.category)
}

#let skills-in(category) = shown-skills.filter(stat => (
  stat.category == category
))

#let human-languages = data.skills.filter(skill => (
  skill.category.slug == "human-language"
))

#let newest-first(items) = (
  items.sorted(key: item => month-index(item.start)).rev()
)

#let engagements = newest-first(data.engagements)

#let projects-of(engagement) = newest-first(data.projects.filter(project => (
  project.engagement == engagement.name
)))

#let organization-of(engagement) = organizations.at(engagement.organization)

#let qualifications = {
  data
    .qualifications
    .filter(qualification => qualification.category.slug != "degree")
    .sorted(key: qualification => month-index(qualification.date))
}

#let qualification-label(qualification) = {
  if qualification.ja == none { qualification.name } else { qualification.ja }
  if (
    qualification.level != none
      and not (
        if qualification.ja == none { qualification.name } else {
          qualification.ja
        }
      ).contains(qualification.level)
  ) {
    " " + qualification.level
  }
  if qualification.score != none { "（" + qualification.score + "）" }
}

#let phase-labels(project) = project.phases.map(phase => phase.ja).join("、")
