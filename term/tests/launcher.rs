//! One command with a menu and a config file (D125; docs/TERMINAL.md, "The command, the menu
//! and the config"): the options, the file, which wins, the menu's keys and the intro's cells.
//!
//! SPDX-License-Identifier: MIT
use pi_term::audio::Backend;
use pi_term::host::Mode;
use pi_term::input::{Input, Reader, kitty_bytes, Kind, Key, KeyEvent};
use pi_term::menu::*;
use pi_term::screen::{Grid, Tok, Cell};
use pi_term::settings::*;

fn args(s: &str) -> Result<Args, String> {
    let v: Vec<String> = s.split_whitespace().map(String::from).collect();
    parse_args(&v)
}

#[test]
fn the_command_line_names_an_instrument_before_or_after_its_options() {
    assert_eq!(args("").unwrap().instrument, None, "no instrument: the menu");
    assert_eq!(args("rhythm").unwrap().instrument, Some(0));
    let a = args("--keys tap rhythm --no-color --audio none --reduced-motion --pocket --beat x.beat").unwrap();
    assert_eq!(a.instrument, Some(0));
    assert_eq!(a.keys, Some(Some(Mode::Tap)));
    assert!(a.no_colour && a.reduced && a.pocket);
    assert_eq!(a.audio, Some(Backend::None));
    assert_eq!(a.beat.as_deref(), Some(std::path::Path::new("x.beat")));
    assert_eq!(args("--keys auto").unwrap().keys, Some(None), "auto, asked for");
    assert!(args("-h").unwrap().help && args("--version").unwrap().version);
    assert!(args("sampler").unwrap_err().contains("no instrument called sampler"));
    assert!(args("rhythm rhythm").unwrap_err().contains("one instrument"));
    assert!(args("--keys").unwrap_err().contains("needs a value"));
    assert!(args("--audio jack").unwrap_err().contains("default, pipewire or none"));
    assert!(args("--loud").unwrap_err().contains("unknown option"));
}

#[test]
fn the_config_file_reads_each_key_and_value() {
    let c = parse_config(
        "# mine\n\nkeys = \"hold\"   # foot\ncolour = false\naudio = \"pipewire\"\nreduced_motion = true\n",
        "c.toml",
    )
    .unwrap();
    assert_eq!(c, Config { keys: Some(Some(Mode::Hold)), colour: Some(false), audio: Some(Backend::PipeWire), reduced: Some(true) });
    assert_eq!(parse_config("color = true", "c").unwrap().colour, Some(true), "either spelling");
    assert_eq!(parse_config("keys = \"auto\"", "c").unwrap().keys, Some(None));
    assert_eq!(parse_config("keys=\"tap\"", "c").unwrap().keys, Some(Some(Mode::Tap)), "no spaces");
    assert_eq!(parse_config("audio = \"none\"", "c").unwrap().audio, Some(Backend::None));
    assert_eq!(parse_config("", "c").unwrap(), Config::default(), "an empty file");
    assert_eq!(parse_config("keys = \"a#b\"", "c").unwrap_err(), "c:1: keys = \"a#b\": \"auto\", \"hold\" or \"tap\"", "a # in a string is not a comment");
}

#[test]
fn the_config_file_refuses_what_it_does_not_read_with_the_line() {
    let e = |t: &str| parse_config(t, "config.toml").unwrap_err();
    assert_eq!(e("\nkeys = hold"), "config.toml:2: keys takes a string in double quotes, as \"auto\"");
    assert_eq!(e("keys = \"fast\""), "config.toml:1: keys = \"fast\": \"auto\", \"hold\" or \"tap\"");
    assert_eq!(e("colour = yes"), "config.toml:1: colour takes true or false");
    assert_eq!(e("reduced_motion = \"true\""), "config.toml:1: reduced_motion takes true or false");
    assert_eq!(e("audio = \"jack\""), "config.toml:1: audio = \"jack\": \"default\", \"pipewire\" or \"none\"");
    assert_eq!(e("volume = 3"), "config.toml:1: unknown key volume (keys, colour, audio, reduced_motion)");
    assert_eq!(e("[rhythm]"), "config.toml:1: tables are not read here; one key = value a line");
    assert_eq!(e("keys"), "config.toml:1: not key = value: keys");
}

#[test]
fn a_missing_config_file_is_no_error() {
    let p = std::env::temp_dir().join("pocket-instruments-no-such-dir-125/config.toml");
    assert_eq!(read_config(&p).unwrap(), Config::default());
}

#[test]
fn an_option_wins_over_the_file_and_the_file_over_no_color() {
    let none = Args::default();
    let empty = Config::default();
    let s = resolve(&none, &empty, false);
    assert_eq!((s.keys, s.mono, s.audio, s.reduced), ((None, From::Default), (false, From::Default), (Backend::Default, From::Default), (false, From::Default)));
    assert_eq!(resolve(&none, &empty, true).mono, (true, From::NoColor));
    let file = Config { keys: Some(Some(Mode::Tap)), colour: Some(true), audio: Some(Backend::None), reduced: Some(true) };
    let s = resolve(&none, &file, true);
    assert_eq!(s.mono, (false, From::Config), "the file's colour = true beats NO_COLOR (R82)");
    assert_eq!((s.keys, s.audio, s.reduced), ((Some(Mode::Tap), From::Config), (Backend::None, From::Config), (true, From::Config)));
    let opts = args("--no-color --keys hold --audio pipewire --reduced-motion").unwrap();
    let s = resolve(&opts, &file, false);
    assert_eq!((s.keys, s.mono, s.audio, s.reduced), ((Some(Mode::Hold), From::Option), (true, From::Option), (Backend::PipeWire, From::Option), (true, From::Option)));
    let off = Config { colour: Some(false), ..Config::default() };
    assert_eq!(resolve(&none, &off, false).mono, (true, From::Config));
}

/// The menu fed real bytes, through the reader.
fn feed(m: &mut Menu, bytes: &[u8]) -> Vec<Choice> {
    let mut r = Reader::new();
    r.feed(bytes)
        .into_iter()
        .filter_map(|i| if let Input::Key(e) = i { Some(m.key(e)) } else { None })
        .filter(|c| *c != Choice::Stay)
        .collect()
}

#[test]
fn the_menu_starts_on_its_number_or_enter_in_both_forms() {
    let mut m = Menu::default();
    assert_eq!(feed(&mut m, &kitty_bytes('1', false, Kind::Press)), vec![Choice::Start(0)], "1, as foot sends it");
    assert_eq!(feed(&mut m, &kitty_bytes('1', false, Kind::Release)), vec![], "its release does nothing");
    assert_eq!(feed(&mut m, b"1"), vec![Choice::Start(0)], "1, as tmux sends it");
    assert_eq!(feed(&mut m, b"\r"), vec![Choice::Start(0)], "enter");
    assert_eq!(feed(&mut m, b"\x1b[13u"), vec![Choice::Start(0)], "enter, as foot sends it");
    assert_eq!(feed(&mut m, b"2"), vec![], "no second instrument yet");
    assert_eq!(feed(&mut m, b"x"), vec![]);
}

#[test]
fn the_menu_quits_on_q_and_ctrl_q_and_opens_the_options_on_question_mark() {
    let mut m = Menu::default();
    assert_eq!(feed(&mut m, b"q"), vec![Choice::Quit]);
    assert_eq!(feed(&mut m, b"\x11"), vec![Choice::Quit], "ctrl+q");
    assert_eq!(feed(&mut m, b"\x1b[113;5:1u"), vec![Choice::Quit], "ctrl+q, as foot sends it");
    assert_eq!(feed(&mut m, b"Q"), vec![], "shift+q is not q");
    assert_eq!(feed(&mut m, b"?"), vec![]);
    assert!(m.help, "? opens the options");
    assert_eq!(feed(&mut m, b"1q"), vec![], "keys do nothing under the options");
    assert!(m.help);
    assert_eq!(feed(&mut m, b"\x1b[27u"), vec![]);
    assert!(!m.help, "esc closes them");
    feed(&mut m, b"?");
    feed(&mut m, &kitty_bytes('/', true, Kind::Press));
    assert!(!m.help, "? closes them too");
    assert_eq!(feed(&mut m, b"\x11"), vec![Choice::Quit]);
    let e = KeyEvent { key: Key::Char('1'), shift: false, ctrl: false, alt: false, kind: Kind::Repeat, text: None };
    assert_eq!(m.key(e), Choice::Stay, "a repeat does nothing");
}

fn row(g: &Grid, y: usize) -> String {
    (0..g.w).map(|x| g.at(x, y).ch()).collect::<String>().trim_end().to_string()
}
fn row_mono(g: &Grid, y: usize) -> String {
    (0..g.w).map(|x| g.at(x, y).mono().0).collect::<String>().trim_end().to_string()
}

fn intro<'a>(s: &'a Settings, note: Option<&'a str>, config: Option<(&'a str, bool)>) -> Intro<'a> {
    Intro { settings: s, mode: Mode::Hold, why: "this terminal reports key-up", config, note, version: "0.1.0", help: pi_term::HELP }
}

#[test]
fn the_intro_page_shows_the_menu_the_settings_and_where_each_came_from() {
    let file = Config { audio: Some(Backend::None), ..Config::default() };
    let s = resolve(&args("--reduced-motion").unwrap(), &file, true);
    let i = intro(&s, Some("could not open the sound device"), Some(("/home/k/.config/pocket-instruments/config.toml", true)));
    let g = intro_screen(&Menu::default(), &i);
    assert_eq!((g.w, g.h), (80, 24));
    assert_eq!(row(&g, 0), " pocket instruments  0.1.0");
    assert_eq!(row(&g, 2), "  Small calculator-format instruments, free and open source.");
    assert_eq!(row(&g, 3), "  An homage to Teenage Engineering's pocket operators; not affiliated.");
    assert_eq!(row(&g, 7), "   1  rhythm  a sixteen-step drum machine");
    assert_eq!(row(&g, 12), "    keys    hold (this terminal reports key-up)  default");
    assert_eq!(row(&g, 13), "    sound   none, silent  config");
    assert_eq!(row(&g, 14), "    colour  off  NO_COLOR");
    assert_eq!(row(&g, 15), "    motion  reduced  option");
    assert_eq!(row(&g, 18), "  could not open the sound device");
    assert_eq!(row(&g, 20), "  config /home/k/.config/pocket-instruments/config.toml read");
    assert_eq!(row(&g, 23), " 1 or enter play   ? options   q quit   ctrl+q quit from anywhere");
    // The chosen line is a lit field, ecru with ground lettering; with no colour, reverse.
    assert_eq!(g.at(3, 7), Cell::text('1', Tok::Ground, Tok::Ecru));
    assert_eq!(g.at(51, 7), Cell::blank(Tok::Ecru), "the field runs 50 cells");
    assert!(g.at(3, 7).mono().1.reverse);
    assert!(!g.at(4, 2).mono().1.reverse, "lettering on ground is not reversed");
    assert_eq!(g.at(2, 18), Cell::text('c', Tok::Madder, Tok::Ground), "a note in madder");
    assert_eq!(row_mono(&g, 7), "   1  rhythm  a sixteen-step drum machine", "the words survive no colour");
    assert_eq!(row_mono(&g, 12), "    keys    hold (this terminal reports key-up)  default");
}

#[test]
fn the_intro_says_when_the_config_file_is_not_there_and_shortens_a_long_path() {
    let s = resolve(&Args::default(), &Config::default(), false);
    let g = intro_screen(&Menu::default(), &intro(&s, None, Some(("/c.toml", false))));
    assert_eq!(row(&g, 20), "  config /c.toml not there");
    assert_eq!(row(&g, 18), "", "no note, no line");
    let long = format!("/home/{}/config.toml", "x".repeat(90));
    let g = intro_screen(&Menu::default(), &intro(&s, None, Some((&long, true))));
    let r = row(&g, 20);
    assert!(r.starts_with("  config …") && r.ends_with("config.toml read") && r.chars().count() <= 80, "{r}");
    let asked = resolve(&args("--keys tap").unwrap(), &Config::default(), false);
    let i = Intro { mode: Mode::Tap, why: "as asked", ..intro(&asked, None, None) };
    assert_eq!(row(&intro_screen(&Menu::default(), &i), 12), "    keys    tap  option");
}

#[test]
fn the_pocket_size_keeps_the_name_the_menu_and_the_keys() {
    let s = resolve(&Args::default(), &Config::default(), false);
    let g = intro_pocket(&Menu::default(), &intro(&s, None, None));
    assert_eq!((g.w, g.h), (48, 10));
    assert_eq!(row(&g, 0), " pocket instruments  0.1.0");
    assert_eq!(row(&g, 2), "  1  rhythm  a sixteen-step drum machine");
    assert_eq!(row(&g, 5), " keys hold   sound default");
    assert_eq!(row(&g, 6), " colour on   motion full");
    assert_eq!(row(&g, 9), " 1 play   ? options   q quit");
    assert_eq!(g.at(2, 2), Cell::text('1', Tok::Ground, Tok::Ecru));
    assert_eq!(row(&intro_tiny(30, 3), 0), "pocket instruments needs 48 by", "clipped at the edge");
}

#[test]
fn question_mark_shows_the_help_in_the_frame_at_both_sizes() {
    let lines: Vec<&str> = pi_term::HELP.lines().collect();
    assert!(lines.len() <= 22, "the help fits the main screen's frame: {} lines", lines.len());
    for l in &lines {
        assert!(l.chars().count() <= 78, "too wide for the frame: {l}");
    }
    let s = resolve(&Args::default(), &Config::default(), false);
    let m = Menu { chosen: 0, help: true };
    let g = intro_screen(&m, &intro(&s, None, None));
    assert_eq!(row(&g, 1), format!(" {}", lines[0]));
    assert_eq!(row(&g, lines.len()), format!(" {}", lines[lines.len() - 1]).trim_end());
    assert_eq!(row(&g, 23), " esc or ? back");
    assert_eq!(g.at(0, 1), Cell::blank(Tok::Display), "in the display's frame");
    let p = intro_pocket(&m, &intro(&s, None, None));
    assert_eq!(row(&p, 8), " the rest: pocket-instruments --help", "the pocket size says where the rest is");
    assert_eq!(row(&p, 9), " esc or ? back");
}

#[test]
fn the_menu_lists_every_instrument_and_only_rhythm_today() {
    assert_eq!(INSTRUMENTS.map(|(n, _)| n), ["rhythm"]);
}
