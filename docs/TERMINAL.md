# The terminal host

**Status (2026-11-26):** designed in issue 8 (D123), for Kenny; built the same day ("Built",
at the end). The look is a stand-in until Kenny rules it. Rhythm in a Linux
terminal, from the same core as the page and Rack. The engine does not change: this host
sends what a finger did and draws the 64-byte view (`ENGINE.md` §13, §14), as the others do.

What this document settles, each tested on Kenny's machine (Omarchy, Hyprland 0.56, foot
1.28, tmux 3.8, herdr 0.9.5, PipeWire 1.6.9; 2026-11-26) and recorded in `REFERENCES.md`
R78 to R81:

| Unknown (issue 8) | Answer | How it was found |
|---|---|---|
| Does foot report key-up? | **Yes.** It answers the kitty keyboard protocol's query and reports press, release and Shift as separate events | R78 |
| Does tmux pass key-up through? | **No,** in any `extended-keys` setting. It does not answer the query, so the host knows to fall back | R79 |
| herdr? | **It looks as if it does and does not.** It answers the query, then sends each release straight after its press, before the key is up. The host treats herdr as a terminal with no key-up | R80 |
| Sound through PipeWire | **cpal on ALSA** (through PipeWire's ALSA plugin), five clean minutes; cpal's own PipeWire backend also clean, kept as an option | R81 |
| Shift+`7` without the protocol | Arrives as `&`. Read as Shift+`7` on a US layout only ⚠ | R79 |

## The crate

- **`term/`, its own crate and its own Cargo workspace** (package `pi-term`). The core keeps
  no dependencies; this crate has two. The root workspace excludes it, so `make build` and
  the core's lock file do not see them.
- **It links the core's static library through the C ABI,** the same `libpi_core.a` the C++
  harness and Rack link (`pi_engine.h`, mirrored in Rust). Its object exports only the
  `pi_*` symbols (checked with `nm`), so it links into a Rust program without a clash. A
  build script takes the library from `PI_CORE_LIB` (the Makefile passes the one `make test`
  just checked) or, with none given, builds it from `core/` with the same `cargo rustc`
  command `make build` uses. So `cargo build --release` in `term/` works alone.
- **The binary is `pi-rhythm`** ⚠: a working name. The family's name is not cleared (O3), so
  it is one line to change before anything is packaged.
- **One binary, no files beside it.** The starter pack's sixteen samples are compiled in
  (`samples/raw/*.s16`, 508 KB, CC0). The only file it writes or reads is the beat (below).
- **`--help` and `--version`.** The version prints the crate's version, the commit it was
  built from and the core's ABI number, which it checks against `PI_ABI_VERSION` at start.

Nothing is packaged, published or submitted by this design. It keeps the doors open: one
binary, `cargo install --path term` works, no runtime files to install, no root.

### The two libraries

| Need | Library | Licence | Why this one |
|---|---|---|---|
| Raw mode, the alternate screen, the size, the cursor | `crossterm` 0.29 | MIT | Runs on a native Windows console as well as on Unix terminals |
| Sound out | `cpal` 0.18 | Apache-2.0 | ALSA and PipeWire on Linux, WASAPI on Windows, CoreAudio on macOS, one API |

Not used: crossterm's key events on Linux. Through foot it reports Shift+`7` as `&` with the
Shift flag dropped (R78), and the latch needs Shift. The host reads the keys itself (below).
Not used: a screen library (ratatui and the like): the screens are fixed grids of cells,
drawn from a cell buffer by this crate, with only what changed sent.

Apache-2.0 allows linking into an MIT program; a binary someone distributes carries cpal's
notice. `CREDITS.md` lists both.

## Sound

- **cpal's default host, on the default output device,** at the device's own rate (48 kHz on
  Kenny's machine), in its default buffer. With the `pipewire` cargo feature, cpal's own
  PipeWire backend instead (`--audio pipewire`). `--audio none` runs with no sound at all,
  for a machine without a device and for the tests.
- **The engine runs in the audio callback** (`PI_AUDIO_MIX`, one float a frame, copied to
  every channel). The callback allocates nothing and takes no lock: commands come in through
  a fixed ring of `pi_cmd` (a single producer and a single consumer), the view goes out
  through a sequence lock of sixteen words. A command lands on the first frame of the next
  callback.
- **The host lends effect 4 its line** (`pi_set_delay`) and the sixteen samples
  (`pi_set_sample`). Every voice is code at the start, as on the page; the sounds sheet
  switches each.
- **Loudness:** the mix as the page plays it, clamped to ±1. No limiter of the host's (O14 is
  still each host's ceiling).

What was measured, and what was not: R81. ⚠ The five clean minutes were measured, not heard.
Issue 56's loss is in recording through PipeWire, not in playing; it was not reproduced here
and is not closed by this.

## The keys

### Reading them

On Unix the host reads the terminal's bytes itself:

- **At start it asks:** it pushes the kitty keyboard protocol's flags 1, 2, 4 and 8
  (disambiguate, event types, alternate keys, every key as an escape) and sends the query
  `CSI ? u` followed by `CSI c`. A reply to the query before the device attributes means
  the protocol is on. It pops the flags at exit, and on a panic.
- **With the protocol** (`CSI code[:shifted[:base]] ; mods[:type] u`) a key is its base
  layout code when the terminal sends one, else its unshifted code. So the key's place
  decides, as the page's `e.code` does: on a French layout the key where `q` is on a US one
  is still key 5. Shift is the modifier bit. Repeats (type 2) count for the one-key
  shortcuts and are ignored for the grid and the function keys, as the page does.
- **Without it** each byte is a tap: a letter or digit is its key, a capital is Shift and
  its key, and `&` `*` `(` are Shift with `7` `8` `9` on a US layout ⚠ (another layout
  sends another character, which is then not the latch). A lone `ESC` is esc; `Ctrl+Q`
  (0x11) and `Ctrl+C` (0x03) quit.

### Hold or tap

`--keys auto` (the default) holds where the terminal reports key-up and taps where it does
not. `--keys hold` and `--keys tap` force either.

| The terminal | Mode | Why |
|---|---|---|
| answers the query, and `HERDR_ENV` is not set | hold | R78 |
| answers the query inside herdr (`HERDR_ENV=1`) | tap | its releases are not real (R80) |
| does not answer (tmux, the Linux console, most others) | tap | no key-up to read (R79) |

**Hold** is the page's logic, line for line (`web/rhythm.html`, `press`, `release`,
`latch`): a key down sends `KEY_DOWN`, its release `KEY_UP`; Shift with sound, pattern,
bpm, function or fx latches it (D113); a latched key lets go when the next press ends, but
fx, which stays latched until Shift-pressed again and is down only while a grid key is
(D78).

**Tap** is D117 item 4 and D119 item 5:

- A grid key sends `KEY_DOWN` and `KEY_UP` at once: a step flips, a sound fires or is chosen.
- A tap on sound, pattern, bpm, function or fx latches it, as Shift does in hold. A second
  tap lets it go. Play and write act on the press, as everywhere.
- **With fx latched, a tap on a grid key starts that effect** (fx and the key go down and
  stay down), **a second tap on it stops it** (both up), **and a tap on another key
  switches** (the first key up, the other down, fx still down). `esc` lets everything go.
- A tap of fx alone never clears the lane: fx latched is down only while a grid key is
  (D78), the same rule that makes the latch the safe way to look (D119 item 4).

### The map

The shared map, as on the page and in Rack (D28, D116, D119 item 3), and the steps of D122:

| Keys | Do |
|---|---|
| `1234` `qwer` `asdf` `zxcv` | the grid, keys 1 to 16 |
| `7` `8` `9` | sound, pattern, bpm |
| `u` `i` `o` `p` | function, fx, play, write |
| Shift + a function key | latch it |
| `t` `g`, `y` `h` | slider A, B up and down: 4096 from where the hand left it |
| `space` | play, stop |
| `,` `.` | tempo down, up: 1 bpm, 60 to 240 |
| `<` `>` | swing less, more: 1, 0 to 48 |
| `-` `=` | volume down, up: 1, 1 to 16 (bpm held and a key, as the page sends it) |
| `m` | the sounds sheet |
| `?` | the keys helper |
| `esc` | lets every latch go; closes a sheet |
| `:` | a command line: `w` saves the beat, `e <file>` opens one, `q` quits ⚠ |
| `Ctrl+Q` | quits ⚠ |

⚠ `:` and Ctrl+Q are D116 item 2's, not ruled (D119 item 3 does not name them). This design
proposes them as written; `:`'s "export" (D116) is left out until someone asks for it.

**The sounds sheet** (`m`) is the page's sheet in cells: the sixteen sounds by their keys,
each saying code or sample and whether it goes through the effects. A grid key switches its
sound between code and sample and plays it (as a tap on the page's sheet does); Shift and
the key switches whether it goes through the effects. `esc` or `m` closes it.

## The screens

Drawn from the view only, by `ENGINE.md` §14's host table ("§14 and §17, designed
2026-11-26"), and from what the host itself holds: its latches and the hand's slider
positions. The weave is `pi_weave`'s cells, unchanged. Nothing is read from a second idea of
the lights.

- **80 by 24 or more:** the main screen, `design/DESIGN.md`'s "Loom in a terminal", drawn
  at the top left.
- **48 by 10 to 79 by 23:** the pocket size. `--pocket` forces it.
- **Smaller:** one line asking for 48 by 10. The sound goes on.
- **Colour:** truecolour, Loom's tokens. With `NO_COLOR` set (any value) or `--no-color`:
  normal, bold and reverse only (D115 item 5).

### What each light is in cells

The main screen's key is 5 by 3 cells (D115 item 3), from `Lights`' terminal row:

| Light | Cells |
|---|---|
| frame | the top row's lower halves, the side cells' outer halves, the bottom row's upper halves: ochre on the playhead, else the face's colour |
| field | the middle row's background: ecru, or tile |
| core | the centre cell: white, or fern through the effect heard. On a lit field the two cells beside it go dark, standing in for Loom's dark line (D115 item 4) |
| twill | the cells either side of the core, hatched `╱` in fern on ground |
| stitch | `▪` right of the legend, dark on ecru, ecru on dark; not drawn under the twill (the board's rule ⚠) |
| bar | the bottom row's lower halves, the full width: madder; ochre while pattern is held |
| pressed | the face white, the frame ground |
| source mark | left of the legend while sound or fx is held or latched: `-` a sample. The ring and the disc need Rack's cables and Rack's effects, which a terminal has not, so they never show here |
| legend | the key you press (`q`, not `5`), dark on a lit face or core, ecru on tile |

The pocket size keeps one row a key and every light ⚠ (`Pocket` drew one light a key, but
issue 8 asks both screens for every light): the frame is the two side cells; the source
mark or the twill's left half, the legend or the core, the stitch or the twill's right half
fill the three between; the bar underlines the three.

With no colour, a colour stands for a level: ground, display, tile and table are nothing;
ecru, lilac, thread, madder, sky and fern are normal; white and ochre are bold. A half-block
cell shows the halves that are not nothing; a lit background is reverse. The twill keeps its
hatching and the core its dark neighbours, so no light is told by hue alone (D115 item 2);
⚠ the frame on a lit key differs from the field only as bold from normal there.

### Function keys, sliders and the display

- **Function keys** as `States` draws them: idle, the band and the key letter in lilac;
  pressed (in `held`), white with a ground edge; latched, the whole face in its own colour
  (ecru for function, fx, play and write: the page's rule, so fx's band alone is fern) with
  the key letter reversed; play lit while playing, write while write is on.
- **The sliders:** a thread, the cap where the hand left it (the view's value while bpm is
  held), and the second mark (bytes 60 to 63) in madder, as on the page.
- **The status line:** `rhythm`, `pattern 1`; then a block for each held or latched function
  key in its colour, `fx n` in fern while an effect is heard (D81's block), the write mark
  in ochre, the step and the tempo (the edges' tempo, byte 30, while the clock is external).
- **The weave** in half blocks, 78 by 15 cells in 8 rows; the lane's ticks in fern; the
  sixteen step marks, beats taller, the playing step tallest and ochre.
- **While a function key is held or latched the display lists what each key now means**
  (D117 item 2): sound, the sixteen sounds with the chosen one marked; bpm, the volume on
  keys 1 to 16 and the sliders as swing and tempo; pattern, the one pattern on key 1;
  function, clear with pattern; fx, the sixteen effects by name.
- **The footer** changes with the mode, as `Main` draws it.

### Where the boards disagree, and what is drawn

`design/DESIGN.md` is the later reading where the boards differ (CONTRIBUTING, "A look"):

- `Main` with write off shows a firing as a whole white face: drawn as the core instead
  (D112 item 1).
- `Pocket` draws the fx key's band ecru: fern (D112 item 3).
- `Pocket` has no core, twill or bar: all three, as above.
- The screens are still *Proposed* on the canvas. They are built as stand-ins; Kenny
  rules the look after seeing them (issue 8's last box).

## The beat

- Saved as the core's snapshot (`pi_save`, version 4), the same bytes Rack keeps in a patch
  and the page in its storage, to `$XDG_DATA_HOME/pocket-instruments/rhythm.beat`
  (`~/.local/share/…` when that is unset). `--beat <file>` names another.
- Read at start if it is there; written a second after the generation last moved, and at
  quit. A file `pi_load` refuses is left alone and the host starts empty, saying so.
- One pattern, as the other hosts have it. Patterns (issue 4) are not waited for.

## The proofs

| What | How |
|---|---|
| **P2 through this host:** the fixtures' traces and views | `term/tests/fixtures.rs` runs every fixture as `harness/harness.cpp` does, at three rates, three block sizes, both sources and from a snapshot, but every `key_down` and `key_up` goes in as the bytes foot sends for that key (the kitty sequence for press and release) and through the host's own reader, map and latch logic. Its report must be the same bytes as the other three: `make test` compares it |
| Every key of the map, both modes | tests feeding bytes in, the kitty form and the plain one, and checking the commands that come out; the tap fallbacks one by one, D119 item 5's three cases among them |
| Every light, both screens, both colourings | tests that build view bytes for each light and read the cells back: the character, its two colours, and with no colour its level |
| Sound | a five-minute run on Kenny's machine, counting xruns, the longest gap between callbacks and frames against the clock (R81); Kenny listens |
| Kenny | plays a beat with holds and an effect punched in, in foot; judges the look and the sound |

**The showpiece's gate for this host** (D118 item 3): P8 asks of the terminal what it asks
of the page, a written list of each difference in what a press does, now including the tap
fallbacks; P9 asks nothing new of the terminal's look beyond Kenny ruling it, since it
is Loom in cells and draws nothing of anyone else's.

## Windows and macOS: what it would take (not built, not claimed)

- **A native Windows console** reports key-down and key-up in its own input records, with
  no protocol, and crossterm reads them as press and release events. So Windows needs a
  second reader, crossterm's events mapped to keys by their character and Shift, in place of
  the Unix byte reader; holds would work there with no fallback ⚠ (read, not tried). cpal
  reaches WASAPI with no change. Windows Terminal draws truecolour and half blocks. The beat's
  folder would be `%APPDATA%`.
- **WSL** draws as Linux does. Its sound goes through WSLg's PulseAudio server, which cpal's
  ALSA host reaches only through the PulseAudio ALSA plugin ⚠; whether the terminal passes
  key-up into WSL depends on the terminal, untested ⚠.
- **macOS:** cpal reaches CoreAudio; kitty, Ghostty, WezTerm and iTerm2 speak the keyboard
  protocol, Terminal.app does not and would tap ⚠ (read, not tried).
- What would be needed for either: a machine to build and play on, the reader above for
  Windows, and the same tests. Nothing in this design shuts them out.

## Not designed here

- Packaging, store listings, a plugin for any host program: Kenny's, after the demos.
- More than one pattern (issue 4).
- MIDI, a clock in or out, recording to a file.

## Built (issue 8, 2026-11-26)

`term/`: `src/input.rs` reads the bytes, `src/host.rs` is the page's press, release and latch
logic with the tap fallbacks, `src/screen.rs` draws both screens, `src/audio.rs` runs the
engine in cpal's callback, `src/samples.rs` compiles in the starter pack, `src/beat.rs` keeps
the beat. `make term` builds it; `make test` runs its tests (`make term-test`).

What was checked, and how:

| Check | Result |
|---|---|
| P2 through the host's key path (`term/tests/fixtures.rs`) | 360 runs, the same report as Rust's, C++'s and Node's, byte for byte (`cmp` in `make test`) |
| Every key of the map, hold and tap (`term/tests/keys.rs`) | 18 tests: the protocol and plain bytes, Shift's latch, fx latched, the one-key shortcuts at D122's steps, the sheet, `:` and Ctrl+Q, D119 item 5's three cases, herdr's releases ignored |
| Every light, both screens, colour and none (`term/tests/screens.rs`) | 16 tests: frame, field, core (white, fern, its dark neighbours), twill, stitch, bar, pressed, the dash; each light alone differs from an unlit key with no colour; truecolour out, and no colour sends no colour |
| Planted faults, each built and caught by a named test | Shift dropped from the protocol (9 tests); two keys swapped in the map (9); no twill (2); fx let up between keys in a tap switch (1); a release read as a press (the fixtures: 150 of 360 runs) |
| Hold, end to end in foot | Keys sent with Hyprland's `send_key_state` to a hidden foot window (R78's method): a plain `7` let go before a key is no hold; Shift+`7` latches and lets go after the next key; `7` held over `s` chooses sound 10. The beat file read back held exactly those steps |
| Tap, end to end in tmux | `i`, then taps on `q`, `w`, `w`, `a`, then esc: effects 5, 6, none, 9, none in the status line |
| Both screens, both colourings | drawn in a headless tmux and read back (`capture-pane -e`): main, sound held, the helper, the sheet, the pocket size, and both with `NO_COLOR`. Kept in `design/term/*.ans`, for Kenny's eye: `cat design/term/term-write-on.ans` in a truecolour terminal shows the screen as drawn |
| Easy to package later | `cargo build --release --locked` in `term/` with no `PI_CORE_LIB` builds the core itself and links it: one binary of 1 142 232 bytes, needing only `libasound.so.2` and libc at run time (`ldd`) |
| The beat | saved at quit and a second after a change, read at the next start; a version 4 snapshot, 1464 bytes |

⚠ Found on the way: Hyprland's `send_key_state` moves focus to the window and back around every
event, and the host, like the page, lets every held key go when focus leaves. The end-to-end
check sets `PI_RHYTHM_TEST_IGNORE_FOCUS=1`, a test aid and not an option, to hold keys across
its events. A player's focus does not move, so this is not a fault of the host.

⚠ Not checked by anyone yet: holds and an effect played by hand; the look and the sound judged
(issue 8's last box, Kenny's); five minutes of `pi-rhythm` playing a beat through the speakers (it ran
five minutes on the Scarlett stopped, so silent: no xrun, ERR 0 on its node and the device in
`pw-top`; R81 measured the engine playing, at gain 0); `pi-rhythm` built with the
`pipewire` feature (it builds; it was not run).
