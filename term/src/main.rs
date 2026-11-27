//! `pocket-instruments`: the instruments, played in a terminal (docs/TERMINAL.md; D123, D125).
//!
//! SPDX-License-Identifier: MIT
use pi_term::audio::{self, Shared};
use pi_term::ffi::*;
use pi_term::host::{Action, Host, Mode};
use pi_term::input::{Input, KITTY_FLAGS, Reader};
use pi_term::menu::{self, Choice, Intro, Menu};
use pi_term::settings::{self, Settings};
use pi_term::{beat, screen};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const HELP: &str = pi_term::HELP;

/// The terminal, set up once for the menu and every instrument it starts.
struct Term {
    out: std::io::Stdout,
    rx: mpsc::Receiver<Vec<u8>>,
    reader: Reader,
    pending: Vec<Input>,
    mode: Mode,
    why: &'static str,
    painter: screen::Painter,
}

impl Term {
    fn take_input(&mut self) {
        while let Ok(bytes) = self.rx.try_recv() {
            self.pending.extend(self.reader.feed(&bytes));
        }
    }
    fn draw(&mut self, grid: screen::Grid) {
        let s = self.painter.paint(grid);
        if s.len() > "\x1b[0m".len() {
            let _ = self.out.write_all(s.as_bytes());
            let _ = self.out.flush();
        }
    }
}

/// How an instrument was left (D125 item 3).
enum Left {
    Menu,
    Quit,
}

/// Puts the terminal back however we leave.
fn restore() {
    let mut out = std::io::stdout();
    let _ = out.write_all(b"\x1b[<u\x1b[?1004l\x1b[0m\x1b[?25h");
    let _ = crossterm::execute!(out, crossterm::terminal::LeaveAlternateScreen);
    let _ = crossterm::terminal::disable_raw_mode();
}

fn fail(e: impl std::fmt::Display, code: i32) -> ! {
    eprintln!("pocket-instruments: {e}");
    std::process::exit(code);
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = settings::parse_args(&argv).unwrap_or_else(|e| fail(e, 2));
    if args.help {
        println!("{HELP}");
        return;
    }
    if args.version {
        println!("pocket-instruments {} (commit {}, engine ABI {})", env!("CARGO_PKG_VERSION"), env!("PI_COMMIT"), unsafe { pi_abi_version() });
        return;
    }
    if unsafe { pi_abi_version() } != PI_ABI_VERSION {
        fail(format!("the engine's ABI is {}, this host's {PI_ABI_VERSION}", unsafe { pi_abi_version() }), 1);
    }
    let config_path = settings::config_path();
    let config = match &config_path {
        Some(p) => settings::read_config(p).unwrap_or_else(|e| fail(e, 2)),
        None => settings::Config::default(),
    };
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let s = settings::resolve(&args, &config, no_color);

    if let Err(e) = crossterm::terminal::enable_raw_mode() {
        fail(format!("not a terminal ({e})"), 1);
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
    let mode = s.keys.0.unwrap_or(if protocol && !herdr { Mode::Hold } else { Mode::Tap });
    if protocol {
        let _ = out.write_all(format!("\x1b[>{KITTY_FLAGS}u").as_bytes());
    }
    let why = match (s.keys.0, protocol, herdr) {
        (Some(_), ..) => "as asked",
        (None, true, false) => "this terminal reports key-up",
        (None, true, true) => "herdr's key-up is not real",
        (None, false, _) => "this terminal reports no key-up",
    };
    let mut t = Term { out, rx, reader, pending: early, mode, why, painter: screen::Painter::new(s.mono.0) };
    let mut report = Vec::new();

    match args.instrument {
        Some(_) => {
            if let Err(e) = rhythm(&mut t, &s, false, &mut report) {
                restore();
                fail(e, 1);
            }
        }
        None => {
            let path = config_path.as_ref().map(|p| p.display().to_string());
            let there = config_path.as_ref().is_some_and(|p| p.exists());
            let mut m = Menu::default();
            let mut note: Option<String> = None;
            'menu: loop {
                t.take_input();
                for i in std::mem::take(&mut t.pending) {
                    let Input::Key(e) = i else { continue };
                    match m.key(e) {
                        Choice::Stay => {}
                        Choice::Quit => break 'menu,
                        Choice::Start(_) => {
                            t.painter.invalidate();
                            match rhythm(&mut t, &s, true, &mut report) {
                                Ok(Left::Quit) => break 'menu,
                                Ok(Left::Menu) => note = None,
                                Err(e) => note = Some(e),
                            }
                            t.painter.invalidate();
                        }
                    }
                }
                let intro = Intro {
                    settings: &s,
                    mode: t.mode,
                    why: t.why,
                    config: path.as_deref().map(|p| (p, there)),
                    note: note.as_deref(),
                    version: env!("CARGO_PKG_VERSION"),
                    help: HELP,
                };
                let (w, h) = crossterm::terminal::size().map(|(w, h)| (w as usize, h as usize)).unwrap_or((80, 24));
                let grid = if w >= 80 && h >= 24 && !s.pocket {
                    menu::intro_screen(&m, &intro)
                } else if w >= 48 && h >= 10 {
                    menu::intro_pocket(&m, &intro)
                } else {
                    menu::intro_tiny(w, h)
                };
                t.draw(grid);
                std::thread::sleep(Duration::from_millis(16));
            }
        }
    }
    restore();
    for line in report {
        eprintln!("pocket-instruments: {line}");
    }
}

/// Rhythm, until it is left. Lines for the terminal after it is put back go in `report`.
fn rhythm(t: &mut Term, s: &Settings, from_menu: bool, report: &mut Vec<String>) -> Result<Left, String> {
    let beat_path: Option<PathBuf> = s.beat.clone().or_else(beat::default_path);
    let loaded = beat_path.as_ref().and_then(|p| beat::read(p).ok());
    let shared = Shared::new();
    let mut load_note = None;
    let (output, sound) = audio::start(s.audio.0, 1, shared.clone(), |e| {
        if let Some(b) = &loaded {
            let r = e.load(b);
            if r != PI_OK {
                load_note = Some(format!("the beat file was refused ({r}); starting empty"));
            }
        }
    })?;

    let mut host = Host::new(t.mode);
    // A test aid, not an option: Hyprland's `send_key_state` moves focus to the window and back
    // around every event it sends, and foot says so, which lets every held key go (as the page
    // does on blur). The end-to-end check sets this to hold keys across its events.
    let keep_on_blur = std::env::var_os("PI_RHYTHM_TEST_IGNORE_FOCUS").is_some();
    let mut cores = screen::Cores::default();
    let t0 = Instant::now();
    let leave = if from_menu { ":q menu" } else { ":q quits" };
    let mode_word = if t.mode == Mode::Hold { "hold" } else { "tap" };
    // The status line has 78 cells: the sound's name goes when the line would not fit.
    let opening = [
        format!("{mode_word} keys: {}. sound: {sound}. ? every key, {leave}", t.why),
        format!("{mode_word} keys: {}. ? every key, {leave}", t.why),
    ]
    .into_iter()
    .find(|m| m.chars().count() <= 78)
    .unwrap_or_else(|| format!("{mode_word} keys. ? every key, {leave}"));
    let mut message: Option<(String, Instant)> = Some((
        load_note.unwrap_or(opening),
        Instant::now() + Duration::from_secs(6),
    ));
    let mut last_gen = shared.generation.load(Ordering::Acquire);
    let mut save_due: Option<Instant> = None;
    let mut left: Option<Left> = None;

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

    while left.is_none() {
        t.take_input();
        host.view = shared.view();
        for i in std::mem::take(&mut t.pending) {
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
                Action::Quit => left = Some(if from_menu { Left::Menu } else { Left::Quit }),
                Action::QuitAll => left = Some(Left::Quit),
                Action::Say(m) => message = Some((m, Instant::now() + Duration::from_secs(4))),
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
        let woven = weave(&view, s.reduced.0);
        if message.as_ref().is_some_and(|(_, t)| Instant::now() >= *t) {
            message = None;
        }
        let scene = screen::Scene { view: &view, weave: &woven, cores: lit, host: &host, message: message.as_ref().map(|(m, _)| m.as_str()) };
        let (w, h) = crossterm::terminal::size().map(|(w, h)| (w as usize, h as usize)).unwrap_or((80, 24));
        let grid = if w >= 80 && h >= 24 && !s.pocket {
            screen::main_screen(&scene)
        } else if w >= 48 && h >= 10 {
            screen::pocket_screen(&scene)
        } else {
            screen::tiny_screen(w, h)
        };
        t.draw(grid);
        std::thread::sleep(Duration::from_millis(16));
    }

    // Let every key go, save, and stop the sound.
    host.let_go();
    host.release_all_keys();
    for c in host.take_cmds() {
        shared.push(c);
    }
    std::thread::sleep(Duration::from_millis(60));
    if let Some(p) = &beat_path {
        if let Err(e) = save_now(&shared, p) {
            report.push(e);
        }
    }
    drop(output);
    let x = shared.xruns.load(Ordering::Relaxed);
    if x > 0 {
        report.push(format!("{x} sound dropouts (xruns) while playing rhythm"));
    }
    Ok(left.unwrap_or(Left::Quit))
}
