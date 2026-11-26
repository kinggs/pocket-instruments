# Contributing

Contributions are welcome, from people and from agents alike. The same rules hold for both.

One person maintains this project in spare time. Replies may be slow. A small, tested pull
request that follows the rules below gets merged soonest.

## The short version

1. Pick an issue, or open one. Issues labelled `maintainer` need the maintainer's ears, eyes or
   ruling: do not take those. `good first issue` marks a small one.
2. Fork the repository and branch from `main`.
3. Make small commits with plain English messages.
4. Run `make test` before you push.
5. Open a pull request with the template filled in.
6. CI must pass. No second approval is required.
7. The pull request is squash merged.

## Design before build

A change of behaviour is first a change to the documents, then a change to the code. These are
two pull requests.

| Kind | What it changes | What it decides |
|---|---|---|
| Design | `docs/ENGINE.md`, and a new decision in `docs/DECISIONS.md` | What the instrument does, and why |
| Build | Code, fixtures, tests | Nothing |

- A design pull request says what will happen and records what was not chosen.
- A build pull request does what the documents say. Where the documents are silent or wrong,
  do not decide. Open a design issue with the question. If the rest of the work does not
  depend on the answer, build round it and list your reading under Unverified in the pull
  request, marked ⚠.
- A bug fix that makes the code match the documents needs no design step.
- A decision is never rewritten. A later decision overrules it and says why.

### A look

Kenny accepted this way of working on 2026-11-26 (the canvas's start page):

1. **Draw it on [the design canvas](https://claude.ai/artifact/Rswd7LH4BPVAPzA5r1AbxK),** on the instrument's page, or on Loom's
   when every instrument shares it. A board's name starts with *Proposed*, *Ruled* or
   *Built*.
2. **The maintainer rules.**
3. **The ruling goes in `docs/DECISIONS.md`; the rules go in `design/DESIGN.md`.** A
   change to what the engine shows also goes in `docs/ENGINE.md`, as for any design.
4. **One issue analyses and builds it,** the design part first, as above.
5. **A picture of what was built goes back on the canvas, beside the drawing,** named
   *Built*.

Tokens and type come from [Loom, the design system](https://claude.ai/artifact/EzTa2cGzc2duUV25f9Huo1). Where the canvas or Loom
differs from the repository, the repository is right until a decision changes it.

## Rules for every change

**Expected output before the core runs it.** Work a fixture's trace and views out by hand from
`docs/ENGINE.md` and commit them. Then run the core. If the two disagree, `docs/ENGINE.md` says
which is wrong.

**No trace, fixture or pass bar is edited to make a build pass.** If a decision changes an
expected output, work it out again by hand before the code changes, and say which lines moved
and why.

**What exists does not move unless the issue says so.** A change that is not meant to alter
the output alters no byte of it. `AGENTS.md` says how to check.

**A planted fault counts only if it built and a named test failed.** A planted fault is a
bug put in on purpose to show that a test catches it.

**Stable Rust only.** A nightly feature would block every reproducible build.

**The same bits on every host.** The sound path uses only `+ − × ÷` and casts. The core stays
`no_std`, with no allocator and no dependencies. `make test` checks this on what was built,
and holds the Rust, C++ and WebAssembly runs to one report.

**Measured efficiency on the audio path.** A change on the audio path says what it costs a
frame, before and after, measured with `make bench`. Do not guess. To read two builds side
by side, copy the old build's `target/bench` somewhere before your change (say
`target/old/bench`), then run `make pairs OLD=target/old/bench`.

**Facts, intentions and guesses stay distinct.** A fact has a source in `docs/REFERENCES.md`.
Mark anything unverified with ⚠, in documents, comments and pull requests.

**No Teenage Engineering material.** No samples, firmware, photographs, traced artwork, logos
or recordings of the hardware. This project is not affiliated with Teenage Engineering. It
uses their names only to describe.

**Nothing whose licence does not allow it.** The project is MIT. Read a source's licence
before anything of it enters the repository. If you cannot tell, ask in an issue first.

**Credit every source.** Code, an algorithm, a sample or an idea taken from somewhere gets a
row in `docs/REFERENCES.md` and a row in `CREDITS.md`.

**Nothing generic until a second instrument needs it.** There is one instrument today. Do not
build a framework for the next one yet.

**No secrets** in commits, issues or pull requests.

## Checking your work

| Command | What it does | When |
|---|---|---|
| `make build` | Builds the core as a static library and as WebAssembly | Any time |
| `make test` | Runs every fixture through Rust, C++ and Node and compares the reports | Before every push |
| `make web` | Copies the built `.wasm` beside the page. `make test` fails if the committed copy is stale | When the core changed |
| `make bench` | Prints what a frame costs, in nanoseconds | Before and after an audio path change |
| `make fuzz OLD=<an older libpi_core.a>` | Checks that this build and an older one give the same output on random input | When a change must move no byte |
| `make play`, `make listen`, `make p4` | Run the pages in headless Chromium and Firefox (`chromium` and `firefox` on the PATH). They need `make test` first | When `web/` changed |
| `make hear` | Serves the repository on `127.0.0.1:8417`; open `/web/rhythm.html` | To play your change in a browser |
| A pull request's preview | `.github/workflows/pages.yml` puts the page at `https://kinggs.github.io/pocket-instruments/pr-<n>/web/rhythm.html` and comments the link. Not for pull requests from forks | To play it on a phone, which needs `https` for its audio |
| `make -C rack install RACK_DIR=<Rack SDK>` | Builds the plugin and installs it in Rack. Needs the Rack SDK 2.6.6 and `jq` | When `rack/` or the core changed |
| `make -C rack selftest RACK_APP=<Rack>` | Runs Rack headless at three sample rates | Before a Rack pull request |

`make test` needs the pinned Rust toolchain (`rust-toolchain.toml`; `mise install` sets it
up), a C++ compiler, binutils and Node. With `clang++` present it also runs the
ThreadSanitizer check. It passed if it exited 0.

For a change to the Rack plugin, also say in the pull request what you loaded in VCV Rack and
what you heard. No test here can hear it for you.

## Commits and pull requests

- One intent a commit. Say what changed in plain English: "Make the swing knob read in sixteenths".
- Link the issue in the pull request: `Closes #12`.
- Paste the last three lines of `make test` in the pull request.
- List anything you could not verify, marked ⚠.

## AI-assisted contributions

Welcome. Much of this project was built with coding agents.

- Say in the pull request what was generated.
- Say that you ran the tests yourself, and paste the result.
- Keep the `Co-Authored-By` trailers on commits.
- You answer for what you submit, however it was written.

Everyone should read `AGENTS.md` for the map and the commands. `docs/README.md` explains the
words and the codes (D66, O30, R60, P6, §16) the documents use.

## Conduct and security

- [Code of conduct](CODE_OF_CONDUCT.md)
- [Security policy](SECURITY.md): report a vulnerability privately, not in an issue.
