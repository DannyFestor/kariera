// PROTOTYPE, throwaway. Variant A「厚生労働省様式」: the ruled two-page form of the 2021 厚生労働省履歴書様式例 (which
// replaced the JIS Z 8303 example). Fixed rows, empty rows filled up to the page, one combined 学歴・職歴 table.

#let doc = json(sys.inputs.data)
#let photo = sys.inputs.at("photo", default: none)

#let rule = 0.5pt
#let frame = 1.2pt
#let row-height = 7.6mm

#set document(title: doc.title, author: doc.name)
#set page(paper: "a4", margin: (x: 15mm, top: 14mm, bottom: 12mm))
#set text(font: "BIZ UDPGothic", size: 9.5pt, lang: "ja", region: "JP")
#set par(justify: false, leading: 0.5em)

#let small(body) = text(size: 7pt, body)
#let dotted = (thickness: rule, dash: "dotted")

#let photo-box = box(
  width: 30mm,
  height: 40mm,
  stroke: (thickness: rule, dash: "dashed"),
  if photo != none {
    image(photo, width: 30mm, height: 40mm, fit: "cover")
  } else {
    align(center + horizon, small[
      写真をはる位置 \ \
      写真をはる必要が \ ある場合 \
      1. 縦 36～40mm \ 　 横 24～30mm \
      2. 本人単身胸から上
    ])
  },
)

#let person = table(
  columns: (18mm, 1fr, 22mm),
  rows: (6mm, 15mm, 8mm, 8mm, 8mm),
  stroke: (x, y) => (
    top: if y == 0 { frame } else if y == 1 { dotted } else { rule },
    bottom: if y == 4 { frame } else { none },
    left: if x == 0 { frame } else { rule },
    right: if x == 2 { frame } else { none },
  ),
  inset: (x: 4pt, y: 3pt),
  align: (left + horizon, left + horizon, center + horizon),
  small[ふりがな],
  doc.name_kana,
  table.cell(
    rowspan: 2,
    small[※性別]
      + linebreak()
      + v(1mm)
      + if doc.gender != none { text(size: 11pt, doc.gender) },
  ),
  [氏　名], text(size: 16pt, doc.name),
  table.cell(colspan: 3)[#doc.birth_date　（#doc.age）],
  table.cell(colspan: 3)[
    #small[国籍]　#doc.nationality
    #h(6mm) #small[在留資格]　#doc.residence_status#if doc.work_restriction != none [（#doc.work_restriction）]
  ],
  table.cell(colspan: 3)[
    #if (
      doc.residence_expiry != none
    ) [#small[在留期限]　#doc.residence_expiry #h(6mm)]
    #if doc.residence_note != none { doc.residence_note }
  ],
)

#let address-block(label, value, note) = table(
  columns: (18mm, 1fr, 52mm),
  rows: (6mm, 15mm),
  stroke: (x, y) => (
    top: if y == 0 { rule } else { dotted },
    bottom: rule,
    left: if x == 0 { frame } else { rule },
    right: if x == 2 { frame } else { none },
  ),
  inset: (x: 4pt, y: 3pt),
  align: left + horizon,
  small[ふりがな],
  if value != none { small(value.kana) },
  small[電話] + linebreak() + if value != none { doc.phone },

  label,
  [#if value != none [〒#value.postal_code \ #value.text] else { note }],
  small[E-mail] + linebreak() + if value != none { text(size: 8pt, doc.email) },
)

#grid(
  columns: (1fr, 34mm),
  column-gutter: 4mm,
  [
    #grid(
      columns: (1fr, auto),
      align: (left + bottom, right + bottom),
      text(size: 18pt, weight: "bold", tracking: 0.6em, doc.title), doc.as_of,
    )
    #v(2mm)
    #person
  ],
  align(right + bottom, photo-box),
)
#address-block([現住所], doc.address, none)
#address-block(
  [連絡先],
  doc.contact,
  [同上],
)

#let year-month-table(heading, entries, rows-to-fill) = {
  let lines = entries.map(entry => (entry.year, entry.month, entry.text))
  let padding = calc.max(0, rows-to-fill - lines.len())
  table(
    columns: (16mm, 10mm, 1fr),
    rows: row-height,
    stroke: (x, y) => (
      left: if x == 0 { frame } else if x == 1 { dotted } else { rule },
      right: if x == 2 { frame } else { none },
      top: if y == 0 { frame } else { rule },
      bottom: rule,
    ),
    inset: (x: 4pt, y: 0pt),
    align: (x, y) => if x < 2 { center + horizon } else { left + horizon },
    table.header(small[年], small[月], align(center, small(heading))),
    ..lines.flatten(),
    ..range(padding * 3).map(_ => []),
  )
}

#let history = (
  (year: "", month: "", text: align(center)[学　歴]),
  ..doc.education,
  (year: "", month: "", text: []),
  (year: "", month: "", text: align(center)[職　歴]),
  ..doc.career,
  (year: "", month: "", text: [現在に至る]),
  (year: "", month: "", text: align(right)[以上]),
)

#v(3mm)
#year-month-table([学歴・職歴（各別にまとめて書く）], history, 25)
#v(3mm)
#year-month-table([免許・資格], doc.licences, 6)

#let free-box(heading, body, height) = block(
  width: 100%,
  height: height,
  stroke: frame,
  inset: 5pt,
)[
  #small(heading)
  #v(1mm)
  #body
]

#v(3mm)
#free-box(
  [志望の動機、特技、好きな学科、アピールポイントなど],
  doc.motivation.join(parbreak())
    + parbreak()
    + [語学：#doc.languages.map(language => [#language.name（#language.level）]).join("、")],
  3fr,
)
#v(3mm)
#free-box(
  [本人希望記入欄（特に給料・職種・勤務時間・勤務地・その他についての希望などがあれば記入）],
  doc.wishes.join(linebreak()),
  2fr,
)
