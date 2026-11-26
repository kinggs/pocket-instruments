# The look: Loom

**Status:** chosen by the owner on 2026-11-19 (D36) from three directions drawn in Claude
Design to `PROMPT.md`. The phone is built (`web/rhythm.html`, Session 12), and the Rack module
(`rack/src/Rhythm.cpp`, Session 13), ~~both without the animation~~ and since Session 14 both weave,
as a first attempt the owner has not yet watched (D84). ⚠ marks a proposal the owner has not ruled on.

**Changed 2026-11-26, not built (D112 to D117):** the owner ruled that a key's lights mean
the same with write on or off, that effects light keys in a dye of their own, that the
latch is Shift, and that Rack's jacks move to one row. The sections "A key's lights", "The
keys and the helper", "Loom in a terminal" and "Layout B" say what was drawn. Older lines
they overrule are struck through, not removed. The page and the Rack module still draw the
older look.

The idea in one line: a pattern is a cloth. The grid is a tight woven swatch, the dividers are
stitches, the sliders are shuttles on a thread, and the display weaves as the pattern plays.

The owner's brief for it: dark, high contrast, easy to read with weak eyes.

## What is in this folder

| File | What it is |
|---|---|
| `PROMPT.md` | The prompt the three directions were drawn from |
| `loom/states.html` | Every key state, and the same states with the colour taken out |
| `loom/rack.html` | The Rack panel at 6 px per mm: write on, playing |
| `loom/rack-sound.html` | The same panel: write off, sound latched |
| `loom/phone.html` | The phone layout, 390 by 844 |
| `loom/frames.html` | Three frames of the display |
| `loom/tokens.html` | The colours and type, as swatches |
| `directions/` | The three directions the owner chose between. A and C were not taken |
| `tui/` | The design session of 2026-11-26, eight Claude Design boards kept as their source: the terminal host (`Main`, `Sound`, `Pocket`, `States`), the keys (`Keyboard`, `Helper`), a key's lights on every host (`Lights`) and Rack's layout B (`Panel`). **Pictures of intent:** none was rendered and checked by the session that drew it, and none opens from this folder as the `loom/` pictures do (`tui/README.md` says what each is and why) |
| `panel.svg` | The static Rack panel, flat, in mm. Made by `make_panel.py` |
| `make_panel.py` | Draws `panel.svg`, prints the widget positions, and (`--check`) holds every jack's label, plate and stitches clear of its neighbour's |
| `rack-face-unlit-100.png` | The Rhythm module as Rack draws it, from Rack's own screenshot mode at 100% zoom (360 px wide: what a player sees without zooming). Nothing lit: no module runs in that mode. Taken in Session 18, with the two jack rows; again in Session 24, with FX SEND's and FX RETURN's stitches |
| `rack-face-unlit-200.png` | The same at 200%, to judge the lettering and the plates |
| `rack-face-lit-100.png`, `rack-face-lit-200.png` | The same from a scratch build whose face was filled by hand (never shipped): playing on step 5 with write on, steps 1, 5, 7, 9, 11 and 13 lit, key 10 pressed, the stitch on key 3, the bar on key 1, fx latched (D78: its face lit like play and write), and each slider's second mark at 20000 and 50000. Taken again in Session 19 with effect 3 heard (the madder `fx 3` block above `write`) and the lane on steps 6, 7, 8 and 13 (four madder ticks over the step marks). Not taken again in Session 20: they show write on, which did not change. Taken again in Session 24 for FX SEND's and FX RETURN's stitches, with the same face (the weave's bytes left at 0, so the loom is bare) |
| `rack-face-writeoff-100.png`, `rack-face-writeoff-200.png` | The same kind of scratch build, taken in Session 20 for the owner to judge write off (D82): playing on step 5 with write off (no write mark, play lit), sound 1 chosen with steps 1, 5, 7, 9 and 13 lit in ecru and no stitch; key 5 white inside the ochre frame (a lit step firing on the playhead: it comes back lit when its half step is over), and key 3 white (another sound firing, on a key that is not lit). The whites are held on by hand for the picture |
| `rack-face-ext-200.png` | The same kind of scratch build, taken in Session 34 for the owner to judge ENGINE.md §14's measured tempo (D100 item 5, O27): on an external clock (the view's flag 2), playing on step 5 with write off, the stored tempo 116 and `ext` 1 230, so the display reads `123` with `ext` where `bpm` was, in bpm's place, size and colour (a stand-in). With `ext` at 0 (before the second edge, or a stopped clock) the number reads `--`; not pictured |
| `rack-weave-bare-200.png` | Session 14, the weave's first attempt (D84), from Rack's screenshot mode at 200%. No module runs there, so this is the real build with nothing played: the bare loom. The beater one cell wide at the left edge, five warp threads across, the shuttle resting at the top |
| `rack-weave-step07-100.png`, `-200.png` | The same from a scratch build whose face was filled by hand (never shipped), as the three below. Step 7 of a first bar, write off, a kick on the beats and a hat between: one finished diamond and most of the second (two of four steps set: ochre round ecru), the beater two cells wide (one sound on this step), the shuttle half way down trailing its thread from the top. Keys 1, 5, 9 and 13 lit, the frame on 7 |
| `rack-weave-step13-100.png`, `-200.png` | Step 13 of the same bar as two sounds fire together: three diamonds, the three newest columns of cloth white (rule 6's flash), the beater three cells wide, the shuttle just leaving the top |
| `rack-weave-busy-100.png`, `-200.png` | The third bar of a busier pattern, write on, four sounds on this step: the cloth has scrolled, so three woven beats and the one on the loom. Diamonds for four, three and one step set (madder round ochre round ecru; the same a size smaller; a small ecru one), then half of the next meeting the beater at its widest, four cells; the shuttle on its way back up, its thread below it. ⚠ The half diamond's ochre runs into the ochre beater: one of the things to judge |
| `rack-weave-stopped-200.png` | Stopped after seven steps: the cloth as it was left, the beater back to one cell, the shuttle at the top with no thread, `step --` |
| `phone-weave-step13.png` | The phone page's display at three device pixels a CSS pixel (a headless browser, the view filled by hand): the same moment as `rack-weave-step13`, to compare the two hosts' painting of the same cells |

The four `rack-face-*` pairs above were taken before the weave and show its area dark; they
were not taken again.

The `.html` files are pictures, not code to build from: open one in a browser. They are the
Claude Design artboards with the canvas wrapper removed. They load the typeface from Google
Fonts and fall back to the system monospace offline.

## Tokens

| Token | Value | Used for |
|---|---|---|
| `loom/ground` | `#14131F` | The body |
| `loom/tile` | `#2B2942` | An unlit key |
| `loom/display` | `#0B0A12` | The display glass |
| `loom/table` | `#07060C` | The phone's background; jack holes |
| `loom/ecru` | `#F2E8D0` | A lit field; all lettering |
| `loom/white` | `#FFFFFF` | A key that is sounding |
| `loom/ochre` | `#F0B429` | The playhead frame; the pattern key |
| `loom/madder` | `#F0654A` | The sound key |
| `loom/sky` | `#8FAAF7` | The bpm key |
| `loom/fern` | `#6FD6A0` | The effects (D112 item 3, 2026-11-26; a proposed value ⚠, not built): the fx key's band, the display's fx block, the lane's ticks, the twill and the fern core on a key |
| `loom/lilac` | `#C9C2DD` | Secondary lettering |
| `loom/thread` | `#6E6A92` | Unlit threads and step marks. Never lettering |

Contrast, by arithmetic and not measured ⚠: ecru on ground about 15:1, ecru on tile about
11:1, ground on ochre about 10:1, ground on sky about 8:1, ground on madder about 6:1.
Madder was lightened from the first sketch (`#D9482F`) because a dark label on it fell near
4:1.

Fern, by the same arithmetic and not measured ⚠: ground on fern about 10:1, fern on tile
about 8:1, fern on display about 11:1. Against the other lights it is close in lightness:
fern to ochre about 1.05:1, to sky about 1.3:1, to ecru about 1.5:1, to white and to madder
about 1.8:1. So fern is never told from ochre by lightness: the twill is hatched and the
core has its dark line for that reason, and ⚠ the fx key's band beside the pattern key's
differs from it by hue, place and word only, as the three top keys' bands already do.

Type is one family, JetBrains Mono, at weight 800. Why: its numerals cannot be mistaken for
each other, and one heavy weight survives small sizes.

| Token | Size on the Rack panel | Used for |
|---|---|---|
| `type/readout` | 10 mm | The tempo |
| `type/numeral` | 3.8 mm | Key numbers |
| `type/label` | 2.8 mm | Function key words, lower case |
| `type/jack` | 2.6 mm (AUDIO 2.8), tracked 0.04 em, capitals | CLOCK, RESET, TRIGGERS, VOICES, AUDIO |

On the phone nothing is under 16 px.

## The lighting

The keys are the lamps (D21): there is no separate light beside a key. ~~Each key has two
lights.~~ Since 2026-11-26 a key has six lights and a mark (D112, D115; "A key's lights"
below). As built, and as `loom/states.html` shows, it has these:

- **The field** is the whole face. Lit means there is something here.
- **The frame** is a band round the edge, an eighth of the key wide. Lit means the playhead
  is here.
- ~~**White** means this key is sounding now. Under a finger the white pulls in from the edge
  by the width of the frame; when the sequencer fires the key it stays full size.~~ (D112
  item 1: a sounding key shows a core, not a white face. White pulled in is still a finger.)

No state depends on hue. Every state differs from every other by shape or by lightness, and
`loom/states.html` shows them in greyscale to prove it.

### A key's lights, from the outside in (D112 decided; D115 proposed; 2026-11-26; not built)

`tui/Lights.dc.html` draws them on a Rack or page key, with no colour, and in terminal
cells. ⚠ Every size below is read off that board, which nobody has seen rendered.

| Light | Where | Lit means | Colour |
|---|---|---|---|
| frame | The outer band, an eighth of the key | The playhead is on this step | ochre |
| twill | A band inside the frame, as wide again, hatched at 45°: fern stripes and ground stripes | Effect *n* is heard (D112 item 2) | fern and ground |
| field | The face inside the bands | The chosen sound has a step here | ecru |
| core | A square in the middle, about two fifths of the key, with a dark line round it | Sound *n* is sounding (D112 item 1) | white; fern while that sound goes through the effect being heard |
| stitch | Top right, inside the frame | Another sound has a step here | ecru on dark, dark on ecru |
| bar | Along the bottom, inside the frame | The chosen sound or pattern | the held key's colour |
| source mark | Top left, inside the frame | Where it comes from (below) | ecru |

- **Pressed** stays white, pulled in from the edge by the frame's width.
- **From the outside in** the lights answer: where the bar is, what is done to the sound,
  what is written, what is sounding.
- **The twill is hatched** so that it reads the same on a lit key and on a dark one. Where
  no effect is heard the band is the face: lit with the field, dark without.
- **No state is told by hue alone.** The twill is told by its hatching and the core by its
  dark line; the board's second row shows every state with the colour taken out.
- **Every light means the same with write on or off** (D112 item 1). Write changes what a
  tap does and the display's write mark.
- On a Rack key of 11.7 mm, by the board's proportions ⚠: frame 1.46 mm, twill 1.46 mm,
  field 5.85 mm, core 4.6 mm with a line of about 0.37 mm.

**The source mark (D112 item 4).** Shown only while sound or fx is held or latched.

| | No mark | A short dash | A ring | A solid disc |
|---|---|---|---|---|
| sound held | built in code | a sample | back in from a cable (RETURN) | |
| fx held | ours | | Rack's beside ours | Rack's in its place |

⚠ Left open by the board, for the build or the owner:

- The stitch and the source mark sit over the twill's band. In terminal cells the stitch is
  not drawn while the twill is (`Lights`' own rule), so there a heard effect hides it.
- When the bar shows is not redrawn: as built, only while sound or pattern is held or
  latched (the next section). The ruling's words would let it show always.
- The page has no cable and no Rack effect, so only the dash can show there.
- What the view carries for the core, the twill and the marks: O32.

## Key states

Grid key:

| State | Field | Frame | Extra | When |
|---|---|---|---|---|
| off | dark | dark | pale numeral | |
| step | ecru | dark | dark numeral | The chosen sound has a step here |
| playing | dark | ochre | | The playhead is on this step |
| step + playing | ecru | ochre | a dark line between the two | |
| pressed | white, pulled in | ground | | A finger or the pointer is down |
| other sound | as it was | as it was | one stitch in the top right corner, ecru on dark, dark on ecru | Another sound has a step here |
| chosen | as it was | as it was | a bar along the bottom, in the held key's colour | The selected sound, or pattern |
| ~~fired~~ | ~~white, full size~~ | ~~as it was~~ | | ~~This key's sound has just played~~ (D112 item 1: "sounding" below) |
| sounding | as it was | as it was | a core in the middle, white, a dark line round it | This key's sound is sounding, write on or off (D112; not built) |
| sounding, through the effect | as it was | as it was | the same core, fern | And the effect being heard takes this sound (D112; not built) |
| effect heard | as it was | as it was | the twill band inside the frame | Effect *n* is heard, punched or replayed (D112 item 2; not built) |
| queued | dark | broken ecru frame | | This pattern plays next. Chaining is "later" in `WORKFLOW.md` |

Function key:

| State | Looks like |
|---|---|
| idle | Dark, with a band across the top: madder, ochre or sky for sound, pattern and bpm, ecru for the other four (fx: fern, D112 item 3; not built) |
| pressed | White, pulled in, no band |
| latched | The whole face lit: in its own colour for the top three, ecru for the rest. Dark label |

Play is latched while the sequencer runs. Write is latched while write mode is on. ~~In Rack a
click latches any function key until the next grid press or a second click (D23).~~ With a
mouse, in Rack and on the page, a click is a real hold and ~~Ctrl+click (Cmd on a Mac)~~
latches sound, pattern, bpm, function or fx until a press of any key ends (D60, D61). Since
Session 19 fx keeps its latch until it is ~~Ctrl+clicked~~ again, and is down only while a
grid key is (D78), on the page as in Rack.

**The latch is Shift (D113, 2026-11-26; not built):** Shift+click and Shift+key, on the
page, in Rack and in a terminal. What a latch does is as above. Until it is built both
hosts still latch by Ctrl+click.

### ~~What the grid shows, by mode~~ What the grid shows (D37, accepted by D38; write off changed by D71, D72 and D82; the by-mode reading ended by D112)

**Since 2026-11-26 (D112 item 1; not built) there is one column.** The table below it is
what is built today.

| | Write on or off |
|---|---|
| Field | the chosen sound's steps |
| Stitch | other sounds' steps ⚠ (with write off too: D112's words; D82 item 3 had none there, and the owner was not asked about the stitch alone) |
| Frame | the playhead |
| Core | each key's own sound sounding |
| Twill | effect *n* heard |
| White, pulled in | a finger |
| Bar | the chosen sound, while sound is held or latched |

As built:

| | Write on | Write off |
|---|---|---|
| The grid is | sixteen steps | sixteen sounds |
| Field | the chosen sound's steps | ~~off~~ the chosen sound's steps (since Session 20) |
| Stitch | other sounds' steps | off |
| Frame | the playhead | the playhead |
| White | a finger | a finger, or the key's sound firing |
| Bar | off | the chosen sound, while sound is held or latched |

**Write off, changed 2026-11-23 (built in Session 20).** D37 had the fields dark with write
off, from what a search said of the hardware (R26, R27). The owner's PO-12 lights the
chosen sound's steps in both modes (D71), so the field now does too, and write changes what
a tap does and the display's write mark, not the grid's fields. With write off a lit key
whose sound fires goes white for half a step and comes back lit ("step" then "fired" then
"step" in the key states above); the frame still marks the playhead; there is no stitch
(D82 item 3: the stitch is for editing). `rack-face-writeoff-100.png` and `-200.png` show
it.

While a function key is held the grid changes meaning, in either mode (D51; the core
decides it, `ENGINE.md` §14): **sound held**, the grid is sounds and the bar marks the chosen
one, fields off; **bpm held**, the fields of keys 1 to *n* are lit, *n* being the master
volume; **pattern held**, the bar is on key 1. The frame runs throughout.

This follows what a search found about the hardware (`WORKFLOW.md`, question 3: R26, R27),
not a check on it. D30 gives all sixteen keys a track, so every key can fire.

The held-key readings above are not about write, and D112 leaves them. The source marks
(D112 item 4) join them: shown while sound or fx is held or latched.

## The display

Left to right: the weave, then a column with the tempo, `bpm`, the write mark and the step
number. A row of sixteen step marks runs under the weave: the four beats stand taller, the
playing step is the tallest and is ochre.

The write mark is a solid ochre block with the word `write` in it. It is absent when write
is off, so its presence does not depend on colour.

### The effects on the display (D81; a stand-in for the owner's eye, ⚠)

Drawn in Session 19 on the page and in Rack, from the same view bytes. ~~No key lights for an
effect (D66 item 3)~~ (superseded by D112 item 2: effect *n* heard lights key *n*'s twill),
and there is no picture of a machine and no spark (D81 item 3).

**The colour, changed 2026-11-26 (D112 item 3; not built):** the block and the ticks are
fern, the effects' own dye, where "madder" stands below. Built, they are still madder.

- **The effect heard** (byte 55): a solid madder block the size of the write mark, with `fx`
  and the number in it in ground (`fx 3`, `fx 11`). It sits just above the write mark's place
  in the column and keeps that place whether write is on or off; absent when nothing is
  heard, so, like the write mark, its presence does not depend on colour. Rack: 11 by 3.6 mm,
  0.8 mm above the write mark. Phone: the write mark's width (five characters and 8 px each
  side), directly above it.
- **The lane** (bytes 56 and 57): a madder tick over the step mark of each step that holds an
  effect, the mark's width, on one line clear of the tallest mark, so the marks themselves do
  not change. Rack: 0.67 mm tall, 0.6 mm above the tallest mark. Phone: 4 px tall, 4 px above.
- Neither moves, so reduced motion changes nothing (D81 item 4).

### The animation ⚠ (~~proposed, not built~~ built in Session 14 as a first attempt, D84; not accepted)

**Where it stands (2026-11-23):** the seven rules below are drawn, once, in the core
(`core/src/weave.rs`; `ENGINE.md` §16), and the page and the Rack display paint the same
cells. The owner has not watched it move. Until they have, nothing below "How each rule is
drawn" is settled, and no frame is kept as the picture to hold to.

The subject is a cloth being woven. Flat colour on a 1 mm weave grid: no gradients.

The owner keeps weaving as the subject (O12, answered 2026-11-19). The original's protected
display art shows a sewing machine and thread spools (R6): a different drawing, a
neighbouring subject, so it still goes to O7 and D19 at P9. If it has to move then, the rules
below are about a beat, a density and a loudness and carry over; the shuttle and thread are
the parts that would go.

1. **A diamond is a beat.** Four steps weave one diamond, a quarter of it per step. Four
   diamonds are a bar.
2. **The cloth grows to the right.** The bright bar at its edge is the beater. Once three
   diamonds are woven the cloth scrolls left one diamond a beat, so the edge stays in view.
3. **The diamond's size is the beat's density.** A beat with no steps set weaves only the
   centre stitch; one with all four set weaves the full diamond.
4. **The beater's width is the loudness of this step,** from 1 mm on a silent step to 4 mm
   when everything fires.
5. **The shuttle crosses once a step,** top to bottom on odd steps and back on even ones,
   trailing one pale thread.
6. **When two or more sounds fire together** the three newest rows flash white for a
   sixteenth of a step. (Built as a quarter of a step ⚠, D84 item 11: a sixteenth is 7.8 ms
   at 120 bpm, less than one frame of a screen.)
7. **Stopped,** the cloth holds still and the shuttle rests at the top.

With reduced motion asked for, rule 6 is off and the shuttle jumps instead of travelling.

`loom/frames.html` shows steps 1, 7 and 13.

#### How each rule is drawn ⚠ (Session 14; every line is D84's, a first attempt)

The weave's area is 78 by 15 cells of 1 mm. Left to right: the cloth, the beater at its
edge, then the bare warp, which the shuttle crosses. Seven palette numbers: display (not
painted), thread, ecru, white, ochre, tile, madder. Each host paints a cell as a flat square
five sixths of its place, so every part reads as woven; `frames.html` has that grid on the
cloth only, and thin warp lines a cell cannot draw.

| Rule | Drawn as |
|---|---|
| 1, a diamond is a beat | A beat is 16 columns of tile-coloured cloth, 4 a step, with a diamond centred in it: up to 15 cells wide and tall, so a cell of plain cloth parts it from the next |
| 2, the cloth grows to the right | Four beats show: three woven and the one on the loom. From the fifth beat the cloth jumps left a whole beat as each new beat starts, so the edge moves between a quarter and four quarters of the last place and never leaves the area |
| 3, size is density | The diamond reaches 0, 2, 4, 6 or 7 cells from its middle for 0 to 4 of the beat's steps with any sound set: a single ecru stitch; a small ecru diamond; ochre round ecru; madder round both; the same to the full height. The beat on the loom is drawn from its steps so far, so it can grow as it is woven |
| 4, the beater | An ochre bar the whole height at the cloth's edge: 1 cell with nothing fired on this step, 2 for one sound, 3 for two, 4 for three or more. Taps count |
| 5, the shuttle | An ecru block 3 by 4 cells, its middle 7 cells right of the cloth's edge, crossing in one step: down on the first, third, fifth step since play, back up on the others, with a thread one cell wide behind it |
| 6, the flash | The three columns of cloth next to the beater go white, the whole height, for a quarter of a step from the tick the step's sounds fire (so a swung step flashes late, with its sound) |
| 7, stopped | Nothing reads the clock: the cloth as it was left, the beater 1 cell, the shuttle at the top with no thread. Before any play, a bare loom: beater, warp, shuttle |
| Reduced motion | No flash. The shuttle is at the far side for the whole step with its thread the whole height, so it jumps once a step. On the page from `prefers-reduced-motion`; in Rack from the menu's "Display: reduced motion" |

The warp is five thread-coloured rows from the cloth's edge to the right. Play starts a new
cloth, and so does a reset while playing.

**What it must stay clear of (R6, D20):** the original's display shows a sewing machine and
thread spools. Nothing here is a machine, a needle or a spool: the parts are a loom's (warp,
cloth, beater, shuttle), drawn as cells. ⚠ The shuttle is a block crossing bare warp with a
thread behind it; whether that could be read as a needle and thread is for the owner's eye
and then O7.

## The Rack panel

24 HP: 121.92 mm wide, 128.5 mm tall. `panel.svg` holds only what never changes: the body,
the display's frame, the two stitched dividers, the slider threads, A and B, the jack labels,
the pale plate under each output with its marks, and the wordmark.

Everything else is a widget the module draws: the display, sixteen keys, seven function keys,
two slider caps, thirteen sockets (six until Session 18). Why: they change, and Rack's SVG reader draws no text, no
dashes and no filters. For the same reason `make_panel.py` turns lettering into outlines and
draws each stitch as its own rectangle.

Positions in mm from the top-left corner (`uv run pocket-instruments/design/make_panel.py --positions`):

| Widget | Centre x | Centre y | Width | Height |
|---|---|---|---|---|
| display (inside the frame) | 61.00 | 19.50 | 111.30 | 24.30 |
| key *n*, column *c* and row *r* from 0 | 11 + 12*c* | 44 + 12*r* | 11.70 | 11.70 |
| slider A (travel) | 60.00 | 59.50 | 5.00 | 43.00 |
| slider B (travel) | 70.00 | 59.50 | 5.00 | 43.00 |
| sound | 83.47 | 49.85 | 12.93 | 23.70 |
| pattern | 97.00 | 49.85 | 12.93 | 23.70 |
| bpm | 110.53 | 49.85 | 12.93 | 23.70 |
| function | 81.78 | 74.15 | 9.55 | 23.70 |
| fx | 91.93 | 74.15 | 9.55 | 23.70 |
| play | 102.08 | 74.15 | 9.55 | 23.70 |
| write | 112.23 | 74.15 | 9.55 | 23.70 |
| jack CLOCK (in) | 12.40 | 97.50 | | |
| jack RESET (in) | 25.40 | 97.50 | | |
| jack A OUT (out, 16 channels) | 40.00 | 97.50 | | |
| jack A IN (in, 16 channels) | 51.67 | 97.50 | | |
| jack B OUT (out, 16 channels) | 63.33 | 97.50 | | |
| jack B IN (in, 16 channels) | 75.00 | 97.50 | | |
| jack FX GATES (out, 16 channels) | 90.00 | 97.50 | | |
| jack TRIGGERS (out, 16 channels) | 12.40 | 116.30 | | |
| jack VOICES (out, 16 channels) | 25.40 | 116.30 | | |
| jack RETURN (in, 16 channels) | 38.40 | 116.30 | | |
| jack FX SEND (out, 16 channels) | 84.00 | 116.30 | | |
| jack FX RETURN (in, 16 channels) | 96.00 | 116.30 | | |
| jack AUDIO (out) | 109.67 | 116.30 | | |

Until Session 18 there were six, in one row at y 116.30: CLOCK 10.67, RESET 22.33, RETURN
34.00, TRIGGERS 80.67, VOICES 95.17, AUDIO 109.67. Every one but AUDIO moved (D76 item 2);
a patch keeps its cables, because Rack ties a cable to a port's number, and the numbers
did not change.

### The jack rows

Two rows, thirteen jacks, grouped by what they carry (D76, the owner: "inputs and outputs
for the same kinds of things close together"). Drawn in Session 18, and "accepted for now":
judged in use, with cables in.

```
upper row, y 97.5 (voltages that steer):
  CLOCK  RESET  |  A OUT  A IN  B OUT  B IN  |  FX GATES
lower row, y 116.3 (triggers and sound):
  TRIGGERS  VOICES  RETURN  |  wordmark  |  FX SEND  FX RETURN  |  AUDIO
```

| Jack | Carries |
|---|---|
| CLOCK, RESET (in) | one edge a step; reset to step 1 |
| A OUT, B OUT (out, 16 channels) | 0 to 10 V: the slider each sound was last hit with, by the instrument's own reckoning (the hand's, or what the step keeps) |
| A IN, B IN (in, 16 channels) | 0 to 10 V: channel *n*'s voltage at sound *n*'s hit takes that slider's place. Which sounds: the channels the cable carries, or the menu's ticks |
| FX GATES (out, 16 channels) | 10 V on channel *n* while effect *n* is heard, punched or replayed |
| TRIGGERS (out, 16 channels) | 10 V for 1 ms as each sound fires |
| VOICES (out, 16 channels) | each sound alone, 5 V to 1.0, before the effects and the volume |
| RETURN (in, 16 channels) | channel *n* takes sound *n*'s place in the mix and, if that sound is switched in, in the effects |
| FX SEND (out, 16 channels) | a channel an effect: the sounds switched into the effects, before any effect and before the volume, ramped in over 2 ms on channel *n* while effect *n* is heard and out after (D88; until Session 24, one channel, always the bus) |
| FX RETURN (in, 16 channels) | every channel summed into the mix, before the volume, playing or stopped. Each effect is Ours, Rack's beside the dry or Rack's in place of the dry, by the menu (D88; until Session 24, one channel, heard in place of the effects ticked in the menu) |
| AUDIO (out) | the mix |

- **Each group reads as it is patched.** The sounds' loop left to right: TRIGGERS out,
  VOICES out, RETURN in. Each slider a pair: out, then in. The effects one column group at
  the right: FX GATES over FX SEND and FX RETURN. The clock top left, where a patch starts;
  AUDIO bottom right, where it ends.
- **An output is told by its plate, not by its side:** each has its own patch of the pale
  ground with dark lettering, as wide as its stitches or its label (R33 asks for an
  inverted background on outputs). TRIGGERS' and VOICES' meet and read as one. An input
  sits on the dark body with pale lettering.
- **Sixteen channels:** ten of the thirteen have sixteen stitches round the socket, one
  for each key, dark on a plate and pale on the body. CLOCK, RESET and AUDIO have none.
  (FX SEND and FX RETURN gained theirs in Session 24, D87: nothing moved, and `--check`
  holds FX SEND to FX RETURN at 1.20 mm and FX RETURN to AUDIO at 1.57 mm.)
- **AUDIO is the main output.** It alone has a solid frame inside its plate, and its label
  is a size larger.
- **FX SEND and FX RETURN have FX on a line of its own** over the word. Why: on one line,
  at 2.6 mm, the two labels are 11.5 and 14.8 mm wide and overlapped.
- **What moved from D76's proposal, and why** (D80 items 16 to 18): the left three columns
  from 10.67, 22.33 and 34.00 to 12.4, 25.4 and 38.4 (TRIGGERS' lettering touched
  VOICES'); FX SEND and FX RETURN from 80.67 and 92.33 to 84.0 and 96.0, and FX GATES from
  86.5 to 90.0 to stay over them; the rest as proposed. `make_panel.py --check` prints
  every clearance: the tightest are 0.87 mm from a slider output's plate to the next
  input's stitches, and 1.2 mm each side of the wordmark, which keeps its 3.0 mm.
- The single row it replaced (Session 13): three inputs on the dark body at the left, the
  three outputs on one plate at the right, the wordmark between at 0.5 mm clear. RETURN was
  drawn then with sixteen pale stitches, as the two sixteen-channel outputs had dark ones,
  and by D62 item 4 the wordmark dropped from 3.5 to 3.0 mm: at 3.5 mm it is 37.8 mm wide
  and the room was 33.4 mm. A first
  draft of that had a stitched box round the outputs with a plate under AUDIO only; it left
  two outputs on the dark body, against R33, so it went.

### Layout B: one row of jacks (D114, 2026-11-26; decided, not drawn by `make_panel.py`, not built)

The owner ruled the layout; the width is not ruled (O34) and the labels are not (O35).
`tui/Panel.dc.html` draws it at both widths. ⚠ Every number here is the design session's
arithmetic from `make_panel.py`'s own clearances. **None of it has been through
`make_panel.py --check`,** and the board was not seen rendered.

```
CLOCK  |                                   |
       |            the display            |  AUDIO
RESET  |                                   |

            keys      sliders      function keys

 SOUNDS            SLIDER A     SLIDER B     FX
 TRIG  OUT  IN     OUT  IN      OUT  IN      GATES  SEND  RETURN
```

- **CLOCK over RESET** at the left of the display, x 9.5 mm, y 13.5 and 27.5. **AUDIO** at
  its right, 9.5 mm in from the edge, y 21. The display runs from 17 mm to 17 mm short of
  the right edge and keeps its height.
- **Ten sixteen-channel jacks in one row,** y 116.3 as the lower row is now, in four
  groups. A thread-coloured rule runs over each group with the group's name above it.
- **The keys, the sliders and the function keys** keep their places among themselves and
  sit 6 mm lower; at 27 HP they are centred, 7.62 mm further right.
- **The pitch is our own,** not VCV's: `docs/RACK-RULES.md` says the manual gives no
  numbers. Stitches reach 5.2 mm from a socket's centre and a plate 5.6 mm; with 0.5 mm
  clear that is 11.3 mm from an output to an input and 11.2 mm between two outputs (two
  outputs may share a plate).

| | 24 HP (121.92 mm, as now) | 27 HP (137.16 mm) |
|---|---|---|
| Extra between groups | 1.87 mm | 6.95 mm |
| From the first and last jack's edge to the panel's | 2.0 mm | 2.0 mm |
| Jack centres, x | 7.60, 18.80, 30.10, 43.27, 54.57, 67.74, 79.04, 92.21, 103.41, 114.71 | 7.60, 18.80, 30.10, 48.35, 59.65, 77.90, 89.20, 107.45, 118.65, 129.95 |
| The display's width | about 88 mm (111.3 now) | about 103 mm |
| The weave | Cannot keep its 78 cells. The board draws 62 and says three beats: a change in the core (`ENGINE.md` §16) | Keeps 78 cells; the core is untouched |
| The tempo's column | about 20 mm (about 30 now) | about 20 mm |

The design session recommends 27 HP. Not chosen: the ten jacks in a column down the right
(about 29 HP, and cables hang over the labels below: R36).

**The labels, not ruled (O35).** The board puts a group's name over a short word: SOUNDS
over TRIG, OUT, IN; SLIDER A over OUT, IN; SLIDER B over OUT, IN; FX over GATES, SEND,
RETURN. That renames TRIGGERS, VOICES and RETURN on the panel. Port numbers in code do not
change either way (`docs/RACK-RULES.md`).

⚠ Not drawn on the board, for the redraw to settle: where the wordmark goes (it has no
place in a full bottom row; the band between the keys and the group names is free); whether
CLOCK's label, at about 5.4 mm down, clears the top left screw; the stitched dividers; the
plates' heights with one line of label. The fx key's band is drawn fern.

Also to draw as widgets:

- **Slider cap:** 5 by 3 mm, ecru, with a dark slot across the middle. Since Session 21
  (D75) the cap is the hand's: it moves by itself only while bpm is held or after a load.
- ⚠ **The second mark** (D75 item 4; `ENGINE.md` §20): not designed yet. Built in Session 21
  as PLAN.md's stand-in, below in "Not done".
- **Function key label:** runs up the key, read from the right.
- **Socket:** an ecru ring 8 mm across round a dark hole for the inputs; a dark ring round a
  pale hole for the outputs.
- **Screws:** Rack's own, at the four corners. The display starts 7 mm down to clear them.

~~⚠ `panel.svg` was outlined from the Bold cut (700), the only one installed. The design asks
for 800. Pass an ExtraBold file to `make_panel.py` to redo it.~~ Since Session 13 the
ExtraBold cut is in `fonts/` with its licence (OFL), `make_panel.py` outlines from it, and the
Rack module draws its lettering in the same file.

## What VCV's manual asks, and where this stands

The rules themselves are in `../RACK-RULES.md`. This is Loom against them.

| The manual asks | Here |
|---|---|
| mm units; 128.5 mm tall; a multiple of 5.08 mm wide | Yes: 24 HP |
| All text as paths | Yes: `make_panel.py` outlines it |
| No complex gradients, no CSS | Yes: flat fills only |
| Component graphics off the panel, on a hidden `components` layer | Partly: the panel is static only, but it has no `components` layer. The positions table above stands in; the builder adds the layer if Rack's helper script is used |
| An inverted background for output ports | Yes: each output on its own plate (Session 18; one plate under all three before) |
| Room for thumbs between knobs and ports | Sockets are 11.67 mm apart or more (12, 13 and 13.67 in the lower row); the two rows are 18.8 mm apart; keys are at 12 mm; the sliders are 10 mm apart (they were 7; the owner widened them, and the dividers closed in to make the room) |
| Text readable at 100% on an ordinary monitor | ⚠ Jack labels were raised to 2.6 mm for this, about 5.6 px of capital height at 100%. Read at that size in Rack's own screenshot in Session 18 (`rack-face-unlit-100.png`): every label reads; TRIGGERS and VOICES, 1.5 mm apart, are the closest. The owner's eye has not judged it |
| Roughly the density and text sizes of VCV's own modules | ⚠ Not compared against one |
| Do not use others' intellectual property | D20's avoid list; O7 at P9 |
| A dark panel may be offered | Not needed: it is already dark |

For the build, not the look: labels in code leave out the words input and output (R35); the
module takes the `Polyphonic` tag; triggers are 10 V for 1 ms; RESET masks CLOCK for 1 ms;
audio is about ±5 V (R34).

## The phone

Portrait, 390 by 844, one instrument filling the screen (this answers O9). A stitched hem
round the body on a darker table. Top to bottom: the display, the grid, then the sliders and
the function keys side by side.

- Grid keys are 88 px square with a 2 px gap: the grid is as wide as the body.
- Both sliders sit on the left, 44 px wide over the full height of the bottom block. The
  function keys sit on the right: the top three 81 px wide, the bottom four 60 px, all
  151 px tall.

## The keys and the helper (D113 decided; D116 proposed; 2026-11-26; not built)

One map on every host: the page, Rack and a terminal. `tui/Keyboard.dc.html` draws it on a
keyboard and `tui/Helper.dc.html` draws the helper at phone size.

| Keys | Do | Stands |
|---|---|---|
| `1234` / `qwer` / `asdf` / `zxcv` | the grid, keys 1 to 16 | built (D28) |
| `7` `8` `9` | sound, pattern, bpm | built |
| `u` `i` `o` `p` | function, fx, play, write | built |
| Shift + a function key, or Shift + click | latch it (D113) | decided, not built; Ctrl+click until then |
| `t` `g` | slider A up, down | proposed ⚠ |
| `y` `h` | slider B up, down | proposed ⚠ |
| `space` | play, stop | proposed ⚠ |
| `,` `.` | tempo down, up | proposed ⚠ |
| `<` `>` | swing less, more | proposed ⚠ |
| `-` `=` | volume down, up | proposed ⚠ |
| `m` | the sounds sheet | proposed ⚠ |
| `?` | the keys helper | proposed ⚠ |
| `esc` | lets every latch go | proposed ⚠ |
| `:` | in a terminal: a command line (save, open, export, quit) | proposed ⚠ |
| Ctrl+Q | in a terminal: quit | proposed ⚠ |

The keyboard reads left to right as the panel does: the grid, the sliders, the function
keys. The proposed keys are not ruled one by one (O36).

**The helper** is one sheet with three parts: the map; "two keys at once" (hold the first
and press the second, or Shift to latch; fx stays latched until Shift-pressed again, the
others let go after the next key; `esc` lets everything go); and the one-key shortcuts. It
opens by `?` or by a tap. ⚠ Where the tap's target sits on the page and on the Rack panel
is not designed; suggested, a `?` in the display's corner. Drawn at 390 by 844 with nothing
under 16 px.

⚠ Not checked, for the build:

- Each proposed key against what Rack and a browser already do with it.
- `?`, `<` and `>` are Shift with `/`, `,` and `.` on a US layout, while Shift is the
  latch. They are not function keys, so nothing collides on that layout; other layouts were
  not looked at. The page reads physical keys (`CODES`), so it does not care.
- In a terminal without the kitty keyboard protocol Shift+`7` arrives as a character
  (`&` on a US layout), not as Shift and `7`.
- The step a slider moves for one press of `t`, `g`, `y` or `h` is not drawn.

## Loom in a terminal (D115, D117; proposed 2026-11-26; no terminal host exists)

A terminal host is part of the showpiece (D111 item 3, ruled by D118 item 3; issue 8) and
is not designed or built. These are its
first drawings: `tui/Main.dc.html`, `Sound`, `Pocket` and `States`. The colours are the
tokens above, as truecolour.

**The screen, 80 by 24, fits exactly:**

| Rows | What |
|---|---|
| 1 | The status line: `rhythm`, the pattern; then a block for a held function key (`sound`, madder), the write mark (ochre), the step, the tempo |
| 8 | The weave: its 78 by 15 cells in half blocks |
| 1 | The lane's ticks, fern, one over each step that holds an effect |
| 1 | The sixteen step marks: the beats taller, the playing step tallest and ochre |
| 12 | Left to right as the panel: the grid (four rows of keys), sliders A and B with `t` `g` and `y` `h` at their ends, the function keys in two rows |
| 1 | A footer of hints that changes with the mode (write on, write off, a key held) |

While a function key is held the display's ten rows list what each key now means. Sound
held: the sixteen sounds, each beside the keyboard key that chooses it, the chosen one
marked in madder.

**A key is 5 by 3 cells** (`States`, `Lights`):

```
▄▄▄▄▄      the frame, as lower halves
▌ q▪▐      half-width sides; the field is this row's background
▀▀▀▀▀      upper halves; the bar goes here
```

- The core is the centre cell; the twill is the two cells either side of it; the stitch is
  a small block right of the legend.
- **The legend is the keyboard key you press** (`q`, not `5`).
- **One departure from Loom:** a cell holds two colours, so the dark line between a lit
  field and the frame is gone. The frame is told from the field by lightness.
- **With no colour (`NO_COLOR`):** normal, bold and reverse only.
- **A function key:** idle, its band along the top and its key letter in lilac; pressed,
  white with a ground edge; held, the whole face in its own colour; latched, the same with
  the key letter reversed, so a latch is told from a hold; play lit while running.

**The pocket size, 48 by 10,** for a small pane: the status line, the step marks, the grid
at one row a key with one light a key (the field, the frame as the two side cells, the
stitch), the sliders one cell wide, the seven function keys as a list, a footer.

**Holds (D117 item 4).** The kitty keyboard protocol reports key-up; a plain terminal does
not, so there a plain tap latches. How an effect is played without key-up is O33.

⚠ What the boards do not yet agree on, or leave out:

- `Main` and `Pocket` were drawn before the owner's rulings and were not redrawn: `Main`
  with write off shows a firing as a whole white face, not a core, and `Pocket` draws the
  fx key's band ecru. `States` and `Lights` are the later reading.
- `Pocket` has no core, no twill and no bar.
- Nothing was tried in a terminal: only foot 1.28 was found on the owner's machine, and it
  is believed, not tested, to support the protocol; tmux probably does not pass key-up
  through; sound out (PipeWire) was not looked at.

## Where this departs from the prompt

| Departure | Why |
|---|---|
| Function keys at 10 mm pitch on the Rack panel, not 12 | Four across at 12 mm do not fit beside a 48 mm grid and two sliders in 24 HP |
| Function keys are tall, with the label running up the key | A label under a 10 mm key can only be about 1.2 mm high. On the key it is 2.8 mm |
| On the phone both sliders are on the left | One at each edge left the bottom four keys under 60 px |
| The wordmark moved 14 mm left when the outputs became three | The brief said nothing else moves; three sockets with readable labels do not fit beside it where it was |
| The wordmark moved right again and down to 3.0 mm for RETURN | D62 item 4: an input sits on the dark body, beside RESET, and only 33.4 mm is left |
| Four grid states the prompt did not ask for: other sound, chosen, fired, queued | The owner kept the first, second and fourth (2026-11-19). Fired came from `WORKFLOW.md` afterwards ⚠ |

## Not done

- ~~The keys, slider caps, sockets and display are not drawn as Rack widgets or web
  components.~~ On the phone they are (Session 12, `web/rhythm.html`, plain CSS); in Rack
  too (Session 13, `rack/src/Rhythm.cpp`, NanoVG). Nobody has looked at them in Rack yet:
  the build machine ran Rack headless only.
- In Rack, readings the design did not settle (Session 13, ⚠), each measured off
  `loom/rack.html` at 6 px a mm or the page: the frame an eighth of a key (1.46 mm); the
  dark line between a lit field and the frame 0.66 mm; the stitch 1.83 mm square, 1.33 mm
  in from the corner; the bar 1.6 mm tall; a function key's band 1.67 mm; the cap's slot
  0.7 mm; a socket's hole 3.5 mm across. The display's sizes are the picture's: tempo 10 mm,
  `bpm` 2.5 mm in lilac, the write mark and the step number 2.67 mm. A Ctrl+click latch
  lights the face in the key's own colour, as on the page. In a self-test the weave's area
  shows the fixture's name, its count and a square, green while every pass has matched
  (green is not a Loom token: it marks a test, not the instrument). The screws are Rack's
  own (`ThemedScrew`).
- On the phone, readings the design did not settle (Session 12, ⚠): a slider is dragged
  from where its cap shows, so a touch alone moves nothing; the bar is ochre while pattern
  is held and madder otherwise; a key whose sound fires stays white with reduced motion
  asked for, because it is a state and not movement; the sounds sheet's switch is lit
  as a step is lit (ecru face, dark letters) when it plays the sample, and a name goes white
  when its sound fires. A Ctrl+click latch (D61) lights the whole face in the key's colour,
  as `states.html`'s "latched". Below 720 px tall the
  function keys' words drop from 18 to 16 px to fit up the key.
- ⚠ The slider's second mark (D75 item 4: "a second, thinner mark at the value the chosen
  sound was last hit with", the view's bytes 60 to 63) is not designed. Session 21 drew
  PLAN.md's stand-in on both hosts: a thin line in `loom/thread` across the slider's track
  where the cap's middle would be at that value, drawn over the cap and always shown, bpm
  held or not. In Rack 6 mm by 0.35 mm (the cap's 5 mm and half a millimetre each side); on
  the phone 36 by 2 px across the 6 px track. Seen in Rack's screenshot mode with a face
  filled by hand (A at 20000, B at 50000, the caps in the middle): visible on the ecru
  thread and on the cap. To be drawn with the owner's eye (Surge XT's way of showing a
  modulated value is the idea, D40; none of theirs).
  **Changed 2026-11-24, by the owner:** the line is `loom/madder` on both hosts, same size.
  In `loom/thread` "it really just doesn't stand out". Still a stand-in for the shape.
- ⚠ The two jack rows (Session 18) are drawn and seen only in Rack's screenshot mode,
  never with cables in. D76 says the grouping is judged in use. For the owner's eye: the
  plates of three heights in the lower row (TRIGGERS and VOICES one line of label, FX SEND
  and AUDIO two); FX over SEND and over RETURN; the empty corner right of FX GATES; whether
  0.87 mm between a slider output's plate and the next input's stitches is enough.
- ⚠ The display's effect block and the lane's ticks (Session 19, "The effects on the
  display" above) are D81's stand-in, built on both hosts and not yet seen by the owner. The
  readings built on: the block sits just above the write mark's place, keeping it whether or
  not write is on ("beside the write mark in the display's column"); the ticks sit on one
  line above the tallest mark rather than on each mark's own top; the sizes are the
  builder's. ⚠ **Madder:** D81 calls madder "the fx key's own colour", but the fx key's band
  and latched face are ecru here (madder is the sound key's). Built in madder, as D81 names
  it twice; the owner's to rule. Seen on the phone in a headless screenshot and in Rack's
  screenshot mode (`rack-face-lit-*.png`), never on a real screen.
- ⚠ The sounds sheet's "through fx" switch (D69, Session 19): a third button in each row,
  120 px wide, always reading `through fx`, lit as the sample switch is (ecru face, dark
  letters) while the sound goes through the effects. A tap sends `FXSOUND` and plays nothing
  (D60 item 4's "and plays the result" is the sample switch's; D69 is silent). On a 360 px
  screen the longest names wrap to two lines inside the 64 px row.
- ~~The animation is seven rules and three frames, not a running thing. Nobody has seen it move.~~
  ⚠ Since Session 14 it runs, on the page and in Rack, as a first attempt (D84, O19): seen
  moving only by `make play`'s checks, and as still pictures (`rack-weave-*.png`,
  `phone-weave-step13.png`). **The owner has not watched it.** Not done: the acceptance, and
  the three views and their cells to keep as the picture to hold to (`PLAN.md`, Session 14's
  fourth box). The drawing choices least sure, for the owner's eye: a diamond growing after
  its first columns are woven (D84 item 8); the cloth jumping a whole beat (item 9); the
  flash's quarter step (item 11); every cell a square with a gap, the beater and shuttle too
  (item 14); the half diamond's ochre meeting the ochre beater (`rack-weave-busy-200.png`);
  five warp threads where the picture has nine; on the phone the weave is 15 cells tall in a
  place about 26 tall, so it sits with dark above and below it.
- In Rack the weave is painted every frame Rack draws (about 1 170 small rectangles at
  most, one path a colour); it is not in a framebuffer and its cost was not measured ⚠.
  `pi_weave` itself is called only when the view changes.
- Contrast is by arithmetic. Nobody has looked at it on a phone in daylight or in Rack at
  normal zoom.
- ⚠ **The design session of 2026-11-26 (D112 to D117) is drawn and not built.** Its boards
  (`tui/`) were not rendered and checked by the session that drew them, and are kept as
  Claude Design source, not as pictures that open (issue 98). To build: the view for
  the new lights (O32, issue 92), then the lights on the page (issue 93) and in
  Rack (issue 94); the Shift latch and the shared keys (issue 96); the helper
  (issue 97); layout B (issue 95, after O34 and O35). For the owner: O31,
  O33, O34, O35, O36.
- O3 (the name) and O7 (a lawyer's view of the mock-up) are as open as they were.
