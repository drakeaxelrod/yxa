
#let clrs = (
  gray: rgb("#bbbbbb"),
  base: rgb("#000000"),
  media: rgb("#4c00b0"),
  nav: rgb("#008080"),
  mouse: rgb("#ffde21"),
  symbol: rgb("#008000"),
  number: rgb("#0000ff"),
  function: rgb("#ff0000"),


  purple: rgb("#800080"),
  red: rgb("#ff0000"),
  blue: rgb("#0000ff"),
  green: rgb("#008000"),
  yellow: rgb("#ffff00"),
  orange: rgb("#ffa500"),
  black: rgb("#000000"),
  white: rgb("#ffffff"),
)

#let keycap = (
  tl: [TL],
  tr: [TR],
  bl: [BL],
  br: [BR],
  ch: [CH],
) => {
  box(
    height: 1.15cm,
    width: 1.15cm,
    stroke: 0.1em + clrs.gray,
    fill: clrs.white,
    inset: 0.5em,
    {
      text(fill: red.darken(20%), tl) + h(1fr) + text(fill: green.darken(20%),tr)
      v(1fr)
      align(center+horizon, text(fill: clrs.base, ch))
      v(1fr)
      text(fill: purple.darken(0%), bl) + h(1fr) + text(fill: blue.darken(0%), br)
    }
  )
}

#set text(size: 0.7em, font: "Lilex Nerd Font", weight: 700)

#text(size: 2.5em, "Left side")
#stack(dir: ltr, spacing: 1em,
  keycap(
    ch: "Q",
    tr: "{",
    tl: "F12",
    br: "[",
    bl: "",

  ),
  keycap(
    ch: "W",
    tr: "&",
    tl: "F7",
    br: "7",
    bl: "󰝁"
  ),
  keycap(
    ch: "F",
    tr: "*",
    tl: "F8",
    br: "8",
    bl: ""
  ),
  keycap(
    ch: "P",
    tr: "(",
    tl: "F9",
    br: "9",
    bl: ""
  ),
  keycap(
    ch: "B",
    tr: "}",
    tl: "󰐪",
    br: "]",
    bl: ""
  )
)
#stack(dir: ltr, spacing: 1em,
  keycap(
    ch: "A",
    tr: ":",
    tl: "F11",
    br: ";",
    bl: "󰘳",
  ),
  keycap(
    ch: "R",
    tr: "$",
    tl: "F4",
    br: "4",
    bl: "󰘵",
  ),
  keycap(
    ch: "S",
    tr: "%",
    tl: "F5",
    br: "5",
    bl: "󰘴",
  ),
  keycap(
    ch: "T",
    tr: "^",
    tl: "F6",
    br: "6",
    bl: "󰘶",
  ),
  keycap(
    ch: "G",
    tr: "+",
    tl: "󱅜",
    br: "=",
    bl: "",
  )
)
#stack(dir: ltr, spacing: 1em,
  keycap(
    ch: "Z",
    tr: "~",
    tl: "F10",
    br: "`",
    bl: "",
  ),
  keycap(
    ch: "X",
    tr: "!",
    tl: "F1",
    br: "1",
    bl: "󰘵",
  ),
  keycap(
    ch: "C",
    tr: "@",
    tl: "F2",
    br: "2",
    bl: "",
  ),
  keycap(
    ch: "D",
    tr: "#",
    tl: "F3",
    br: "3",
    bl: "",
  ),
  keycap(
    ch: "V",
    tr: "|",
    tl: "󱤳",
    br: "\\",
    bl: "",
  )
)
#stack(dir: ltr, spacing: 1em,
  keycap(
    ch: "󰻈",
    tr: "(",
    tl: "󰍜",
    br: ".",
    bl: "",
  ),
  keycap(
    ch: "󱁐",
    tr: ")",
    tl: "󱁐",
    br: "0",
    bl: "",
  ),
  keycap(
    ch: "",
    tr: "_",
    tl: "",
    br: "-",
    bl: "",
  ),
)

#text(size: 2.5em, "Right side")

#stack(dir: ltr, spacing: 1em,
  keycap(
    ch: "J",
    tr: "󰑎",
    tl: "",
    br: "󰑎",
    bl: "󰔎",
  ),
  keycap(
    ch: "L",
    tr: "",
    tl: "",
    br: "",
    bl: "",
  ),
  keycap(
    ch: "U",
    tr: "󰆏",
    tl: "",
    br: "󰆏",
    bl: "",
  ),
  keycap(
    ch: "Y",
    tr: "",
    tl: "",
    br: "",
    bl: "",
  ),
  keycap(
    ch: "' \"",
    tr: "󰕌",
    tl: "",
    br: "󰕌",
    bl: "",
  )
)
#stack(dir: ltr, spacing: 1em,
  keycap(
    ch: "M",
    tr: "",
    tl: "",
    br: "󰪛",
    bl: "",
  ),
  keycap(
    ch: "N",
    tr: " 󰍽",
    tl: "",
    br: "",
    bl: "󰒮",
  ),
  keycap(
    ch: "E",
    tr: " 󰍽",
    tl: "",
    br: "",
    bl: "󰖀",
  ),
  keycap(
    ch: "I",
    tr: " 󰍽",
    tl: "",
    br: "",
    bl: "󰕾",
  ),
  keycap(
    ch: "O",
    tr: " 󰍽",
    tl: "",
    br: "",
    bl: "󰒭",
  )
)
#stack(dir: ltr, spacing: 1em,
  keycap(
    ch: "K",
    tr: "",
    tl: "",
    br: "󰁂",
    bl: "",
  ),
  keycap(
    ch: "H",
    tr: "",
    tl: "",
    br: "",
    bl: "",
  ),
  keycap(
    ch: ", <",
    tr: "󱕐",
    tl: "",
    br: "",
    bl: "",
  ),
  keycap(
    ch: ". >",
    tr: "󱕑",
    tl: "",
    br: "",
    bl: "",
  ),
  keycap(
    ch: "/ ?",
    tr: "",
    tl: "",
    br: "󱥒",
    bl: "",
  )
)
#stack(dir: ltr, spacing: 1em,
  keycap(
    ch: "󰌑",
    tr: "󰍽",
    tl: "",
    br: "",
    bl: "",
  ),
  keycap(
    ch: "󰁮",
    tr: "󰍽",
    tl: "",
    br: "",
    bl: "󰐎",
  ),
  keycap(
    ch: "󰭜",
    tr: "󰍽|",
    tl: "",
    br: "",
    bl: "󰖁",
  ),
)