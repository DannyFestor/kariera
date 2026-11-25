// The 履歴書, layout variant A「厚生労働省様式」 (issue #22, branch prototype/rirekisho): the ruled two-page form of the
// 2021 厚生労働省履歴書様式例. The content and the form's words come from the documents crate as JSON in
// sys.inputs.data, already in Japanese; this file holds the layout.

#let doc = json(sys.inputs.data)
#let t = doc.labels
#let photo = sys.inputs.at("photo", default: none)

#let rule = 0.5pt
#let frame = 1.2pt
#let row-height = 7.6mm
#let gap = 3mm
#let margin-bottom = 12mm
// Page 2 keeps at least this many ruled rows of 学歴・職歴, and 免許・資格 as many (owner's decision on issue #22).
#let min-rows = 6

#set document(title: t.title, author: doc.name)
#set page(paper: "a4", margin: (x: 15mm, top: 14mm, bottom: margin-bottom))
#set text(font: "BIZ UDPGothic", size: 9.5pt, lang: "ja", region: "JP")
#set par(justify: false, leading: 0.5em)

#let small(body) = text(size: 7pt, body)
#let dotted = (thickness: rule, dash: "dotted")
#let shown(value) = if value != none { value }

#let photo-box = box(
  width: 30mm,
  height: 40mm,
  stroke: (thickness: rule, dash: "dashed"),
  if photo != none {
    image(photo, width: 30mm, height: 40mm, fit: "cover")
  } else {
    align(center + horizon, small(t.photo_note.join(linebreak())))
  },
)

#let residence-rows(residence) = (
  table.cell(colspan: 3)[
    #small(t.nationality)　#shown(residence.nationality)
    #h(6mm) #small(t.residence_status)　#shown(residence.status)
  ],
  table.cell(colspan: 3)[
    #small(t.residence_expiry)　#shown(residence.expiry)
    #h(6mm) #shown(residence.note)
  ],
)

#let person = {
  let rows = (6mm, 15mm, 8mm)
  if doc.residence != none { rows += (8mm, 8mm) }
  let last = rows.len() - 1
  table(
    columns: (18mm, 1fr, 22mm),
    rows: rows,
    stroke: (x, y) => (
      top: if y == 0 { frame } else if y == 1 { dotted } else { rule },
      bottom: if y == last { frame } else { none },
      left: if x == 0 { frame } else { rule },
      right: if x == 2 { frame } else { none },
    ),
    inset: (x: 4pt, y: 3pt),
    align: (left + horizon, left + horizon, center + horizon),
    small(t.furigana),
    shown(doc.name_kana),
    table.cell(
      rowspan: 2,
      small(t.gender)
        + linebreak()
        + v(1mm)
        + if doc.gender != none { text(size: 11pt, doc.gender) },
    ),
    t.name,
    text(size: 16pt, doc.name),
    table.cell(colspan: 3)[
      #shown(doc.birth_date)#if doc.age != none [　（#doc.age）]
    ],
    ..if doc.residence != none { residence-rows(doc.residence) } else { () },
  )
}

#let address-block(label, kana, body, phone, email) = table(
  columns: (18mm, 1fr, 52mm),
  rows: (6mm, 15mm),
  stroke: (x, y) => (
    top: if y == 0 or x == 2 { rule } else { dotted },
    bottom: rule,
    left: if x == 0 { frame } else { rule },
    right: if x == 2 { frame } else { none },
  ),
  inset: (x: 4pt, y: 3pt),
  align: left + horizon,
  small(t.furigana),
  if kana != none { small(kana) },
  small(t.phone) + h(2mm) + shown(phone),

  label,
  body,
  small(t.email) + linebreak() + if email != none { text(size: 8pt, email) },
)

#let year-month-table(heading, lines, rows) = {
  let padding = calc.max(0, rows - lines.len())
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
    table.header(small(t.year), small(t.month), align(center, small(heading))),
    ..lines.flatten(),
    ..range(padding * 3).map(_ => []),
  )
}

#let dated(lines) = lines.map(line => (line.year, line.month, line.text))
#let undated(body) = ("", "", body)

#let history = (
  undated(align(center, t.education)),
  ..dated(doc.education),
  undated([]),
  undated(align(center, t.career)),
  ..dated(doc.career),
  undated(t.until_now),
  undated(align(right, t.closing)),
)

// The rows of 学歴・職歴 that fill page 1 below <history-start>; needs context.
#let rows-on-page-one() = {
  let top = locate(<history-start>).position().y
  let free = page.height - margin-bottom - top - row-height
  // Rounding must never let one row too many push the table onto page 2.
  calc.max(0, calc.floor(free / row-height - 0.01))
}

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

#grid(
  columns: (1fr, 34mm),
  column-gutter: 4mm,
  [
    #grid(
      columns: (1fr, auto),
      align: (left + bottom, right + bottom),
      text(size: 18pt, weight: "bold", tracking: 0.6em, t.title), doc.as_of,
    )
    #v(2mm)
    #person
  ],
  align(right + bottom, photo-box),
)
#address-block(
  t.address,
  doc.address.kana,
  [#t.postal_mark#shown(doc.address.postal_code) \ #shown(doc.address.text)],
  doc.phone,
  doc.email,
)
#address-block(t.contact, none, t.same_as_address, none, none)

#v(gap)
#metadata(none) <history-start>
#context {
  let rows = rows-on-page-one()
  year-month-table(
    t.history,
    history.slice(0, calc.min(rows, history.len())),
    rows,
  )
}
#pagebreak()
#context {
  let rows = rows-on-page-one()
  year-month-table(
    t.history,
    history.slice(calc.min(rows, history.len())),
    min-rows,
  )
}

#v(gap)
#year-month-table(t.licences, dated(doc.licences), min-rows)

#v(gap)
#free-box(t.appeal, doc.appeal.join(parbreak()), 3fr)
#v(gap)
#free-box(t.wishes, doc.wishes.join(linebreak()), 2fr)
