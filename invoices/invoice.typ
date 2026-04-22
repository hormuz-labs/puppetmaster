#set page(
  margin: (x: 2.5cm, y: 2.5cm),
  paper: "a4",
)
#set text(
  font: "Helvetica",
  size: 11pt,
  fill: rgb("#111111"),
)

#let text_color = rgb("#111111")
#let light_grey = rgb("#f5f5f5")
#let dark_bg = rgb("#333333")
#let border_color = rgb("#dddddd")

// Header
#grid(
  columns: (1fr, 1fr),
  [
    #v(15pt)
    #text(weight: "bold", size: 12pt, fill: rgb("#666666"))[Bill To:] \
    #text(weight: "bold", size: 14pt)[Relocal Pte Ltd]
  ],
  align(right)[
    #text(size: 36pt, weight: "bold", fill: text_color)[INVOICE] \
    #v(5pt)
    #text(fill: rgb("#666666"))[
      INV-2026-001 \
      Date: April 22, 2026
    ]
  ]
)

#v(80pt)

// Title
#text(size: 16pt, weight: "bold")[Professional Consultancy Services]
#v(15pt)

// Table
#table(
  columns: (1fr, auto),
  align: (left, right),
  stroke: none,
  fill: (x, y) => if y == 0 { dark_bg } else if y == 2 { light_grey } else { none },
  inset: 12pt,
  [#text(fill: white, weight: "bold")[Description]], [#text(fill: white, weight: "bold")[Amount (USD)]],
  [Professional Consultancy Fees], [\$700.00],
  [#text(weight: "bold")[Total]], [#text(weight: "bold", size: 12pt)[\$700.00]],
)

#v(60pt)

// Payment Details Box
#block(
  width: 100%,
  stroke: border_color + 1pt,
  radius: 6pt,
  inset: 20pt,
)[
  #text(weight: "bold", size: 12pt)[Payment Details — Bank Transfer]
  #v(15pt)
  
  #grid(
    columns: (120pt, 1fr),
    row-gutter: 10pt,
    [#text(fill: rgb("#666666"))[Account Holder:]], [#text(weight: "bold")[SHANUR RAHMAN]],
    [#text(fill: rgb("#666666"))[Account Number:]], [20012209145231],
    [#text(fill: rgb("#666666"))[IFSC Code:]], [STCB0000065],
    [#text(fill: rgb("#666666"))[Bank:]], [SBM Bank India Ltd],
    [#text(fill: rgb("#666666"))[Currency:]], [USD]
  )
]

#v(1fr)

#align(center)[
  #text(fill: rgb("#888888"))[Thank you for your business.]
]
