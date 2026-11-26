# AGENTS.md

Read this before you work. The rules are in `CONTRIBUTING.md` and hold for people and agents
alike; this file adds the map, the commands and how to work an issue.

## What this is

Small calculator-format musical instruments, free and open source (MIT). One Rust engine
plays the same bits in VCV Rack and in a browser. Today there is one instrument, Rhythm.

An affectionate, credited, non-commercial homage to Teenage Engineering's pocket operators.
Not affiliated with Teenage Engineering; their names are used only to describe.

## The map

| Path | What it holds |
|---|---|
| `core/` | The engine: one Rust crate, `no_std`, no allocator, no dependencies. Tests in `core/src/tests/`. The C contract is `core/include/pi_engine.h` |
| `rack/` | One VCV Rack plugin, slug `PocketInstruments`. A module an instrument: `src/Rhythm.cpp`. `src/Probe.cpp` is a test module, not an instrument |
| `web/` | One site, a page an instrument: `rhythm.html`. `worklet.js` runs the engine; `pi_core.wasm` is committed so GitHub Pages can serve it |
| `term/` | The terminal host (`docs/TERMINAL.md`): its own crate and Cargo workspace, `pi-rhythm`, on the core's static library through the C ABI. Tests in `term/tests/` |
| `harness/` | The same fixtures through the `.wasm` in Node and through C++ on the static library; the bench, the fuzz, the race harness |
| `fixtures/` | Test vectors and the traces they must produce. Formats: `fixtures/README.md` |
| `samples/` | The starter drum pack: sixteen CC0 files and their origins |
| `design/` | The look: `DESIGN.md`, the panel and the script that draws it. The drawings are on [the design canvas](https://claude.ai/artifact/Rswd7LH4BPVAPzA5r1AbxK); the tokens and rules in [Loom, the design system](https://claude.ai/artifact/EzTa2cGzc2duUV25f9Huo1) |
| `docs/` | The design documents (below) |
| `docs/history/` | `PLAN.md` and `PLAN-HISTORY.md`, frozen as they stood on 2026-11-26. Read-only |

The live backlog is GitHub issues. There is no plan file.

| Document | What it is | How it changes |
|---|---|---|
| `docs/ENGINE.md` | The engine's specification | By a decision; the header and fixtures change with it |
| `docs/DECISIONS.md` | Every decision and why. The Open table is at its end | Append only. A decision is superseded, not rewritten |
| `docs/PROOFS.md` | What must be proven, the pass bar, the result | Bar before the run, result after |
| `docs/REFERENCES.md` | Outside facts, with source and date | Re-check an old entry before leaning on it |
| `docs/LEARNINGS.md` | Dated notebook of what surprised us | Append only |
| `docs/WORKFLOW.md`, `docs/VOICES.md` | What each press does; the sixteen sounds | With the owner's rulings |
| `docs/RACK-RULES.md` | What any Rack module of ours is held to | Re-read VCV's manual first |
| `docs/ARCHITECTURE.md` | How the pieces join, and where a second instrument would go | A map, not a framework |
| `docs/BRIEF-v0.1.md`, `docs/REVIEW.md` | The first brief and its review | The brief is frozen |

## Read first, and no more than this

The documents are large (about 700 KB, and over 1 MB with `docs/history/`). Do not read them
whole.

1. The issue you are working on.
2. This file, and `CONTRIBUTING.md` for the rules. `docs/README.md` explains the words and
   the codes (D66, O30, R60, P6, §16).
3. `docs/ENGINE.md`: the sections the issue names, and §3 and §8 if you are new to the engine.
4. `docs/DECISIONS.md`: the decisions the issue and those sections cite, and the **Open**
   table at its end. From D66 onwards for anything about effects, sliders or Rack's jacks.
5. `docs/PROOFS.md`: the ladder table at its top, and the proof you run, if you run one.

Open the rest only when a line sends you there: `docs/LEARNINGS.md` before repeating a kind
of work it names (planting faults, moving a size bound), `docs/history/` for how a past
session went, `docs/WORKFLOW.md`, `docs/VOICES.md` and `design/DESIGN.md` when the issue
touches them.

**Before touching a look** (a key, the panel, the page, the display, a terminal screen):
read Loom's brand book, the design system's `project/README.md` ([Loom](https://claude.ai/artifact/EzTa2cGzc2duUV25f9Huo1); with the
Artifact tool, action `read`, `path` `project/README.md`), and the page of [the design
canvas](https://claude.ai/artifact/Rswd7LH4BPVAPzA5r1AbxK) for the instrument. Why: the canvas is where the owner rules a look, and
it is ahead of `design/tui/`. A session whose user cannot open them says so and works from
`design/DESIGN.md`. Where they differ, `design/DESIGN.md` and `docs/DECISIONS.md` are right.

## Rules, and how to keep them

`CONTRIBUTING.md` has the rules. These are the ones with a method:

Design comes before a build. A design change edits `docs/ENGINE.md` and adds a decision to
`docs/DECISIONS.md`. A build decides nothing. They are two pull requests: the design one
merges first.

- **Decide nothing in a build.** `docs/ENGINE.md` and the decisions the issue cites are the
  brief. Where the documents are silent or wrong, do not decide. Open a design issue with
  the question. If the rest of the work does not depend on the answer, build round it and
  list your reading under Unverified in the pull request, marked ⚠.
- **Expected output before the core runs it.** Work a fixture's trace and views out by hand
  from `docs/ENGINE.md`, commit them (in `fixtures/designed/` if no core can run them yet;
  no harness reads that folder, and you create it: it is empty between designs and so not
  in the repository), and only then run. If the core disagrees, `docs/ENGINE.md` decides which is wrong.
- **No trace and no pass bar is edited to pass a build.** A failed proof is a result:
  record it. A view changed by the owner's decision is worked again by hand before the
  code, and `fixtures/README.md` says which lines and why.
- **What exists does not move unless the issue says so.** Keep `target/report-rust.txt` and
  a `sha256sum` of `target/render-*.f32` from before the first change, and compare at the
  end. Save the old snapshots (`fixtures/snapshots-v*.txt`) before a new snapshot version.
  Every snapshot version ever written must still load.
- **The same bits on every host.** The sound path uses only `+ − × ÷` and casts. The core
  stays `no_std`, with no allocator and no dependencies.
- **A planted fault counts only if it built and a named test failed.** Run the unplanted
  copy first.
- **Stable Rust only.** A nightly feature would block every reproducible build.
- **Efficiency is measured, not guessed.** A change on the audio path says what it costs a
  frame, before and after: `make bench` for the engine, `perf record` on a headless Rack
  for the module's `process` (the commands the owner used are in `docs/history/PLAN.md`,
  "Machine state").
- **Nothing generic until a second instrument needs it.**
- **No Teenage Engineering material:** no samples, firmware, photos, traced art, logos or
  recordings of the hardware.
- **Facts, intentions and guesses stay distinct.** A fact has a source in
  `docs/REFERENCES.md`. ⚠ marks anything unverified.
- No secrets. Small commits, one intent each.

## Commands that work

Run the commands below from the repository root. `cargo` resolves through mise, which reads
`mise.toml` here: it works anywhere inside the repository and fails outside it with "No
version is set for shim: cargo". For a scratch copy elsewhere, stay inside the repository
and pass `--manifest-path` and `CARGO_TARGET_DIR`.

| Command | What it does |
|---|---|
| `mise install` | Once: the pinned Rust (`rust-toolchain.toml`, `mise.toml`) |
| `make build` | The static library and the `.wasm` |
| `make test` | The gate, about 40 seconds. Runs every fixture through Rust, C++ and Node and compares the three reports byte for byte; then checks the display cells, the Rack jack maths and the rendered audio the same way; then a ThreadSanitizer run if `clang++` is there. Needs a C++ compiler, binutils and Node |
| `make web` | Copies the new `.wasm` to `web/pi_core.wasm`. `make test` fails if the committed copy is stale |
| `make bench` | Nanoseconds a frame of `pi_process`. Pinned to one CPU: `BENCH_CPU=<n>` |
| `make pairs OLD=<a bench>` | This build beside another, ten interleaved rounds |
| `make fuzz OLD=<an older libpi_core.a>` | Differential fuzz: a change that must move no byte runs this |
| `make targets` | The library for all four Rack targets (first run downloads three) |
| `make play`, `make p4`, `make listen` | The pages in headless Chromium and Firefox (`chromium` and `firefox` on the PATH). Run `make test` first |
| `make hear` | Serves the repository on `127.0.0.1:8417`. It prints the address of the listening page (`web/listen.html`); the instrument is at `/web/rhythm.html`. Ctrl+C stops it |
| `make term` | The terminal host, `term/target/release/pi-rhythm`. Needs ALSA's headers (`alsa-lib`, `libasound2-dev`). `make test` runs its tests (`make term-test`) |
| `make -C rack install` | Builds the plugin and installs it in Rack's user folder. Needs the Rack SDK (`RACK_DIR=<path>`) and `jq` |
| `make -C rack selftest` | Rack headless, in a throwaway user folder, at three sample rates. Needs Rack (`RACK_APP=<path>`) |
| `make -C rack dist` | The `.vcvplugin` |

Three things that hold on any machine:

- **Rack only gets a build from `make -C rack install`.** `selftest` installs into a
  throwaway folder. Restart Rack after an install. A module's right-click menu starts with
  `Build <commit>`: compare it with `git log -1`.
- **A headless Rack (`./Rack -h`) exits at once if its stdin is at end of file.** Hold
  stdin open. It also draws nothing: a face is seen in Rack's screenshot mode.
- **The default `RACK_DIR` and `RACK_APP` are one laptop's paths.** Set your own.

The rest of the first machine's notes are in `docs/history/PLAN.md`, "Machine state".

## The pull request flow

1. Branch from `main` (from a fork, if you cannot push here). Never commit to `main`
   directly. A design change and its build are two pull requests.
2. Make the change. Run `make test`. For the Rack module, also `make -C rack selftest`.
3. Open a pull request and fill in the template. Link the issue. If it changes the page,
   put its preview link under "Try it": `https://kinggs.github.io/pocket-instruments/pr-<n>/web/rhythm.html`,
   where `<n>` is the pull request's number. `.github/workflows/pages.yml` publishes it a
   minute or two after each push (not for a pull request from a fork).
4. CI must pass. No second approval is required.
5. Squash merge.

## The owner's alone

Issues labelled `owner` are not an agent's to close. They need:

- **Ears:** whether a sound or an effect is right.
- **Looks:** the panel, the page, the weave. Every look is a stand-in until the owner rules.
- **Recordings of the hardware (takes):** recordings and measurements of the owner's own
  unit. None of them enters the repository.
- **Rulings:** any open row of the Open table, and any ⚠ reading a build left.

An agent may prepare the material for these: a patch to open, a page to hear, a short list
of what to check.

## Playing the instrument

An agent cannot yet be asked to write a beat. There is no readable beat file, no command
line tool, and one pattern only.

Both come with the patterns work (D110). ⚠ The issues for it were still being opened when
this was written: search the issues for "patterns" and for "agent can play".

Until then a beat lives in two places only:

- the binary snapshot the core writes and reads (`docs/ENGINE.md` §8, and §15, §17 and §20
  for versions 2 to 4);
- the test fixtures in `fixtures/`, as commands or key presses with the trace they must
  produce (`fixtures/README.md`).
