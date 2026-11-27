# The beat file

**Status:** designed 2026-11-27 in issue 4's design pass (D134), accepted by Kenny (D136).
Built 2026-11-27 in issue 140: "Built", at the end.
The readable text form of a beat (D110 items 6 and 7): one thing that an agent can write, a
link can carry, a host can save and load, and a bank of our own beats ships in (D131).

Why one form: "the link that carries a beat, saving on either host, a pattern library and an
editor all want the same thing" (D110 item 7).

## What it is

**A beat file is snapshot version 5 (`ENGINE.md` §21) written as text, one to one.** Every
field of the snapshot has one place in the text, and the text holds nothing the snapshot
does not, but one note (`own`, below). So:

- reading a file and writing it again gives the same text, when the file is written the way
  the writer writes (its canonical form, below);
- a snapshot written as text and read back is the same bytes;
- the core's own load and save are the referee: text read into bytes, loaded by `pi_load`
  and saved by `pi_save` must give those same bytes.

**Its own version number** is on its first line: `pocket-instruments beat 1`. Beat 1 maps onto
snapshot version 5. A later snapshot version (the step multiplier, D110 item 6, will need
one) brings beat 2, which adds words and drops none.

- **An older file in a newer build** reads, always: every beat version ever written must
  still read, as every snapshot version must still load (§8). What it does not say takes the
  default.
- **A newer file in an older build** is refused with what was found and what the build
  reads: `this is beat 2; this build reads beat 1`. Never half read.
- **An unknown word, a bad value or a row of the wrong length is refused** with its line
  number and what the line takes. Strict on purpose: an agent's slip is caught at once, not
  played wrong.

## Who reads and writes it

**Not the core.** The core has no allocator and carries no parser (§8, D25), and that does
not change. The text is turned into snapshot bytes and back by **one small crate,
`beat/`,** beside `core/` in the root workspace:

- **`no_std`, no allocator, no dependencies,** as the core, so it builds to a static library
  that Rack and the terminal host link, and to a `.wasm` that imports nothing, which the page
  loads beside `pi_core.wasm` (`web/pi_beat.wasm`, committed as `pi_core.wasm` is).
- **A C ABI of two calls,** on the caller's buffers, pure functions of their bytes: no
  engine, no thread rule (like `pi_weave`), never on the audio path.

  ```
  int32_t pib_read(const uint8_t *text, uint32_t len, uint8_t *snap, uint32_t cap,
                   uint32_t *own, uint8_t *msg, uint32_t msg_cap);
          /* the snapshot's length; or < 0, with "line 12: ..." in msg */
  int32_t pib_write(const uint8_t *snap, uint32_t len, uint32_t own,
                    uint8_t *text, uint32_t cap);
          /* the text's length; or < 0 (not a version 5 snapshot, too little room) */
  ```

- **Where text becomes a snapshot:** in the host, off the audio thread: `pib_read`, then the
  bytes go to the core by the host's ordinary load (§8: a marker in the queue, loaded between
  blocks). **Where a snapshot becomes text:** the host gives `pib_write` the latest snapshot
  its audio thread handed over (§8), and a version 1 to 4 snapshot is first loaded and saved
  by the core to make it version 5.
- **Who calls it:** the page (open and save a beat, issue 7's link later), Rack (its menu),
  the terminal host (`:e`, `:w`, and its own saved beat, below), the agent's command (issue
  6), and the bank of beats (issue 134).

**How the two are kept from drifting.** The snapshot's offsets live in one Rust file,
`core/src/snapshot_layout.rs`, of constants only, which the core uses and `beat/` includes
by `include!`: one table, two crates, no dependency between them. And `beat/`'s tests, run
by `make test`:

1. Every fixture's version 5 snapshot, and every version 1 to 4 snapshot in
   `fixtures/snapshots-v*.txt` loaded and saved by the core: `pib_write` then `pib_read`
   gives the same bytes.
2. Every `.beat` file in the repository (the worked example below, the bank): `pib_read`,
   `pi_load` succeeds, `pi_save` gives the same bytes, and `pib_write` gives the same text.
3. The worked example's trace and views (`fixtures/designed/beat-four-and-a-fill.json`)
   through all three harnesses, the beat read by `pib_read`.
4. A differential fuzz: random edits on an engine, saved, written as text, read back: the
   same bytes, every seed.

Not chosen: a reader in each host's language (JavaScript, C++, Rust: three to drift apart);
the core reading text (a parser on the audio path, against D25); the beat as JSON (a reader
for JSON in `no_std` is larger than this grammar, and a grid is not readable in JSON).

## The form

UTF-8 text, one statement a line. A `#` starts a comment to the end of the line, except
inside `name` and `credit`. Blank lines and leading spaces are ignored. Words are lower case.
Numbers are decimal. A key or a pattern is numbered 1 to 16, as on the instrument; a step is
numbered 1 to 16, as a player counts.

| Line | Means | Default |
|---|---|---|
| `pocket-instruments beat 1` | the first line, always | (required) |
| `name <text>` | the beat's name, to the end of the line, at most 32 bytes | none |
| `credit <text>` | who made it, and what it is from: at most 128 bytes | none |
| `tempo <bpm>` | 30 to 300, up to three decimals (`97.5`) | 120 |
| `swing <ticks>` | 0 to 48, of a step's 96 | 0 |
| `volume <n>` | 1 to 16 | 16 |
| `seed <n>` | the random generator's seed, 1 to 4294967295 | 1 |
| `chosen <key>` | the chosen sound | 1 |
| `sliders <A> <B>` | every sound's own A and B, 0 to 65535 | 32768 32768 |
| `sound <key> <words>` | one sound apart from the rest: `sample` or `code`, `dry` or `fx`, `a <A>`, `b <B>`; any of them, in that order | code, fx, the `sliders` |
| `own <key>...` | these keys played the writer's own sample (D34; below) | none |
| `value <letter> <A> <B>` | a pair of slider values steps can keep; letters `a` to `w` and `A` to `W` (not `x` or `X`) | none |
| `chain <pattern>...` | one to sixteen patterns, in play order | `1` |
| `pattern <n> [length <L>]` | the lines after it, to the next `pattern`, are pattern *n*'s; length 1 to 16 | length 16 |

Inside a pattern:

| Line | Means |
|---|---|
| `<key> [<name>] <sixteen steps> [prob ...] [keep ...]` | one sound's row |
| `fx <step>=<effect>...` | the lane: effect 1 to 16 on those steps |

**A row's sixteen steps**, always sixteen whatever the length (a step past the length is still
held, and comes back when the length grows):

| Character | The step |
|---|---|
| `.` | off |
| `x` | on, keeping nothing: it plays its sound's own A and B (as a step set by an edit) |
| a `value` letter | on, keeping that pair (as a step entered by a key: §20) |
| `*` | on, keeping the pair its `keep` list gives |

- **`prob <step>=<p>...`**, 0 to 100, for any step whose probability is not 100 (on or off).
- **`keep <step>=<A>,<B>...`** for a `*` step, and for an off step that keeps values (an edit
  can make one: `ENGINE.md` §20, "⚠ `KEEP` on a step that is off keeps").
- **The name**, if given, must be the key's: `kick snare hat openhat synsnare sticks cymbal
  noise clap click lowtom hitom cowbell blip tone bass` for keys 1 to 16 (`docs/VOICES.md`'s
  roles). A wrong name is refused, so an agent that means the snare on key 1 is told.
- A sound with nothing on, nothing kept and every probability 100 has no row. A pattern
  with no row and no lane, length 16, has no block. The file's patterns are those it names.

**Canonical form** (what `pib_write` writes): the lines in the order of the table above;
a line left out when it holds its default; `seed` always written; `sound` lines in rising
key; letters given in the order the steps first use a pair, reading pattern 1 to 16, key 1 to
16, step 1 to 16, `a` first; past 46 pairs, `*` and a `keep` list; rows in rising key, each
with its name, the key in two columns and a space, the name padded with spaces to one more
than the longest name in the file; `prob` and `keep` lists in rising step, one space
between fields; `fx` last in each pattern; one blank line before each `pattern`. A file in
any other order or spacing reads the same and is written back canonical.

**What it cannot carry: a player's own samples** (D34). The snapshot says only that a key
plays *its* sample, not which; a host holds the sounds (§12). So `sound 9 sample` means the
starter pack's sample on key 9 wherever the file is opened. When a key the writer saved
plays the writer's own sample, the host passes that key in `own` and the file says
`own 9`; whoever opens it hears the starter pack's sample there, and the host says so: "key
9 used its writer's own sample; playing the starter pack's". `own` is the one line that is
not in the snapshot; reading it back into bytes gives `sample`.

Also not carried, because a snapshot does not hold them: the playhead, the chain's position,
playing or stopped, write mode, the clock source, the sliders' second marks, Rack's routes
and cables.

## Size, and a link

| Beat | Text | Snapshot |
|---|---|---|
| The worked example below: two patterns, a chain of four | 428 bytes (counted: `fixtures/designed/beat-four-and-a-fill.beat`) | 3 004 bytes |
| One pattern of four sounds | about 250 bytes | 1 640 |
| Sixteen full patterns, every step keeping one of 46 pairs | about 6 KB | 22 100 |
| The worst: sixteen patterns, every step on with its own pair (`*` and `keep`) | under 64 KiB (`PIB_TEXT_MAX`) ⚠ by arithmetic; the build measures it | 22 100 |

**Assumed for the link (issue 7), not designed here:** a typical beat is under 2 KB of text,
so a link can carry it compressed in its fragment (a browser has `CompressionStream`) or
even as text. The characters are chosen so that nothing but the spaces and line ends needs
escaping. A beat at the worst size may be too long for a link; issue 7 says what then.

## A worked example: "Four and a fill"

The file is `fixtures/designed/beat-four-and-a-fill.beat`, byte for byte. A real beat of our
own (CC0): a four-to-the-floor house pattern played three times, then a
fill with a snare roll, chained. The Amen Break across four patterns is issue 134's, and
what is relied on to ship its rhythm is not ruled there yet, so it is not the example.

```
pocket-instruments beat 1
name Four and a fill
credit Pocket Instruments contributors, 2026; CC0
tempo 124
swing 12
volume 12
seed 1
sliders 30000 36000
sound 9 sample dry
value a 32768 32768
value b 24000 40000
chain 1 1 1 2

pattern 1
1  kick  a...a...a...a...
3  hat   ..a...a...a...a.
9  clap  ....b.......b...

pattern 2
1  kick  a...a...a.a.....
2  snare ............xxxx prob 15=70
3  hat   ..a...a.........
fx 13=2 14=2
```

What each line fills in snapshot version 5 (`ENGINE.md` §21's table):

| Line | Bytes |
|---|---|
| first line | magic, version 5, kind 1; beat 1 |
| `name`, `credit` | 112 to 143, 144 to 271, zero-padded |
| `tempo 124` | 16: 124 000 |
| `swing 12` | 20: 12 |
| `volume 12` | 21: 12 |
| `seed 1` | 12: 1 |
| (no `chosen`) | 22: 0, the kick |
| `sliders 30000 36000` | 24 to 87: every sound's own A 30 000 and B 36 000 |
| `sound 9 sample dry` | 88: `sampled` bit 8 (0x0100); 90: `fx sounds` all but bit 8 (0xfeff) |
| `chain 1 1 1 2` | 94: 4; 96 to 99: 0, 0, 0, 1 |
| `pattern 1`, `pattern 2` | 92: written 0x0003; two blocks from 272, length 16 each |
| `1 kick a...a...a...a...` | pattern 1: steps of track 0 = 0x1111; keeps of track 0 = 0x1111; values of steps 1, 5, 9, 13 = (32 768, 32 768) |
| `3 hat ..a...a...a...a.` | track 2: steps and keeps 0x4444, values (32 768, 32 768) |
| `9 clap ....b.......b...` | track 8: steps and keeps 0x1010, values (24 000, 40 000) |
| `1 kick a...a...a.a.....` | pattern 2, track 0: steps and keeps 0x0511 |
| `2 snare ............xxxx prob 15=70` | track 1: steps 0xf000, keeps 0; probability of step 15 = 70, the rest 100 |
| `3 hat ..a...a.........` | track 2: steps and keeps 0x0044 |
| `fx 13=2 14=2` | pattern 2's lane: steps 13 and 14 hold effect 2 |
| (the end) | CRC; 276 + 2 × 1 364 = **3 004 bytes** |

**What it plays** (by `ENGINE.md` §5, §17 and §21; the trace is in
`fixtures/designed/beat-four-and-a-fill.json`, worked by hand). Bars 1 to 3 are pattern 1:
the kick on every beat, the hat between, the clap on 2 and 4, none of them on an odd grid
step, so the swing moves nothing. Bar 4 is pattern 2: the kick on 1, 5, 9 and 11, the hat on
3 and 7, and the snare on 13, 14, 15 and 16, where 14 and 16 are odd grid steps, 12 ticks
late; step 15's 70% takes the beat's first draw from seed 1, 270 369, and fires (69 is under
70). Effect 2 on steps 13 and 14 changes the sound, not the trace. Bar 5 is pattern 1 again:
the chain starts over (R86). Every sound fires through the effects but the clap, which plays
its starter-pack sample.

## The terminal host's beat

The terminal host saves its beat in one file today (`docs/TERMINAL.md`, "The beat": a
version 4 snapshot, binary, at `~/.local/share/pocket-instruments/rhythm.beat`). Proposed
(D134 item 5): it keeps that path and writes the **text** there from then on, so an agent or
a person can open the beat in an editor. A file that starts with the bytes `PKIN` is read as
a snapshot, as now, and written back as text at the next save. ⚠ Kenny to rule; the other
reading keeps the binary file and adds `:w file.beat`.

## Built (issue 140, 2026-11-27)

`beat/`, beside `core/` in the root workspace: `no_std`, no allocator, no dependencies, stable
Rust. The C ABI is `beat/include/pi_beat.h`: `pib_read` and `pib_write` as designed above, with
the results below zero named (`PIB_ERR_TEXT` -1, `PIB_ERR_ROOM` -2, `PIB_ERR_NEWER` -3,
`PIB_ERR_SNAPSHOT` -4). Built as `target/release/libpi_beat.a` and `web/pi_beat.wasm` (39 999
bytes, imports nothing; exports `memory`, `pib_read`, `pib_write`), for size (`opt-level = "s"`:
59 871 bytes at the core's level 3). Nothing in the core changed but where its snapshot code
finds its offsets.

- **The one table:** `core/src/snapshot_layout.rs`, constants only, a module of the core and
  `include!`d by `beat/src/lib.rs`. Compile-time checks in it tie each field to the next; the
  core asserts the table's counts are its own. Planted: the core saving the name one byte late
  (a literal in place of the table) failed four of `core/src/tests/beat.rs`'s tests by name.
- **The tests this file asks for,** in `make test`: `core/src/tests/beat.rs` (1, 2 and 4, with
  `pi_load` and `pi_save` as the referee; the crate is a dev-dependency of the core's tests only)
  and `fixtures.rs` (3: `beat-four-and-a-fill` moved up unchanged and passed at its first run, in
  Rust, C++, Node and the terminal host). The Rust tests also write the cases (every core
  snapshot, a share of the fuzz's, every `.beat`, refusals, edited texts); the C++ harness on
  `libpi_beat.a` and Node on `pi_beat.wasm` run them, and the three reports must be the same
  bytes. `beat/src/tests/` holds the crate's own: the worked example against the table above,
  every refusal's message in full, random snapshots through text and back, 20 000 random edits
  of good text with never a panic.
- **`PIB_TEXT_MAX` is 98 304 (96 KiB), not 64 KiB.** The worst text measured is 94 349 bytes:
  sixteen patterns, every step on with its own pair and a probability of 99, every sound apart
  from the rest; the bound summed line by line is 95 038 (`beat/src/tests/size.rs`). The
  arithmetic above, "under 64 KiB", left out the `prob` lists and the 46 value lines.
- **Readings where this file is silent,** each ⚠ for Kenny, each built round and tested:
  1. `sliders` gives the value most sounds have, a tie to the lowest key's; the rest are `sound`
     lines' `a` and `b`.
  2. A line that comes once (`tempo`, `name`, a `sound` key, a `value` letter, a `pattern`, a
     row's key in a pattern, `fx` in a pattern, `prob` and `keep` in a row, a step in a list) is
     refused the second time, with the line of the first.
  3. The beat's own lines may stand anywhere after the first line, also after a `pattern`: a
     row and `fx` are told apart by their first word. Rows and `fx` before any `pattern` are
     refused.
  4. A `pattern` named with nothing in it is a cleared one: no block, as the core would save it.
  5. `name` and `credit`: the spaces at either end dropped, and again after a cut; no tabs or
     control characters; `name` with no text refused. `pib_write` refuses a snapshot whose name
     or credit the text could not carry back to the same bytes.
  6. `own`: a key it names plays its sample; `own` with `sound <key> code` is refused. Written
     after the `sound` lines, and only for keys that play a sample.
  7. Letters are given to steps that are on; a kept step that is off is always in `keep`.
  8. The first line must be line 1. Line ends `\n` or `\r\n`; words are separated by spaces, a
     tab is refused. Numbers may have leading zeros (`seed 0001`); `tempo 120.000` reads.
  9. A refusal's message is `line L, column C: ...` (C in characters), or `line L: ...` where no
     one column is wrong; a word quoted is cut after 24 bytes.
  10. `pib_write` refuses any snapshot the core would not write (a field outside its range, a
      cleared pattern written, bytes after a chain's length, a value kept where nothing is):
      so whatever it writes reads back to the same bytes.

## The agent's command (designed 2026-11-27, D140; proposed, for Kenny to rule; not built)

Issue 6: "a small command checks a beat file, renders it to sound, and makes a link or a Rack
paste". Nothing below is built; the build is issue 163. Where this section and D140 differ,
D140 is the record. The bank of beats it is proved on is `docs/BANK.md`.

**What it is: `node beat/beat.mjs`, one Node script over the two committed `.wasm` files**
(`web/pi_core.wasm` and `web/pi_beat.wasm`). An agent with only the repository needs Node and
nothing else: no Rust, no C++ compiler, no ALSA headers, no build. The bits are the native
ones: `make test` already holds the `.wasm` to the same report as the static library (P2), and
fails if a committed copy is stale. `--built` takes the copies `make build` just wrote in
`target/` instead. It reuses the Node harness rather than repeating it: the module loading and
the beat calls of `harness/beat.mjs`, and the starter pack, effect 4's line and the render loop
of `harness/trace.test.mjs`, moved into one module that both import (no byte of any report
moves). One instrument, one script: nothing generic.

```
node beat/beat.mjs check <file.beat>...
node beat/beat.mjs render <file.beat> [-o <out.wav>] [--rate 44100|48000|96000] [--times <n>] [--trace]
node beat/beat.mjs pack <file.beat> [-o <out.beat>]
```

| Exit | Means |
|---|---|
| 0 | every file read by `pib_read` and loaded by `pi_load` (and, for `render` and `pack`, written) |
| 1 | a file is refused: by `pib_read` (its message), or by `pi_load` (its code) |
| 2 | the command misused, a file that cannot be read or written, or `--link` (issue 7, not built) |

**check** reads each file with `pib_read`, loads the snapshot into a fresh engine (`pi_init`,
then `pi_load`) and saves it again with `pi_save`. One line a file, on standard output:

```
beats/amen-break.beat: ok. "Amen Break", 136 bpm, chain 1 2 3 4: 4 bars, 7.059 s; patterns 1 2 3 4; snapshot 5 732 bytes
mine.beat: line 14, column 12: <the crate's message, as pib_read wrote it>
```

A file that reads but is not in canonical form adds `(not canonical: pack writes it so)` and
still exits 0. A file with `own` keys adds the hosts' words: `key 9 used its writer's own
sample; playing the starter pack's`.

**render** plays the beat once through its chain and writes a WAV, by default
`target/beat/<file name>.wav`:

- **Format: WAV, 32-bit IEEE float, mono, 48 000 Hz.** The engine renders 32-bit floats
  (`PI_AUDIO_MIX`), and a float WAV holds them bit for bit: its data is the same bytes as a
  harness `.f32`, with no dither or rounding to choose, and audio players and editors open it.
  `--rate` takes the harness's other two rates.
- **What plays:** the starter pack on every key (a `sample` key plays its sample, as on every
  host), effect 4 lent its line (`pi_set_delay`), the table clock, `PLAY 1` on frame 0.
- **The length:** from step 1 of the chain's first pattern to the step where the chain would
  start over: the sum, over the chain's entries, of each pattern's length times a step
  (60 / tempo / 4 seconds). `--times <n>` plays the chain *n* times (1 to 16). Swing moves no
  step past that end.
- **The tail:** then `PLAY 0`, and the render runs on until half a second in a row stays under
  −80 dBFS (1e-4, where a voice is cut to zeros, `ENGINE.md` §11), at most 4 s (a cymbal rings
  at most 1.2 s; effect 4's echo is what the cap is for).
- **What it prints,** so a beat can be read without ears: the file written, its length (chain
  and tail), the peak, and every bar of the pass as a sixteen-step grid per sound, from the
  trace (`x` a firing; a digit where an effect's repeat fired that many times, against the step
  its run began on, `ENGINE.md` §17):

  ```
  target/beat/amen-break.wav: 48 000 Hz, mono, 32-bit float; 7.059 s of chain, 0.989 s of tail; peak 0.819 (-1.7 dBFS)
  bar 1 (pattern 1)
     1 kick     |x.x.|....|..xx|....|
     2 snare    |....|x..x|.x..|x..x|
     3 hat      |x.x.|x.x.|x.x.|x.x.|
  ```

  `--trace` prints the trace's lines instead, in the fixtures' form (`0 trig 0 0`), so an agent
  can make a fixture of what it rendered.

**pack** writes the beat in canonical form (`pib_read`, then `pib_write`) to standard output,
or to `-o`. That one text is everything the command hands on:

- **To the link (issue 7):** the canonical text, as UTF-8 bytes, after a check passes. Issue 7
  designs the link's form; until it is built, `--link` exits 2 and says so.
- **To Rack:** the `.beat` file itself, which the Rhythm module's beat menu opens (issue 142),
  as the page's Open does (issue 141) and the terminal's `:e` (issue 143). One file, three
  hosts. Not a `.vcv` patch (D140 item 4 says why).

**In CI** (`make test`, so `.github/workflows/test.yml` runs it): `check` on every `.beat` in
`fixtures/` and `beats/` exits 0; `check` on a refused text kept inside the test (not a `.beat`
in either folder: the core's tests read every one of those and require it to pass) exits 1
with the crate's message, byte for byte; `render --trace fixtures/beat-four-and-a-fill.beat
--times 2` gives the first 80 steps of `fixtures/beat-four-and-a-fill.json`'s trace, worked by
hand long before; `render` on every bank file exits 0 with a peak under 1, a WAV whose length
is the chain's plus a tail, and the same WAV bytes from the committed `.wasm` as with
`--built`; `pack` on every bank file gives the file back unchanged.

### What `AGENTS.md`'s "Playing the instrument" will say once built (a draft; not in AGENTS.md yet)

> ## Playing the instrument
>
> A beat is a text file, `docs/BEAT.md` ("The form"): sixteen patterns of sixteen steps, a
> chain, the tempo, each sound's sliders. Keys: 1 kick, 2 snare, 3 hat, 4 open hat, 5 synth
> snare, 6 sticks, 7 cymbal, 8 noise, 9 clap, 10 click, 11 low tom, 12 high tom, 13 cowbell,
> 14 blip, 15 tone, 16 bass (`docs/VOICES.md`). There is no velocity: a quieter hit is a
> shorter one (slider B), kept on its step with a `value` letter. Start from the bank in
> `beats/` (`docs/BANK.md`).
>
> ```
> node beat/beat.mjs check my.beat             # what is wrong, with line and column; exit 1 if refused
> node beat/beat.mjs render my.beat            # target/beat/my.wav, and every bar printed as a grid
> node beat/beat.mjs pack my.beat -o my.beat   # canonical form: the file the page, Rack and the terminal open
> ```
>
> Read the grid `render` prints against what you meant before you say it is done; nobody has
> heard it until a person does. Write no pattern of Teenage Engineering's (none a unit ships
> with, D137), none from Pocket Operations (O29) and none from a pattern book or product.
> Never write over a player's saved beat (the terminal's `rhythm.beat`, a page's storage, a
> patch) unasked.

**The acceptance example,** "Please create a 4-pattern Amen Break on my pocket instrument
rhythm": the agent reads that section and "The form", writes `amen.beat` (four patterns, chain
`1 2 3 4`, from the break's commonly described structure, or starting from
`beats/amen-break.beat`), then runs:

```
node beat/beat.mjs check amen.beat
node beat/beat.mjs render amen.beat          # reads the four grids against the structure it meant; fixes and runs again
node beat/beat.mjs pack amen.beat -o amen.beat
```

It answers with the file, the WAV's path and the four grids, and how to open the file: on the
page (Open), in Rack (the module's menu, Beat, Open) or in the terminal (`:e amen.beat`). Into
the terminal's own saved beat only if the player says so, keeping the old one beside it.
