# The design session of 2026-11-26: eight boards

**Status:** pictures of intent. Kenny and a design session drew these in Claude Design
on 2026-11-26. **None was rendered and checked by the session that drew it:** they are
drawings of what is meant, not measured pictures. What Kenny ruled from them is D112,
D113, D114 and D119 in `docs/DECISIONS.md`; the rest is proposed (D115, D116, D117). The design
text is in `../DESIGN.md`: "A key's lights", "The keys and the helper", "Loom in a
terminal" and "Layout B". Where a board and that text disagree, the text is the later
reading.

**This folder is behind the canvas.** It is a copy taken before [the design canvas](https://claude.ai/artifact/Rswd7LH4BPVAPzA5r1AbxK)
was laid out in pages: eight of its fifteen boards, and it is not refreshed. Read the canvas
for the current boards. Whether the repository keeps a copy of the boards at all is not
ruled (issue 98).

The folder is named for the terminal host, where the canvas began (issue 8; D111 item 3).
Three of the boards are about every host.

| File | What it shows | Size, px |
|---|---|---|
| `Main.dc.html` | Rhythm in a terminal at 80 by 24 cells: write on, playing. A `screen` property switches it to write off or sound held | 1200 by 760 |
| `Sound.dc.html` | `Main` with sound held: the display lists the sixteen sounds by key. It only imports `Main` | 1200 by 760 |
| `Pocket.dc.html` | A small terminal, 48 by 10: one light a key | 752 by 360 |
| `States.dc.html` | Every key state in terminal cells, in truecolour and with no colour (normal, bold and reverse only); a key is 5 by 3 cells; the function key's states | 1200 by 760 |
| `Keyboard.dc.html` | The keyboard map for every host, and how holding and the Shift latch work | 1200 by 760 |
| `Lights.dc.html` | A key's lights from the outside in, on a Rack or page key, with no colour, and in terminal cells; the source marks | 1592 by 860 |
| `Helper.dc.html` | The keys helper at phone size | 390 by 844 |
| `Panel.dc.html` | The Rack panel in layout B at 27 HP, the width Kenny ruled (D119), with the `?` in the display's corner, at 5 px a mm. Refreshed from the canvas after the ruling; the 24 HP drawing it replaced is in this file's history and its numbers are in `../DESIGN.md` | 782 by 900 |
| `canvas.json` | The canvas's own index: each board's title, size and place | |

## These do not open as pictures

`../loom/` holds Claude Design artboards "with the canvas wrapper removed": open one in a
browser and it is the picture. These eight are not that. Each is the canvas's source: a
template (`sc-for`, `sc-if`, `{{ }}`, and `dc-import` in `Sound`) filled by a script in the
file, and run by the canvas's own `support.js`, which is not ours and is not in this
repository. Opened from this folder a board shows its template unfilled.

They are kept as they are rather than flattened by hand, because a flattened copy nobody
compared with the canvas would be a second unchecked drawing. Until they are flattened,
read a board as text: the markup and the `renderVals()` script give every colour, size and
position. The colours are Loom's tokens, and `#6FD6A0` is `loom/fern`.

## What was changed from the canvas

- `Lights.dc.html` on the canvas carries a stray block of text pasted from another task. It
  is not part of the design and is left out; nothing of the drawing differs.
- The other seven files are the canvas's bytes.
- `Panel.dc.html`'s own title still reads "at 24 and 27 HP"; it draws 27 HP only.

## Known to be out of step

- `Main` (with write off) shows a firing as a whole white face, and `Pocket` draws the fx
  key's band ecru. Both were drawn before Kenny's rulings and not redrawn.
- `Panel`'s numbers are arithmetic. None has been through `make_panel.py --check`.
