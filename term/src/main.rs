//! `pi-rhythm`: Rhythm, played in a terminal (docs/TERMINAL.md).
//!
//! SPDX-License-Identifier: MIT
use pi_term::audio::{self, Backend, Shared};
use pi_term::ffi::*;
use pi_term::host::{Action, Host, Mode};
use pi_term::input::{Input, KITTY_FLAGS, Reader};
use pi_term::{beat, screen};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const HELP: &str = "\
pi-rhythm: Rhythm, a small drum machine, played in a terminal.

usage: pi-rhythm [options]

  --keys auto|hold|tap   hold: a key is down until it comes up (needs a terminal that
                         reports key-up: the kitty keyboard protocol, as foot, kitty,
                         Ghostty, WezTerm). tap: a tap latches a function key; with fx
                         latched a tap starts an effect, a second stops it, another
                         switches. auto (the default): hold where key-up is real
  --pocket               the 48 by 10 screen, whatever the size
  --no-color             normal, bold and reverse only (also: NO_COLOR set)
  --audio default|pipewire|none
                         where sound goes; none runs silent
  --beat FILE            the beat to read and write (default:
                         ~/.local/share/pocket-instruments/rhythm.beat)
  --reduced-motion       the weave's shuttle jumps; nothing flashes
  -h, --help             this
  -V, --version          the version, the commit and the engine's ABI

Keys: 1234 qwer asdf zxcv the grid; 7 8 9 sound, pattern, bpm; u i o p function, fx,
play, write; shift and a function key latches it; t g / y h sliders; space play;
, . tempo; < > swing; - = volume; m the sounds; ? every key; esc lets go; : a command
(w save, e FILE open, q quit); ctrl+q quits.";

struct Options {
    keys: Option<Mode>,
    pocket: bool,
    mono: bool,
    audio: Backend,
    beat: Option<PathBuf>,
    reduced: bool,
}

fn options() -> Result<Options, String> {
    let mut o = Options {
        keys: None,
        pocket: false,
        mono: std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()),
        audio: Backend::Default,
        beat: None,
        reduced: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("pi-rhythm {} (commit {}, engine ABI {})", env!("CARGO_PKG_VERSION"), env!("PI_COMMIT"), unsafe { pi_abi_version() });
                std::process::exit(0);
            }
            "--keys" => {
                o.keys = match value("--keys")?.as_str() {
                    "auto" => None,
                    "hold" => Some(Mode::Hold),
                    "tap" => Some(Mode::Tap),
                    v => return Err(format!("--keys {v}: auto, hold or tap")),
                }
            }
            "--pocket" => o.pocket = true,
            "--no-color" | "--no-colour" => o.mono = true,
            "--reduced-motion" => o.reduced = true,
            "--audio" => {
                o.audio = match value("--audio")?.as_str() {
                    "default" | "alsa" => Backend::Default,
                    "pipewire" => Backend::PipeWire,
                    "none" => Backend::None,
                    v => return Err(format!("--audio {v}: default, pipewire or none")),
                }
            }
            "--beat" => o.beat = Some(PathBuf::from(value("--beat")?)),
            a => return Err(format!("unknown option {a} (see --help)")),
        }
    }
    Ok(o)
}

/// Puts the terminal back however we leave.
fn restore() {
    let mut out = std::io::stdout();
    let _ = out.write_all(format!("\x1b[<u\x1b[?1004l\x1b[0m\x1b[?25h").as_bytes());
    let _ = crossterm::execute!(out, crossterm::terminal::LeaveAlternateScreen);
    let _ = crossterm::terminal::disable_raw_mode();
}

fn main() {
    let o = match options() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("pi-rhythm: {e}");
            std::process::exit(2);
        }
    };
    if unsafe { pi_abi_version() } != PI_ABI_VERSION {
        eprintln!("pi-rhythm: the engine's ABI is {}, this host's {PI_ABI_VERSION}", unsafe { pi_abi_version() });
        std::process::exit(1);
    }
    let beat_path = o.beat.clone().or_else(beat::default_path);
    let loaded = beat_path.as_ref().and_then(|p| beat::read(p).ok());

    // Sound first, so a missing device is said before the screen takes over.
    let shared = Shared::new();
    let mut load_note = None;
    let (_output, sound) = match audio::start(o.audio, 1, shared.clone(), |e| {
        if let Some(b) = &loaded {
            let r = e.load(b);
            if r != PI_OK {
                load_note = Some(format!("the beat file was refused ({r}); starting empty"));
            }
        }
    }) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("pi-rhythm: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = crossterm::terminal::enable_raw_mode() {
        eprintln!("pi-rhythm: not a terminal ({e})");
        std::process::exit(1);
    }
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        prev(info);
    }));
    let mut out = std::io::stdout();
    let _ = crossterm::execute!(out, crossterm::terminal::EnterAlternateScreen);
    // Ask whether the kitty keyboard protocol is on (its query, then the device attributes),
    // hide the cursor, and ask for focus in and out.
    let _ = out.write_all(b"\x1b[?u\x1b[c\x1b[?25l\x1b[?1004h");
    let _ = out.flush();

    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 4096];
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                break;
            }
        }
    });

    // The protocol's answer: its flags before the device attributes, within half a second.
    let mut reader = Reader::new();
    let mut early = Vec::new();
    let mut protocol = false;
    let deadline = Instant::now() + Duration::from_millis(500);
    'ask: while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        let Ok(bytes) = rx.recv_timeout(left) else { break };
        for i in reader.feed(&bytes) {
            match i {
                Input::Flags(_) => protocol = true,
                Input::DeviceAttributes => break 'ask,
                other => early.push(other),
            }
        }
    }
    let herdr = std::env::var_os("HERDR_ENV").is_some_and(|v| v == "1");
    let mode = o.keys.unwrap_or(if protocol && !herdr { Mode::Hold } else { Mode::Tap });
    if protocol {
        let _ = out.write_all(format!("\x1b[>{KITTY_FLAGS}u").as_bytes());
    }
    let why = match (o.keys, protocol, herdr) {
        (Some(_), ..) => "as asked",
        (None, true, false) => "this terminal reports key-up",
        (None, true, true) => "herdr's key-up is not real",
        (None, false, _) => "this terminal reports no key-up",
    };

    let mut host = Host::new(mode);
    // A test aid, not an option: Hyprland's `send_key_state` moves focus to the window and back
    // around every event it sends, and foot says so, which lets every held key go (as the page
    // does on blur). The end-to-end check sets this to hold keys across its events.
    let keep_on_blur = std::env::var_os("PI_RHYTHM_TEST_IGNORE_FOCUS").is_some();
    let mut painter = screen::Painter::new(o.mono);
    let mut cores = screen::Cores::default();
    let t0 = Instant::now();
    let mut message: Option<(String, Instant)> = Some((
        load_note.unwrap_or_else(|| format!("{} keys: {why}. sound: {sound}. ? for every key", if mode == Mode::Hold { "hold" } else { "tap" })),
        Instant::now() + Duration::from_secs(6),
    ));
    let mut last_gen = shared.generation.load(Ordering::Acquire);
    let mut save_due: Option<Instant> = None;
    let mut quit = false;
    let mut pending = early;

    let save_now = |shared: &Shared, path: &PathBuf| -> Result<(), String> {
        shared.want_save.store(true, Ordering::Release);
        let until = Instant::now() + Duration::from_millis(500);
        while Instant::now() < until {
            if let Some(b) = shared.saved.lock().unwrap().take() {
                return beat::write(path, &b).map_err(|e| format!("could not write {}: {e}", path.display()));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        Err("the engine did not answer the save".into())
    };

    while !quit {
        while let Ok(bytes) = rx.try_recv() {
            pending.extend(reader.feed(&bytes));
        }
        host.view = shared.view();
        for i in pending.drain(..) {
            match i {
                Input::Key(e) => host.key(e),
                Input::FocusOut if !keep_on_blur => host.focus_lost(),
                _ => {}
            }
            for c in host.take_cmds() {
                shared.push(c);
            }
        }
        for a in host.take_actions() {
            match a {
                Action::Quit => quit = true,
                Action::Say(s) => message = Some((s, Instant::now() + Duration::from_secs(4))),
                Action::Save(file) => {
                    let path = file.map(PathBuf::from).or_else(|| beat_path.clone());
                    let said = match path {
                        Some(p) => match save_now(&shared, &p) {
                            Ok(()) => format!("saved {}", p.display()),
                            Err(e) => e,
                        },
                        None => "nowhere to save: give a file, :w FILE".into(),
                    };
                    message = Some((said, Instant::now() + Duration::from_secs(4)));
                }
                Action::Open(file) => {
                    let said = match std::fs::read(&file) {
                        Ok(bytes) => {
                            *shared.to_load.lock().unwrap() = Some((bytes, None));
                            let until = Instant::now() + Duration::from_millis(500);
                            let mut r = None;
                            while Instant::now() < until && r.is_none() {
                                std::thread::sleep(Duration::from_millis(2));
                                r = shared.to_load.lock().unwrap().as_ref().and_then(|(_, r)| *r);
                            }
                            *shared.to_load.lock().unwrap() = None;
                            match r {
                                Some(PI_OK) => format!("opened {file}"),
                                Some(code) => format!("{file} is not a beat this engine reads ({code})"),
                                None => "the engine did not answer".into(),
                            }
                        }
                        Err(e) => format!("could not read {file}: {e}"),
                    };
                    host.hand = [None, None];
                    message = Some((said, Instant::now() + Duration::from_secs(4)));
                }
            }
        }
        // Save a second after the beat last changed.
        let g = shared.generation.load(Ordering::Acquire);
        if g != last_gen {
            last_gen = g;
            save_due = Some(Instant::now() + Duration::from_secs(1));
        }
        if save_due.is_some_and(|t| Instant::now() >= t) {
            save_due = None;
            if let Some(p) = &beat_path {
                if let Err(e) = save_now(&shared, p) {
                    message = Some((e, Instant::now() + Duration::from_secs(4)));
                }
            }
        }

        let view = shared.view();
        let now_ms = t0.elapsed().as_millis() as u64;
        let lit = cores.update(&view, now_ms);
        let woven = weave(&view, o.reduced);
        if message.as_ref().is_some_and(|(_, t)| Instant::now() >= *t) {
            message = None;
        }
        let scene = screen::Scene { view: &view, weave: &woven, cores: lit, host: &host, message: message.as_ref().map(|(m, _)| m.as_str()) };
        let (w, h) = crossterm::terminal::size().map(|(w, h)| (w as usize, h as usize)).unwrap_or((80, 24));
        let grid = if w >= 80 && h >= 24 && !o.pocket {
            screen::main_screen(&scene)
        } else if w >= 48 && h >= 10 {
            screen::pocket_screen(&scene)
        } else {
            screen::tiny_screen(w, h)
        };
        let s = painter.paint(grid);
        if s.len() > "\x1b[0m".len() {
            let _ = out.write_all(s.as_bytes());
            let _ = out.flush();
        }
        std::thread::sleep(Duration::from_millis(16));
    }

    // Let every key go, save, and put the terminal back.
    host.let_go();
    host.release_all_keys();
    for c in host.take_cmds() {
        shared.push(c);
    }
    std::thread::sleep(Duration::from_millis(60));
    let saved = beat_path.as_ref().map(|p| save_now(&shared, p));
    restore();
    match saved {
        Some(Err(e)) => eprintln!("pi-rhythm: {e}"),
        Some(Ok(())) => {}
        None => {}
    }
    let x = shared.xruns.load(Ordering::Relaxed);
    if x > 0 {
        eprintln!("pi-rhythm: {x} sound dropouts (xruns) while playing");
    }
}
