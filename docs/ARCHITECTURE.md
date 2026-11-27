# Built for several instruments

**Status (2026-11-26):** there is one instrument, Rhythm. No second instrument is chosen
(`DECISIONS.md`, O30). This document is a map, not a framework. Paths without a folder are
in `docs/`.

**The rule:** nothing is extracted, renamed or made generic until a second instrument asks
for it (`../CONTRIBUTING.md`, "Nothing generic until a second instrument needs it"; D110
item 4). Every "would" below is an intention. ⚠ marks what was inferred from the code's
shape and not read in a decision.

D is a decision and O an open question in `DECISIONS.md`; R is an entry in `REFERENCES.md`;
§ is a section of `ENGINE.md`. `README.md` beside this file explains the other words.

## How the pieces join today

```
finger ─► host ─ KEY_DOWN / KEY_UP / PARAM ─► core: keys ─► edits, FIRE ─► sequencer ─► voices ─► bus, effects ─► mix
                                                │                              ▲
lights, caps, display, weave ◄─ host draws ◄─ view                Rack: returns, FX RETURN, A and B in
```

FIRE: the core's order to start a sound. Returns: sound coming back in from Rack. A and B:
the two sliders.

- **A host sends what a finger did and draws what it is told.** The core holds the workflow
  and says what to light in one 64-byte view. The page and the Rack module read the same
  block, so they cannot differ. The weave is a pure function of that view.
- **The snapshot** (version 4) holds the beat: steps, the effects lane, what each step keeps
  of the sliders, the settings. Every older version still loads.
- **In Rack** sound also comes in: a returned sound takes its own voice's place before the
  effects (`ENGINE.md` §19).
- **A third host, the terminal** (`TERMINAL.md`, D123; built 2026-11-26): its own crate,
  `term/`, linking the same static library as Rack through the same C ABI. One command,
  `pocket-instruments`, opens on a menu of instruments (D125, designed 2026-11-27): the one
  place a list of instruments exists before a second one. The menu is names and a function
  each, not a framework: a second instrument would add its own screens there and a line to it.

## What is shared in fact

These already serve more than one user, or hold nothing of Rhythm's.

| Piece | Where | Why it counts as shared |
|---|---|---|
| The host contract | `core/include/pi_engine.h`, `core/src/abi.rs`, `ENGINE.md` §3 and §8 | One call, `pi_process`: commands in, events and sound out. A view out (`pi_view`). A snapshot (`pi_save`, `pi_load`). The caller owns the memory; only `u32`, `i32` and pointers cross |
| The clock | `core/src/clock.rs`, `ENGINE.md` §2 and §4 | Turns frames into ticks, from a table or from edges. It is the only layer that sees a frame |
| The snapshot's header | `core/src/snapshot.rs`, `core/src/crc.rs` | Magic, version, a `kind` field (1 is the rhythm instrument), payload size, CRC-32 |
| The harnesses | `harness/` | The same fixtures through Rust's tests, C++ on the static library and Node on the `.wasm`, held to one report. The bench, the differential fuzz, the race harness, the render comparison |
| The fixture formats | `fixtures/README.md` | Language-neutral JSON with the trace each must produce |
| The thread handoff | `rack/src/handoff.hpp` | Used by the Probe, the Rhythm module and `harness/race.cpp`. How a host moves things between its audio thread and its other thread |
| The Rack build | `rack/Makefile`, `rack/math.mk`, `rack/selftest.sh`, `rack/check-dist.sh`, CI | One plugin, the core linked in, the maths flags that keep the engine's bits |
| The page's worklet | `web/worklet.js`, `web/pi_core.wasm` | The raw `.wasm` in an AudioWorklet, no glue. `web/rhythm.html`, `web/listen.html` (the listening page) and `web/index.html` (the P4 proof page, not the landing page) all load it |
| The rules | `RACK-RULES.md`, `../AGENTS.md` | Written for any module of ours |

⚠ Shared in use is not the same as free of Rhythm. The contract's commands, the view's 64
bytes and the fixture formats 2 to 4 are written in Rhythm's terms (sixteen keys, two
sliders, an fx key). `rack/src/jacks.hpp` is pure and tested, but it is Rhythm's thirteen
jacks; its volt scaling and trigger pulses are the part a second module would want.
`web/worklet.js` knows the starter pack and effect 4's line. Nobody has tried to pull
these apart.

## What is Rhythm's own

| Piece | Where |
|---|---|
| Its sixteen voices and the starter pack | `core/src/voice.rs`, `VOICES.md`, `samples/` |
| Its workflow: what each key does, what to light | `core/src/keys.rs`, `WORKFLOW.md` |
| Its sequencer: sixteen tracks, sixteen steps, the lane, the repeats ⚠ | `core/src/seq.rs` |
| Its effects and the bus | `core/src/bus.rs`, `ENGINE.md` §17 and §18 |
| Its weave | `core/src/weave.rs`, `ENGINE.md` §16 |
| Its snapshot payload, versions 1 to 4 | `core/src/snapshot.rs` |
| Its Rack module and jacks | `rack/src/Rhythm.cpp`, `rack/src/jacks.hpp`, `rack/test/jacks.cpp` |
| Its panel and look (Loom) | `design/DESIGN.md`, `design/panel.svg`, `design/make_panel.py`, `design/loom/` |
| Its page | `web/rhythm.html` |

⚠ Whether the sequencer is Rhythm's or shared is not decided. It is listed here because
its sizes and its effects lane are Rhythm's.

## Where a second instrument's files would go

Intentions only. `<Name>` and `<name>` stand for an instrument not chosen. D110 item 4
gives the shape: one core crate, one Rack plugin, one site.

| Place | Today | A second instrument would add |
|---|---|---|
| `core/` | One crate, `pi-core`; Rhythm's modules flat in `core/src/` | Its own modules in the same crate, and a new snapshot `kind`. ⚠ Whether Rhythm's files move into a folder of their own is that instrument's design question |
| `rack/src/` | `Rhythm.cpp`, `Probe.cpp`, `plugin.cpp` | `<Name>.cpp`, a second `addModel` in `plugin.cpp`, a second entry in `rack/plugin.json` |
| `rack/res/` | `panel.svg`, copied from `design/` at build time | Its own panel file. ⚠ `panel.svg` has no instrument in its name yet; it would gain one then, not before |
| `web/` | `rhythm.html` | `<name>.html`, on the same worklet if it can be |
| `term/` | `pocket-instruments`, Rhythm's screens and keys, a menu of one (D125) | Its own screens and keys, and a line in the menu |
| `fixtures/` | Rhythm's fixtures, flat | Its own fixtures. ⚠ A prefix or a folder: not decided |
| `design/` | Rhythm's look at the top of the folder | Its own look. ⚠ A folder an instrument is the likely shape; not decided |
| `docs/` | `WORKFLOW.md`, `VOICES.md` describe Rhythm | Its own workflow and voices documents |

The root `index.html` is the landing page. It would list each instrument's page.

## How Rack lays them out

- **One plugin.** Slug `PocketInstruments`. The slug can change until the VCV Library has
  it (O3).
- **A module an instrument.** Today: Rhythm, 24 HP, thirteen jacks.
- **Expanders are their own modules,** sitting to the right of their instrument. None is
  built. Proposed so far: a breakout with a jack a sound (D33, D70), small breakouts for
  the effects (D86, not decided: O20), a lane and chain editor (D110 item 8).
- **Until an expander exists, the jacks on the panel do the work:** sixteen channels on one
  cable for triggers, voices, returns and effects (D33, D62, D86 item 1, D87).
- ⚠ **Rack 3 is expected to rework expanders,** "possibly in an API-breaking way"
  (`REFERENCES.md` R60: forum posts, not a published API). An expander built now may need
  rebuilding.
- **Neighbours:** D40 and D41 chose the Surge Synth Team's MIT helper for connecting to a
  neighbouring module. ⚠ Not built.

## What happens when the second instrument arrives

1. A design issue names the instrument and what it needs from the engine.
2. That design says what is shared, by pointing at code two instruments now use. It edits
   `ENGINE.md` and adds a decision.
3. Only then does a build extract or rename anything, and it moves no byte of Rhythm's
   output (`../AGENTS.md`, "What exists does not move unless the issue says so").
