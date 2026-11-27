# Credits

Everything Pocket Instruments leans on: who made it, its licence, and exactly what of it is in
this repository. "Nothing" means nothing: we read it, admire it or patch into it, and no file
of it is here.

Our own work is under the MIT licence ([LICENSE](LICENSE)). The licences below belong to their
authors. ⚠ marks a claim not confirmed at its source. Where an entry and
[docs/REFERENCES.md](docs/REFERENCES.md) disagree, the references are the record, with the date
each fact was checked.

## The inspiration

**Teenage Engineering's pocket operators.** <https://teenage.engineering/products/po>

- Author: Teenage Engineering AB.
- Licence: none granted to us. Their products, names, artwork and designs are theirs.
- In the repository: **nothing.** No samples, firmware, photographs, traced artwork, logos or
  guide text beyond short quoted phrases in `docs/REFERENCES.md`.
- What we take: the idea of a pocket-sized sixteen-step instrument, and its workflow (the key
  names, the combinations, the order of doing things), described in our own words in
  `docs/WORKFLOW.md`. The effects are tuned by measuring Kenny's own PO-12; the recordings
  and their reports are kept outside the repository.
- Pocket Instruments is not affiliated with or endorsed by Teenage Engineering. Their names
  are used only to describe. Buy the real ones.

## In the repository

| Source | Author | Licence | What is here | Where the notice lives |
|---|---|---|---|---|
| [stmlib](https://github.com/pichenettes/stmlib), `dsp/filter.h` | Émilie Gillet | MIT | The effects' state-variable filter and its polynomial `tan`, written again in Rust in the core's `bus.rs`. A few lines | `core/stmlib-plaits-MIT.txt`, the head of `bus.rs`, the page's credits, the Rack module's description. The plugin package ships the notice file |
| [Sonic Pi's bundled samples](https://github.com/sonic-pi-net/sonic-pi/blob/dev/etc/samples/README.md) | Freesound authors (below), gathered by the Sonic Pi project | CC0 1.0 ⚠ | Sixteen FLAC files in `samples/`, byte for byte as Sonic Pi ships them, and the same sounds as raw 16-bit audio in `samples/raw/` | [samples/README.md](samples/README.md) |
| [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono), ExtraBold | The JetBrains Mono Project Authors | SIL Open Font License 1.1 | One font file, `design/fonts/JetBrainsMono-ExtraBold.ttf`. Used for the panel and the Rack module. The plugin package ships the font. The page uses the same typeface, loaded from Google Fonts and not from this file | `design/fonts/OFL.txt`, shipped beside the font |

### The starter samples, sample by sample

[samples/README.md](samples/README.md) is the record: each file's key, its Freesound author,
the link to the original, and a checksum. The authors, with thanks: Rodrigo The Mad, Dolfeus,
bdu, TheEndOfACycle, looppool, medetix, SoundCollectah, menegass, Neotone and ani_music.

⚠ CC0 is each uploader's own declaration. Sonic Pi's README says the files are from Freesound
and "have also been placed in the public domain via the Creative Commons 0 License". The
Freesound pages were not opened one by one. CC0 asks for no credit; the authors are listed
because it is right.

### Code that was here and is not now

The notice file is still named `stmlib-plaits-MIT.txt`. Two things it once covered have gone:

- **Plaits** ([pichenettes/eurorack](https://github.com/pichenettes/eurorack),
  `plaits/dsp/fx/sample_rate_reducer.h`, Émilie Gillet, MIT). Effect 1 was first a port of its
  sample-and-hold. Effect 1 is now our own design. Plaits' code is no longer in the core; the
  idea of a grid on a phase accumulator stays.
- **stmlib's soft clip** (`dsp/dsp.h`, MIT, the same notice dated 2012). Effect 2 once used it.
  A plain clip replaced it.

Not affiliated with Émilie Gillet or Mutable Instruments.

## Built for, built against, patched into

| Source | Author | Licence | What is here | What we take |
|---|---|---|---|---|
| [VCV Rack](https://vcvrack.com/) | VCV | See [VCV's plugin licensing page](https://vcvrack.com/manual/PluginLicensing) | Nothing. The plugin is built against the Rack SDK 2.6.6, which stays outside the repository | The host. `docs/RACK-RULES.md` holds our modules to VCV's panel, voltage and plugin guides |
| [Surge XT for Rack](https://github.com/surge-synthesizer/surge-rack) | Surge Synth Team | GPL-3.0 | Nothing. No code, no artwork | Ideas: how modulation shows on a control, and how modules connect to their neighbours |
| [sst-rackhelpers](https://github.com/surge-synthesizer/sst-rackhelpers) | Surge Synth Team | MIT | Nothing yet | Decided: adopt it for neighbour connection. Not done yet. When it is, its notice goes here |
| [MindMeld MixMaster](https://github.com/MarcBoule/MindMeldModular) | MindMeld | GPL-3.0-or-later | Nothing | Nothing. We read its manual to patch into it well. MixMaster cannot spread one polyphonic cable over its tracks, so today it takes VCV Split and sixteen cables; a breakout is proposed, not built (D70) |

Not affiliated with any of them.

## Admired, asked about, or being read

| Source | Author | Licence | What is here | Where it stands |
|---|---|---|---|---|
| [Pocket Operations](https://shittyrecording.studio/) | Paul Wenzel | ⚠ Unread. A bookseller's listing says CC BY; the author's own page was not readable when we tried | Nothing | A pattern book we admire. We want its patterns loadable, fully attributed and linked. Nothing of it enters the repository until its licence is read or its author says yes. ⚠ The note to the author is drafted; we do not know that it has been sent |
| [VCSL](https://github.com/sgossner/VCSL) | Versilian Studios | CC0 | Nothing | To be auditioned by ear for the starter pack, beside ours. Nothing used yet |
| [libpo](https://github.com/ericlewis/libpo) | Eric Lewis | MIT | Nothing | Being read: beats to and from a PO-32 or PO-33 as sound. Whether our own rules let us use it is that work's first question |

## Read for ideas, nothing copied

`docs/REFERENCES.md` names every project read on the way and what each taught, with its
licence and the commit read. Among them: the browser pocket-operator clones (R11, R30), the
Rust-in-Rack plugins Phona, potential and DanTCrusti (R12 to R16b), birdsynth and Glicol for
WebAssembly in an audio worklet (R20), four punch-in effect projects (R55), Andrew Simper's
filter papers (R56), and Bidoo, Bespoke Synth, HetrickCV and others for performance effects and
gates (R57, R59). GPL, MPL and LGPL code among them was read and never ported: the core is one
MIT crate with no dependencies.

## Tools

The core is Rust. The Rack plugin is C++ against the Rack SDK. The page is plain HTML and
JavaScript with the core as WebAssembly. Tests run on Node. None of these are in the repository.

## If we missed you

If something of yours is here without the credit or notice it should carry, or you would rather
not be listed, open an issue and we will put it right.
