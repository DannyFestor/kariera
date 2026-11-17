// PROTOTYPE, throwaway (branch prototype/cv): what the CV layout variants share. Not layout: data, words and dates.

#let cv = json(sys.inputs.at("data"))
#let personal = yaml(sys.inputs.at("personal"))
#let lang = cv.lang

#let words = (
  en: (
    profile: "Profile",
    strengths: "Strengths",
    experience: "Experience",
    projects: "Projects",
    selected-projects: "Selected projects",
    career: "Career",
    skills: "Skills",
    languages: "Languages",
    education: "Education",
    qualifications: "Certificates and licences",
    contact: "Contact",
    personal: "Personal details",
    born: "Born",
    nationality: "Nationality",
    present: "present",
    since: "since",
    available: "Available from",
    days-per-week: "days per week",
    team: "team of",
    client: "Client",
    end-client: "End client",
    via: "as",
    years-short: "y",
    more-projects: n => str(n) + " further projects",
    page: "Page",
  ),
  de: (
    profile: "Profil",
    strengths: "Stärken",
    experience: "Berufserfahrung",
    projects: "Projekte",
    selected-projects: "Ausgewählte Projekte",
    career: "Werdegang",
    skills: "IT-Kenntnisse",
    languages: "Sprachen",
    education: "Ausbildung",
    qualifications: "Zertifikate und Lizenzen",
    contact: "Kontakt",
    personal: "Persönliche Daten",
    born: "Geburtsdatum",
    nationality: "Staatsangehörigkeit",
    present: "heute",
    since: "seit",
    available: "Verfügbar ab",
    days-per-week: "Tage pro Woche",
    team: "Team aus",
    client: "Kunde",
    end-client: "Endkunde",
    via: "als",
    years-short: "J.",
    more-projects: n => "+ " + str(n) + " weitere Projekte",
    page: "Seite",
  ),
)
#let t = words.at(lang)

#let month-names = (
  en: (
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec",
  ),
)

// German CVs write months as MM/YYYY; English ones as "Oct 2023".
#let month(value) = {
  let (year, number) = value.split("-")
  if lang == "de" { number + "/" + year } else {
    month-names.en.at(int(number) - 1) + " " + year
  }
}

#let date(value) = {
  let (year, number, day) = value.split("-")
  if lang == "de" { day + "." + number + "." + year } else {
    str(int(day)) + " " + month-names.en.at(int(number) - 1) + " " + year
  }
}

#let period(start, end) = {
  if end == none { t.since + " " + month(start) } else if start == end {
    month(start)
  } else { month(start) + " – " + month(end) }
}

#let years(months) = {
  let value = calc.round(int(months) / 12, digits: 1)
  let written = if lang == "de" { str(value).replace(".", ",") } else {
    str(value)
  }
  written + " " + t.years-short
}

#let month-index(value) = {
  let (year, number) = value.split("-")
  int(year) * 12 + int(number) - 1
}

// The Skills a CV shows: those used in a Project within the last `within-years` years.
#let recent-skills(within-years) = {
  let cutoff = month-index(cv.build_month) - within-years * 12
  cv
    .skills
    .filter(skill => (
      skill.last_used != none and month-index(skill.last_used) > cutoff
    ))
    .sorted(key: skill => -int(skill.months))
}

#let skills-by-category(skills) = {
  let groups = (:)
  for skill in skills {
    let entries = groups.at(skill.category_label, default: ())
    groups.insert(skill.category_label, entries + (skill,))
  }
  groups
}

#let education = cv.qualifications.filter(q => q.category == "degree")
#let certificates = cv.qualifications.filter(q => q.category != "degree")

#let qualification-line(q) = {
  (q.name, q.level, q.score).filter(x => x != none).dedup().join(", ")
}

#let personal-details-shown = lang in personal.at("show_on", default: ())

#let availability = {
  let profile = cv.profile
  if profile.available_from == none { none } else {
    let days = if profile.days_per_week_min == none { "" } else if (
      profile.days_per_week_min == profile.days_per_week_max
    ) { ", " + profile.days_per_week_min + " " + t.days-per-week } else {
      (
        ", "
          + profile.days_per_week_min
          + "–"
          + profile.days_per_week_max
          + " "
          + t.days-per-week
      )
    }
    t.available + " " + date(profile.available_from) + days
  }
}

#let strip-https(url) = url.replace("https://", "")

// A stand-in for the photo of personal.yaml: the prototype holds no picture of anyone.
#let photo-placeholder(width, height, fill: luma(225), figure: luma(170)) = box(
  width: width,
  height: height,
  fill: fill,
  clip: true,
  place(center + bottom, {
    stack(
      dir: ttb,
      spacing: -0.06 * height,
      align(center, circle(radius: 0.2 * width, fill: figure)),
      ellipse(width: 0.95 * width, height: 0.55 * height, fill: figure),
    )
  }),
)
