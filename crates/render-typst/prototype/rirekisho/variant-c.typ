// PROTOTYPE, throwaway. Variant C「フリーフォーム」: no ruled form. The look of the 職務経歴書 (accent colour, section
// rules), personal details as a two-column list beside the photo, 職歴 as one line per Engagement with its period
// instead of separate start and end lines.

#let doc = json(sys.inputs.data)
#let photo = sys.inputs.at("photo", default: none)

#let accent = rgb("#1f3a5f")
#let muted = luma(110)

#set document(title: doc.title, author: doc.name)
#set page(
  paper: "a4",
  margin: (x: 16mm, top: 18mm, bottom: 18mm),
  footer: context align(right, text(
    size: 7.5pt,
    fill: muted,
    counter(page).display("1 / 1", both: true),
  )),
)
#set text(font: "BIZ UDPGothic", size: 9.5pt, lang: "ja", region: "JP")
#set par(justify: true, leading: 0.7em, spacing: 0.9em)

#let section(title) = block(
  sticky: true,
  above: 1.4em,
  below: 0.6em,
  width: 100%,
  stroke: (bottom: 1pt + accent),
  inset: (bottom: 3pt),
  text(size: 11pt, weight: "bold", fill: accent, title),
)

#let details(..rows) = grid(
  columns: (24mm, 1fr),
  row-gutter: 6pt,
  ..rows.pos().map(((key, value)) => (text(fill: muted, key), value)).flatten(),
)

#let photo-box = box(
  width: 30mm,
  height: 40mm,
  stroke: 0.5pt + luma(180),
  if photo != none {
    image(photo, width: 30mm, height: 40mm, fit: "cover")
  } else { align(center + horizon, text(size: 7.5pt, fill: muted)[写真]) },
)

#grid(
  columns: (1fr, auto),
  align: (left + bottom, right + bottom),
  text(size: 20pt, weight: "bold", fill: accent, doc.title),
  text(size: 8.5pt, fill: muted, doc.as_of),
)
#v(4mm)
#grid(
  columns: (1fr, 30mm),
  column-gutter: 8mm,
  [
    #text(size: 8pt, fill: muted, doc.name_kana) \
    #text(size: 18pt, weight: "bold", doc.name)
    #v(3mm)
    #details(
      ([生年月日], [#doc.birth_date（#doc.age）]),
      ([国籍], doc.nationality),
      ([在留資格], [#doc.residence_status（在留期限 #doc.residence_expiry）]),
      ([現住所], [〒#doc.address.postal_code #doc.address.text]),
      ([電話], doc.phone),
      ([E-mail], doc.email),
    )
  ],
  photo-box,
)

#let dated(entries) = grid(
  columns: (24mm, 1fr),
  row-gutter: 7pt,
  ..entries
    .map(entry => (
      text(fill: muted)[#(entry.year)年#(entry.month)月],
      entry.text,
    ))
    .flatten(),
)

#section[学歴]
#dated(doc.education)

#section[職歴]
#grid(
  columns: (44mm, 1fr, auto),
  row-gutter: 7pt,
  column-gutter: 6pt,
  ..doc
    .career_ranges
    .map(range => (
      text(fill: muted, range.period),
      [#text(weight: "bold", range.organization)#if (
          range.role != none
        ) [　#range.role]],
      text(size: 8.5pt, fill: muted, range.kind),
    ))
    .flatten(),
)
#align(right)[以上]

#section[免許・資格]
#dated(doc.licences)

#section[語学]
#details(..doc.languages.map(language => (language.name, language.level)))

#section[自己PR・志望動機]
#doc.motivation.join(parbreak())
#list(..doc.strengths)

#section[本人希望記入欄]
#doc.wishes.join(parbreak())
