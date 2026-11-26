# The terminal host: a first look

**Status:** a mock-up the owner drew in Claude Design on 2026-11-26 for the terminal host
(issue #8; D111 item 3, proposed, not ruled). Nothing in it is decided: the host's design
pass settles it in `docs/ENGINE.md` or a host document, with a decision. Every look is a
stand-in until the owner rules.

The idea: Loom drawn in terminal cells, and the keyboard laid out as the panel, left to
right: the grid, sliders A and B, the function keys.

| File | What it shows |
|---|---|
| `Main.dc.html` | Rhythm at 80 by 24 cells: write on, playing. A `screen` property switches it to write off or sound held |
| `Sound.dc.html` | Main with sound held: the display lists the sixteen sounds by key |
| `Pocket.dc.html` | A small terminal, 48 by 10: one light a key |
| `States.dc.html` | Every key state in cells, in truecolour and with no colour (normal, bold and reverse only); a key is 5 by 3 cells |
| `Keyboard.dc.html` | The keyboard map: the grid on `1234 qwer asdf zxcv`, sliders on `t g` and `y h`, function keys on `7 8 9` and `u i o p`; how holds work in a terminal |

**What the mock-up proposes for holds ⚠:** a tap arms a function key until the next grid key
(works everywhere); a real hold where the terminal reports key-up (the kitty keyboard
protocol); shift with the key locks it; esc lets everything go. Which of the owner's
terminals report key-up is not checked.

**How to open them:** these are Claude Design boards. They render on the owner's canvas,
not from this folder: each loads the canvas's own `support.js`. The markup and the
`renderVals()` script in each file give every colour, size and cell position. The colours
are Loom's tokens (`DESIGN.md`).
