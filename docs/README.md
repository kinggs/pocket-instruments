# The documents

Design comes before a build: a change to `ENGINE.md` and a decision in `DECISIONS.md` first.
Builds decide nothing. The live backlog is the repository's issues, not a file here.

| File | What it holds | How it changes |
|---|---|---|
| `docs/ENGINE.md` | The engine interface: time, the clock, commands, events, the snapshot | Changed by a decision; `core/include/pi_engine.h` and the fixtures change with it |
| `docs/DECISIONS.md` | Every decision, its state and why | Append only. A decision is superseded, not rewritten |
| `docs/PROOFS.md` | What must be proven, the pass bar, the result | Pass bar written before the experiment; result after |
| `docs/REFERENCES.md` | Outside facts we rely on, with source and date checked | Re-check before relying on an old entry |
| `docs/LEARNINGS.md` | Dated notebook: what surprised us, what broke, why | Append only |
| `docs/WORKFLOW.md` | What each press does, in our words; the owner ticks each row on the hardware | Until P8 |
| `docs/VOICES.md` | The sixteen sounds: role, recipe, what the sliders do, and what each was built as | Until each voice passes by ear |
| `docs/RACK-RULES.md` | What any module of ours for VCV Rack is held to: the panel, the voltages, the code | Re-read VCV's manual (R33 to R35) before leaning on an old line |
| `docs/ARCHITECTURE.md` | How the pieces join today, and where a second instrument's files would go | A map, not a framework. Updated when a second instrument is designed |
| `docs/REVIEW.md` | The critical review of the brief: findings, evidence, what to change | A new review is a new section, dated |
| `docs/BRIEF-v0.1.md` | The original brief | Frozen. Never edited |
| `docs/history/PLAN.md` | The plan as it stood on 2026-11-26: the rules, the machine, the lists, the last Handover notes | Frozen. Its paths are as they were then |
| `docs/history/PLAN-HISTORY.md` | Every finished session's list and every older Handover note, word for word | Frozen |

Kept beside what they describe:

| File | What it holds |
|---|---|
| `core/include/pi_engine.h` | The engine's contract in C, for both hosts. Follows `ENGINE.md`; where they disagree, `ENGINE.md` is right |
| `fixtures/README.md` | How a harness runs a fixture. A trace is never edited to make a build pass |
| `design/DESIGN.md` | The look (Loom): tokens, key states, the panel in mm, the phone layout |
| `harness/efficiency-candidates/README.md` | The three attempts at the core's cost a frame (D102) |

Older documents name each other without the `docs/` prefix (`ENGINE.md`, `PLAN.md`). They
were written when every file sat in one folder. The names still say which file is meant.

## Words used here

| Word | What it means |
|---|---|
| The core | The Rust engine in `core/` |
| A host | What runs the core: the Rack module, the web page, a test harness |
| A harness | A test program that runs the core and writes a report (`harness/`) |
| A fixture | A test input with the output it must produce (`fixtures/README.md`) |
| A trace | The list of events a run produced |
| The view | The 64 bytes the core gives a host to draw (`ENGINE.md` §14) |
| A snapshot | The saved beat, as bytes (`ENGINE.md` §8) |
| The weave | The display's animation (`ENGINE.md` §16) |
| The lane | The effects recorded on the steps (`ENGINE.md` §17) |
| The bus | Where the sounds are mixed before the effects (`ENGINE.md` §18) |
| A return | Sound coming back into the module from Rack (`ENGINE.md` §19) |
| A take | A recording of the owner's own hardware. Kept outside the repository |
| A proof | A test whose passing mark is written before it runs (`PROOFS.md`) |
| A pass bar | That passing mark |
| The ladder | The table of proofs at the top of `PROOFS.md` |
| A planted fault | A bug put in on purpose, to show that a test catches it |
| The Probe | A Rack module for testing (`rack/src/Probe.cpp`). Not an instrument |
| The ABI | The numbered version of the C contract in `core/include/pi_engine.h` |
| A frame | One audio sample period |
| A session | One working sitting in the old plan (`docs/history/`) |
| The owner | The maintainer |

The codes:

| Code | What it points at |
|---|---|
| D66 | A decision in `DECISIONS.md` |
| O30 | A row of the Open table at the end of `DECISIONS.md`: an open question |
| R60 | An entry in `REFERENCES.md` |
| P6 | A proof in `PROOFS.md` |
| §16 | A section of `ENGINE.md` |

## The old README's rules

Until 2026-11-26 the folder's README carried six numbered rules. "README rule N" in the
documents and in code comments means these:

1. The brief is frozen.
2. Facts, intentions and guesses stay distinct.
3. A proof has a pass bar before it runs.
4. A failed proof is a result.
5. Nothing generic until a second instrument needs it.
6. No Teenage Engineering material enters the repo.

They still hold. `CONTRIBUTING.md` at the root has today's list.
