# The bank of beats, and the beat a new player opens on

**Status:** designed 2026-11-27 for issue 134 (D141, D142), proposed, for Kenny to rule. The
seven beats are written and committed in `beats/`; ~~no host offers them yet~~ the three hosts
offer them since issue 164 ("Built", at the end). **Nobody has heard them.** Where this file and D141 or D142 differ, the decisions are the record. The
builds: issue 164 (the bank and the default on the three hosts), issue 165 (the reference patch).

Kenny, leaving for the weekend: "Really want patterns working well, and want to have a default
amen break pattern linked for any new user." D131 item 3 ruled the bank itself: four to eight
beats of our own, an Amen Break across four chained patterns as its headline, loaded from a
menu, never replacing a player's work unasked.

Why a file of its own, not a section of `BEAT.md`: `BEAT.md` is the format and the tool that
reads it; this is what ships in that format and how three hosts offer it, and it holds the
seven beats' grids.

## The seven beats

Each is a `.beat` file in `beats/`, in canonical form (`pib_write` gives the same text back),
with its `name` and `credit` in the file, so the credit travels in the snapshot into a patch,
a page's storage and the terminal's saved beat (D134 item 4). All are ours: none is taken from
Pocket Operations (O29), from any Teenage Engineering unit (D137), or from a pattern book or
product. The Amen Break is a well-known rhythm written out on our own sounds (below).

| Menu | File | Name | Tempo | Swing | Patterns; chain | What it is for |
|---|---|---|---|---|---|---|
| 1 | `amen-break.beat` | Amen Break | 136 | 0 | 4; `1 2 3 4` | The headline and the default (D142). Kick and snare on their samples, the ride on the closed hat, one crash; ghost snares short |
| 2 | `floor.beat` | Floor | 124 | 8 | 1 | Four on the floor: kick on every beat, clap on 2 and 4, open hat between, closed hats around it, four of them at 60% (shows `prob`) |
| 3 | `boom-bap.beat` | Boom Bap | 90 | 18 | 1 | A slow swung kick, snare and hat, a lower, longer kick (`sound 1 a b`), one rim click kept short |
| 4 | `two-step.beat` | Two Step | 132 | 20 | 1 | A skipping kick (its last at 50%), clap on 2 and 4, sticks between, hats on the off-beats, heavy swing |
| 5 | `half-time.beat` | Half Time | 140 | 0 | 2; `1 2` | The snare on beat 3 only; the bass tone (key 16) follows the kick, A1, C2 and G1, each note kept on its step (shows the tones) |
| 6 | `stutter.beat` | Stutter | 120 | 0 | 1 | A plain beat with the effects lane: high-pass (effect 7) on steps 9 to 12, repeat 8 (effect 11) on 13 to 16; the hat dry, so it keeps time through both (shows `fx` and `sound 3 dry`) |
| 7 | `backbeat-and-fill.beat` | Backbeat and Fill | 100 | 0 | 2; `1 1 1 2` | A plain backbeat three times, then a fill: snares, high toms, low toms (two of them a little higher) |

`beats/bank.txt` lists the files in menu order, one a line. **The first line is the beat a new
player opens on** (D142). A test holds the list to the folder: every `.beat` in `beats/`
listed once, every line a file there.

### Proved on 2026-11-27, nobody has heard them

Each file through the crate and the core on this branch (`origin/build-140-beat-file`, the beat
build #160 on the core build #153), with a scratch C++ program on `libpi_beat.a` and
`libpi_core.a` doing what `harness --render` does (the starter pack given to every key,
effect 4 lent its line, 48 kHz, `PLAY 1` on frame 0, the chain once, then `PLAY 0` and the
tail): `pib_read` with no error, `pi_load` accepts, `pi_save` gives the same bytes, and
`pib_write` gives the same text. `make test` also runs every file in `beats/` through
`pib_read`, `pi_load`, `pi_save` and `pib_write` in Rust, C++ and Node, since issue 140 built it
to read that folder.

| File | Text | Snapshot | Chain played | Tail | Peak |
|---|---|---|---|---|---|
| `amen-break.beat` | 714 bytes | 5 732 | 7.059 s | 0.989 s | 0.819 (−1.7 dBFS) |
| `floor.beat` | 272 | 1 640 | 1.935 s | 0.889 s | 0.361 (−8.9 dBFS) |
| `boom-bap.beat` | 288 | 1 640 | 2.667 s | 0.976 s | 0.489 (−6.2 dBFS) |
| `two-step.beat` | 256 | 1 640 | 1.818 s | 0.904 s | 0.423 (−7.5 dBFS) |
| `half-time.beat` | 487 | 3 004 | 3.429 s | 0.905 s | 0.593 (−4.5 dBFS) |
| `stutter.beat` | 253 | 1 640 | 2.000 s | 0.891 s | 0.508 (−5.9 dBFS) |
| `backbeat-and-fill.beat` | 398 | 3 004 | 9.600 s | 0.904 s | 0.423 (−7.5 dBFS) |

The first bar of each, as the trace fired it (`x` a firing; `8` eight firings of a repeat, from
the step its run began on, `ENGINE.md` §17). The rows of each file are the same grid with its
letters; the Amen's four bars are below.

```
Floor (seed 7: hat step 2's 60% did not fire this bar)
   1 kick     |x...|x...|x...|x...|
   3 hat      |x..x|xx.x|xx.x|xx.x|
   4 openhat  |..x.|..x.|..x.|..x.|
   9 clap     |....|x...|....|x...|
Boom Bap
   1 kick     |x...|...x|..x.|....|
   2 snare    |....|x...|....|x...|
   3 hat      |x.x.|x.x.|x.x.|x.xx|
   6 sticks   |....|....|...x|....|
Two Step (step 16's 50% did not fire this bar)
   1 kick     |x...|....|..x.|....|
   3 hat      |..x.|..x.|..x.|..x.|
   6 sticks   |...x|....|.x..|.x..|
   9 clap     |....|x...|....|x...|
Half Time, bar 1 of 2
   1 kick     |x..x|....|..x.|....|
   2 snare    |....|....|x...|....|
   3 hat      |x.x.|x.x.|x.x.|xxx.|
  16 bass     |x..x|....|..x.|....|
Stutter (kick and snare go through the effects, so repeat 8 takes them from step 13)
   1 kick     |x...|x...|x...|8...|
   2 snare    |....|x...|....|8...|
   3 hat      |x.x.|x.x.|x.x.|x.x.|
Backbeat and Fill, bar 4 (the fill; bars 1 to 3 are the backbeat)
   1 kick     |x...|..x.|....|....|
   2 snare    |....|x...|xx..|....|
   3 hat      |x.x.|x.x.|....|....|
  11 lowtom   |....|....|....|xxxx|
  12 hitom    |....|....|..xx|....|
```

## The Amen Break

**What it is** (R97, R99): the drum break in "Amen, Brother", an instrumental by the Winstons,
the B-side of their 1969 single "Color Him Father", played by the drummer Gregory C. Coleman.
Four bars, about seven seconds, at about 136 bpm (Wikipedia, citing a tutorial) or 132
(MusicRadar). "For two bars, Coleman plays the previous beat. In the third bar, he delays a
snare hit. In the fourth bar, he leaves the first beat empty, then plays a syncopated pattern
and an early crash cymbal" (R97).

**How ours was written.** From the commonly described notation only: Elephant Drums' drum tab
of the four bars (R98), read against Wikipedia's description (R97) and MusicRadar's (R99: kick
in "groups of two 16th-notes", "eighth-notes on the ride", a "2 and 4 backbeat along with
filling in some 16th-note ghost notes"; in bars three and four "the final backbeat of the bar
played an eighth-note later"). Only the tab gives steps; the two descriptions give
no grid, and nothing in them contradicts a step of it. No recording of the record
and no sample of it was played, opened or consulted, and none is in the repository.

The file's rows (`beats/amen-break.beat`; tempo 136; chain `1 2 3 4`):

```
            step  1234 5678 9012 3456
pattern 1   kick  x.x. .... ..xx ....
(bar 1)     snare .... x..a .a.. x..a
            hat   b.c. b.c. b.c. b.c.

pattern 2   the same as pattern 1 (bars one and two are the same)

pattern 3   kick  x.x. .... ..x. ....
(bar 3)     snare .... x..a .a.. ..x.      the backbeat on 13 moves to 15
            hat   b.c. b.c. b.c. b.c.

pattern 4   kick  ..xx .... ..x. ....      the first beat empty
(bar 4)     snare .a.. x..a .a.. ..x.
            hat   b.c. b.c. b... b.c.      the ride rests where the crash falls
            cymbal.... .... ..d. ....      the early crash
```

And as the core fired it, bar by bar (every letter above is a firing):

```
bar 1, 2      kick |x.x.|....|..xx|....|   snare |....|x..x|.x..|x..x|   hat |x.x.|x.x.|x.x.|x.x.|
bar 3         kick |x.x.|....|..x.|....|   snare |....|x..x|.x..|..x.|   hat |x.x.|x.x.|x.x.|x.x.|
bar 4         kick |..xx|....|..x.|....|   snare |.x..|x..x|.x..|..x.|   hat |x.x.|x.x.|x...|x.x.|   cymbal |....|....|..x.|....|
```

**What the format offers, and what was used.** A step has no level of its own: what a step
keeps is its sound's two sliders (`ENGINE.md` §20), and for the drums A is pitch and B is
length (`docs/VOICES.md`). So:

- **Ghost snares are short snares:** `a` keeps A 32 768 and B 13 107 (0.2 of the travel), the
  backbeats `x` play the snare's own 32 768 and 32 768. The same values as the reference patch's
  one bar (`rack/patch/make-showcase.mjs`). ⚠ Which snares are ghosts is our reading: R98's tab
  marks every snare alike; R99 says only that "some" sixteenths are ghosts. Here the backbeats
  (steps 5 and 13; 15 in bars 3 and 4) are full and the rest short.
- **The ride is the closed hat** (key 3), on the beat `b` (B 36 044, 0.55) and off it `c` (B
  22 937, 0.35), as in the reference patch. **The crash is the cymbal** (key 7) with a long
  `d` (B 52 000), on bar 4's step 11, where R98's tab moves the ride off.
- **Kick and snare play their starter-pack samples** (`sound 1 sample`, `sound 2 sample`), as in
  the reference patch (pull request 145); the hats and the cymbal are code.
- **Tempo 136,** Wikipedia's figure, and the reference patch's. **No swing, no effects lane:**
  the default should sound the same on every host, and effect 4 is Rack's own delay in the
  reference patch but ours on the page.

**What is relied on to ship it, said plainly and not ruled here:** a four-bar drum rhythm,
commonly described in public notation, written out step by step on our own synthesised voices
and CC0 starter samples, with no recording or sample of the record, and credited in the file,
in `CREDITS.md` and in `docs/REFERENCES.md` (R97 to R100). Wikipedia records that "The
copyright owner of 'Amen, Brother', including the Amen break, was the Winstons bandleader,
Richard Lewis Spencer" (R97). No legal conclusion is drawn; Kenny rules (D131 item 7: no
lawyer's view). ⚠ The licence the other six carry (CC0, as `fixtures/beat-four-and-a-fill.beat`)
is not written into the Amen's credit: whether we can dedicate a writing-out of someone else's
rhythm is part of the same ruling.

## Naming and crediting a beat

- **`name`:** a short title in title case, at most 32 bytes, unique in the bank; the menu shows
  it. The file is the name in lower case with hyphens, `.beat`.
- **`credit`:** at most 128 bytes, saying who wrote it, the year, and what it is from:
  `Pocket Instruments contributors, 2026; CC0` for our own; for a beat after another's work, the
  work, the artist, the year and the player, as the Amen's.
- **Every beat after another's work** also gets a row in `docs/REFERENCES.md` (where its
  structure was read) and in `CREDITS.md`.

## Where the bank lives, and how each host offers it

**One set of files, `beats/`, read by one reader, `pib_read`, on every host at run time.** The
core holds no text (§8), so each host carries the files and turns the chosen one into a
snapshot with the beat crate, off the audio thread, then loads it by its ordinary load (a
marker in the queue, §8). No host keeps a converted copy, so no host can drift from the files.

| Host | How it carries `beats/` | How a player loads one |
|---|---|---|
| The page | Served beside it: the page fetches `../beats/bank.txt` and the file chosen. `pages.yml` adds `beats` to what it publishes (today `index.html web samples fixtures`) | A Beats list beside Open and Save (issue 141's controls): the bank's names, then **Empty** |
| Rack | Copied into the plugin as `res/beats/` at build, as `res/samples/` is from `../samples` (`rack/Makefile`'s `PACK`), read with Rack's `asset::plugin` | The module's menu, **Beat** (issue 142's submenu, beside Open and Save): the bank's names. Rack's own **Initialize** gives the empty beat, as today |
| The terminal | Compiled in from `../beats` (as the starter pack is: one binary, no files beside it) | `:bank` lists the bank on the display, numbered; `:bank 3` loads the third; `:new` the empty beat ⚠ (the words are the build's to propose to Kenny) |

**What loading does:** it replaces the whole beat, as any load does: the sixteen patterns, the
chain, the tempo, the swing, the volume, the sounds' sliders and samples, the name and the
credit. It never happens unasked: only the player's choice in a menu, and the first start
below. **How a player gets their own back:** the beat a load replaced is kept once, until the
next load:

- the page keeps it in its storage under a second key, and the Beats list offers **Your last
  beat** at its top;
- Rack puts the load in its undo history, so Ctrl+Z (Edit, Undo) brings the module's beat back
  ⚠ (the build checks that Rack's history restores a module's data as it does for a preset);
- the terminal writes it to `rhythm.previous.beat` beside `rhythm.beat` (or beside the `--beat`
  file), and says so: `:e` opens it.

## The beat a new player opens on (D142)

**What "new" means, on each host, and what is never replaced:**

| Host | New | Not new: the player's beat loads, as today |
|---|---|---|
| The page | Its storage holds no beat under `pocket-instruments/rhythm/beat`, or the storage cannot be read (a private window: nothing is kept anyway) | A beat in storage, of any version; a beat the core refuses (as today: the page starts empty, and the refused bytes stay until the first edit saves over them) |
| Rack | A Rhythm module added from the module browser: its data never carried a `snapshot` | A module from a patch, a duplicate, a paste or a preset (its `snapshot` loads); the user's own default preset (Rack's "Save default") wins, being theirs |
| The terminal | No file at the beat's path (`rhythm.beat`, or `--beat <file>`) | A file there, of any version; a file `pi_load` refuses (left alone, started empty, said, as today) |

**Where its snapshot comes from:** the first line of `beats/bank.txt`, `amen-break.beat`, read by
`pib_read` on that host at that moment, exactly as when a player picks it from the menu. Not a
snapshot built at compile time: a second, generated copy of the beat could fall behind the file;
read at run time there is one file and one reader, and `pib_read` costs microseconds. Each
host's tests hold the snapshot it opened on to `pib_read` of the file, byte for byte.

- **The page:** at load, with no beat stored, it reads the default and loads it like an opened
  file. It is stored at the first save like any beat, from then on the player's.
- **Rack:** the module's constructor queues the default as its pending load; a `snapshot` in
  `dataFromJson` queues the saved beat after it. Loads are taken in order and the last wins, so
  whichever order Rack calls them in, a saved beat ends up loaded, and only a module with none
  plays the Amen. ⚠ Read from the SDK's headers, not tried: `dataFromJson` is "Not called if
  'data' property is not present", and `onAdd` comes "after adding the module to the Engine";
  the build tries a fresh module, a patch, a duplicate and a preset.
- **The terminal:** with no file at the path, it loads the default and writes it there at the
  next save, as it saves any beat.

**An empty instrument, in one gesture:** the **Empty** entry (the page), Rack's **Initialize**
(unchanged, D142 item 4), `:new` (the terminal): the beat `pi_init` gives, every pattern clear,
the chain pattern 1 alone. And, as ever, clearing patterns one by one (function + pattern,
§21).

**Every test and fixture still starts empty, so no trace moves. The engine's specification does
not change.** `pi_init` still builds the empty engine (§21: "every pattern empty, the chain
pattern 1 alone"); the default is a host's act on its start-up path for a person, and on no
other:

- the core's tests, the C++ and Node harnesses, the race harness, the bench and the fuzz call
  `pi_init` and never a host's start-up path;
- the page's automated runs (`make play` and the others that open `rhythm.html` with the `auto` parameter, under
  which `kept()` already reads no storage but for the reload check) start as today, from
  `pi_init`: under `?auto` the default is never loaded, and only a page opened by a person with
  nothing stored loads it;
- Rack's self-test runs its fixtures on a fresh engine (`fresh()`, as Initialize), never the
  constructor's pending load;
- the terminal host's tests build their engine without `main.rs`'s start-up, and a start-up test
  points `--beat` at a file that does not exist in a scratch folder.

## The reference patch

Once Rack plays chains (issue 142), the DRUMS module of `rack/patch/make-showcase.mjs` holds the
four-pattern Amen: the generator reads `beats/amen-break.beat` with `web/pi_beat.wasm`, loads it
into the core's `.wasm`, adds the patch's own throw (effect 4, Rack's Delay, on step 13 of
patterns 1 and 2, where today's one bar has it on step 13) and saves the snapshot into the patch.
So the patch and the bank cannot drift: the same file, with one lane added and said in a Core
Notes line. ⚠ Two throws in four bars, not four, is ours for Kenny's ear. The MELODY module
keeps its one bar: four drum bars under a one-bar melody loop four times.

## Built (issue 164, 2026-11-27): the bank and the default on the three hosts

Built on D141 and D142 **as proposed, not yet ruled by Kenny** (O40). Which beat is the default
is `beats/bank.txt`'s first line and nothing else: no host, test or check names the Amen as the
default. A ruling that moves it changes `bank.txt`, and then `fixtures/beat-amen-break.json` must
name the new first beat (two checks say so: the terminal's and the page's). No file of the
engine changed; `pi_init` is as it was.

**How the files reach each host,** all read by `pib_read` when a beat is wanted (no snapshot is
made at build time, so none can fall behind its file):

| Host | Carried | Read |
|---|---|---|
| The page | Served beside it: `../beats/` from `web/rhythm.html`. `pages.yml` now copies `beats` into a pull request's preview (main's root already held every file); `make hear` serves the repository's root, so it is there | `fetch` with no cache, then `beatRead` (the page's `pib_read`, issue 141) |
| Rack | Copied into the plugin as `res/beats/` by `rack/Makefile`'s `PACK`, as `res/samples/` is; `rack/check-dist.sh` holds the `.vcvplugin` to `bank.txt` | The file read with `asset::plugin`, then `Rhythm::openBeat`, the menu's Open (issue 142) |
| The terminal | Compiled in by `term/build.rs`: every `beats/*.beat` and `bank.txt` with `include_bytes!`, as the files are, as the starter pack is | `beat::from_file`, the path `:e` takes (issue 143) |

**What a player sees** (every look a stand-in ⚠, words only, until Kenny rules one):

| | The page | Rack | The terminal |
|---|---|---|---|
| New | No beat in its storage, or storage it cannot read: it opens on the bank's first beat and says "A new beat: Amen Break, from the bank below. Empty is there too." It is stored at once as the player's beat | A module with no data at all, as the module browser adds one: it opens on the bank's first beat | No file at the beat's path (or no path at all): it opens on the bank's first beat and the footer says `a new beat: Amen Break, from the bank (:bank lists it, :new empties)`; written to the path at the next save |
| Not new: the player's beat, as before | A stored beat of any version, refused or not | A module from a patch, a duplicate, a paste or a preset: it carries data | A file there of any version, read or left alone |
| Load from the bank | The Beats list in the Beat section of the sheet: each beat's name, from the file | The module's menu, **Beat**: each beat's name | `:bank` lists them on the display, numbered; `:bank <n>` loads the n-th |
| The replaced beat, kept once | In the storage's second key, `pocket-instruments/rhythm/previous`, and offered as **Your last beat** (a swap: the beat it replaces is kept in its place) | Rack's undo history: Edit, Undo (Ctrl+Z) | `rhythm.previous.beat` beside the beat file (`<name>.previous.beat` beside a `--beat` file); `:e rhythm.previous.beat` opens it |
| Empty, one gesture | **Empty** in the Beats list | Initialize, unchanged | `:new` |
| A bank beat that does not read | The default: the page starts empty and says "Started empty: the bank's first beat did not open: …"; in the list, that beat is shown with why and cannot be pressed | The default: the module starts empty and Rack's log says why; in the menu, that beat is a line with why | The default: the instrument starts empty, the footer says why, and the whole line is said again on leaving |

**Rack's way in, as built.** The constructor reads `bank.txt`'s first beat beside the starter
pack (two small files; the constructor already reads sixteen). `onAdd` opens it through
`openBeat` and the ordinary queued load, unless `dataFromJson` came first. Read for this
(Rack SDK 2.6.6, `include/engine/Module.hpp`): `dataFromJson` is "Not called if 'data'
property is not present"; `onAdd` is "Called after adding the module to the Engine";
`include/engine/Engine.hpp`: `addModule` "Exclusively locks", so `onAdd` reads no file (the
text was read in the constructor; `pib_read` and a copy into the inbox are microseconds).
**Tried in a headless Rack:** a module in a patch with `"data"` (even `{}`) has `dataFromJson`
called before `onAdd`, so it is not new (selftest.sh: every one of its modules with data, and
it has 48, is not new; the two with no `"data"` are). ⚠ **Not tried:** a module added from the
browser by hand (Rack draws nothing headless), a duplicate, a paste and a preset. A duplicate,
paste and preset carry `"data"` with a snapshot; if Rack called `onAdd` before their
`dataFromJson`, the saved beat is queued after the default and the last load wins. ⚠ A user's
own default preset (Rack's "Save default") with no snapshot in its data would leave a new
module on the default: such a preset is written by `dataToJson`, which always writes the beat
once the module has run.

**Undo in Rack.** A bank load pushes a `history::ModuleChange` whose old JSON carries the
module's data from before the load (`loadFromBank` takes it). Rack's undo gives it to the
widget's `fromJson`, and so to `dataFromJson`, which queues the old snapshot. ⚠ The self-test
does what that reaches (`dataFromJson` of the kept data, on another thread: `Module::toJson`
and `fromJson` need Rack's UI thread, since a `ParamQuantity` reads `APP`); the menu's own
Ctrl+Z is for Kenny to try.

**Empty on the page and in the terminal** is the text `pocket-instruments beat 1` alone, every
line at its default (`docs/BEAT.md`, "The form"), read by `pib_read`: the terminal's tests and
`make play` hold it to the core's own `pi_init` save (seed 1, as each host's), byte for byte.

**Every test still starts empty.** The default is loaded only on a host's start-up path for a
person: the core's tests, the C++, Node and race harnesses, the bench and the fuzz never reach
one; under `?auto` the page opens on the bank only in the three checks of a person's start
(`first`, `kept`, `badbank`), and `make play` fails if any other phase does; Rack's self-test
modules carry data, and the self-test fails if any of them is new; the terminal's tests build
their engines directly, and its start-up test points `--beat` at a missing file in a scratch
folder. `target/report-rust.txt` and every `target/render-*.f32` are the same bytes as before
the first change but for `beat-amen-break`'s own runs (the pull request has the comparison).

**⚠ Readings the build made where D141 and D142 are silent:**

1. **Empty and `:new` keep the beat they replace,** as a bank load does (Rack's Initialize is
   in Rack's undo history too). D142 says only "an empty instrument in one gesture".
2. **A load from the Beats list does not ask first,** as Open does: the replaced beat is kept,
   one press brings it back.
3. **Your last beat is a swap:** loading it keeps the beat it replaces in its place.
4. **The terminal with no beat path at all** (no `$HOME`, no `--beat`) opens on the default and
   saves nowhere, as it saved nowhere before. `:bank <n>` and `:new` then refuse: there is
   nowhere to keep the replaced beat.
5. **A terminal load whose replaced beat cannot be written** to `rhythm.previous.beat` loads
   nothing and says why.
6. **The config file names no other default:** this file does not say it may.
7. **`:bank` shows the list over the display,** as `?` shows the keys; esc closes.
