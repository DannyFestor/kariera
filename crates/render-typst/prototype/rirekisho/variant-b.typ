// PROTOTYPE, throwaway. Variant B「転職者用」: a ruled form like the 転職者用 履歴書 forms, with 学歴 and 職歴 in separate
// tables sized to their content, 国籍・在留資格 rows for a foreign applicant, and separate 自己PR, 志望動機 and
// 本人希望 boxes. No empty rows to fill up a page.

#let doc = json(sys.inputs.data)
#let photo = sys.inputs.at("photo", default: none)

#let rule = 0.5pt
#let frame = 1.2pt
#let band = luma(238)

#set document(title: doc.title, author: doc.name)
#set page(
  paper: "a4",
  margin: (x: 15mm, top: 14mm, bottom: 14mm),
  footer: context align(center, text(
    size: 7.5pt,
    counter(page).display("1 / 1", both: true),
  )),
)
#set text(font: "BIZ UDPGothic", size: 9.5pt, lang: "ja", region: "JP")
#set par(justify: false, leading: 0.5em)

#let small(body) = text(size: 7.5pt, body)

#let photo-box = box(
  width: 30mm,
  height: 40mm,
  stroke: (thickness: rule, dash: "dashed"),
  if photo != none {
    image(photo, width: 30mm, height: 40mm, fit: "cover")
  } else { align(center + horizon, small[写真\ 縦40mm × 横30mm]) },
)

#let form(..rows) = table(
  columns: (24mm, 1fr),
  stroke: (x, y) => (
    left: if x == 0 { frame } else { rule },
    right: frame,
    top: rule,
    bottom: rule,
  ),
  inset: (x: 5pt, y: 5pt),
  fill: (x, y) => if x == 0 { band },
  align: (x, y) => (if x == 0 { center } else { left }) + horizon,
  ..rows.pos().flatten(),
)

#let personal = form(
  (small[フリガナ], doc.name_kana),
  ([氏名], text(size: 15pt, doc.name)),
  ([生年月日], [#doc.birth_date（#doc.age）]),
  ([国籍], doc.nationality),
  ([在留資格], [#doc.residence_status（在留期限 #doc.residence_expiry）]),
)

#let contact = form(
  (small[フリガナ], small(doc.address.kana)),
  ([現住所], [〒#doc.address.postal_code \ #doc.address.text]),
  ([電話], doc.phone),
  ([E-mail], doc.email),
)

#let section(title) = block(
  sticky: true,
  above: 5mm,
  below: 0mm,
  width: 100%,
  fill: luma(60),
  inset: (x: 5pt, y: 4pt),
  text(
    fill: white,
    weight: "bold",
    title,
  ),
)

#let year-month(entries, closing: ()) = table(
  columns: (16mm, 10mm, 1fr),
  stroke: (x, y) => (
    left: if x == 0 { frame } else { rule },
    right: if x == 2 { frame } else { none },
    bottom: rule,
  ),
  inset: (x: 5pt, y: 5pt),
  align: (x, y) => if x < 2 { center } else { left },
  table.header(..([年], [月], [内容]).map(small)),
  ..entries.map(entry => (entry.year, entry.month, entry.text)).flatten(),
  ..closing.map(text => ([], [], text)).flatten(),
)

#let free-box(body, height) = block(
  width: 100%,
  stroke: (left: frame, right: frame, bottom: frame),
  inset: 6pt,
  grid(
    columns: (0pt, 1fr),
    box(height: height), body,
  ),
)

#grid(
  columns: (1fr, auto),
  align: (left + bottom, right + bottom),
  text(size: 20pt, weight: "bold", tracking: 0.5em, doc.title), doc.as_of,
)
#v(2mm)
#grid(
  columns: (1fr, 34mm),
  column-gutter: 4mm,
  personal, align(right + top, photo-box),
)
#v(3mm)
#contact

#section[学歴]
#year-month(doc.education)

#section[職歴]
#year-month(doc.career, closing: ([現在に至る], align(right)[以上]))

#section[免許・資格]
#year-month(doc.licences)

#section[語学]
#table(
  columns: (40mm, 1fr),
  stroke: (x, y) => (
    left: if x == 0 { frame } else { rule },
    right: if x == 1 { frame } else { none },
    bottom: rule,
  ),
  inset: 5pt,
  ..doc.languages.map(language => (language.name, language.level)).flatten(),
)

#section[自己PR・特技]
#free-box(doc.strengths.map(item => [・#item]).join(linebreak()), 30mm)

#section[志望動機]
#free-box(doc.motivation.join(parbreak()), 30mm)

#section[本人希望記入欄]
#free-box(doc.wishes.join(parbreak()), 20mm)
