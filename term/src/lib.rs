//! Pocket Instruments' terminal host: one command, `pocket-instruments`, with a menu of
//! instruments; today Rhythm (docs/TERMINAL.md; D123, D125). It sends what a finger did and
//! draws the view, from the core's static library through its C ABI.
//!
//! SPDX-License-Identifier: MIT
pub mod audio;
pub mod bank;
pub mod beat;
pub mod ffi;
pub mod host;
pub mod input;
pub mod menu;
pub mod samples;
pub mod screen;
pub mod settings;

/// `--help`, and `?` on the menu: at most 22 lines of at most 78 characters, so it fits the
/// menu's frame (a test holds it to that).
pub const HELP: &str = "\
pocket-instruments: small instruments, played in a terminal.
usage: pocket-instruments [rhythm] [options]   (no instrument: a menu)

  --keys auto|hold|tap  hold: a key is down until it comes up (foot, kitty,
                        Ghostty, WezTerm). tap: a tap latches a function key;
                        with fx latched a tap starts an effect, a second stops
                        it. auto (the default): hold where key-up is real
  --pocket              the 48 by 10 screen, whatever the size
  --no-color            normal, bold and reverse only (also: NO_COLOR set)
  --audio default|pipewire|none   where sound goes; none runs silent
  --beat FILE           the beat (in ~/.local/share/pocket-instruments/)
  --reduced-motion      the weave's shuttle jumps; nothing flashes
  -h, --help   -V, --version

config: ~/.config/pocket-instruments/config.toml, one key = value a line:
  keys = \"auto\"  colour = true  audio = \"default\"  reduced_motion = false
  an option wins over the file, the file over NO_COLOR

rhythm: 1234 qwer asdf zxcv the grid; 7 8 9 sound, pattern, bpm; u i o p
function, fx, play, write; shift+key latches; t g y h sliders; space play;
, . tempo; < > swing; - = volume; m the sounds; ? every key; esc lets go;
: w save, e FILE open, bank [N] a beat, new empty, q leave; ctrl+q quits";
