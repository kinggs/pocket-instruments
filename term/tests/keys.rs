//! Every key of the shared map, in hold and in tap, from the bytes a terminal sends
//! (docs/TERMINAL.md, "The keys"; D113, D119 items 3 and 5, D122 item 2).
//!
//! SPDX-License-Identifier: MIT
use pi_term::ffi::*;
use pi_term::host::{Action, Host, KEYMAP, Mode, Overlay};
use pi_term::input::{Input, Key, Kind, Reader, kitty_bytes};

fn down(k: u8) -> Cmd {
    Cmd::new(PI_CMD_KEY_DOWN, k, 0, 0)
}
fn up(k: u8) -> Cmd {
    Cmd::new(PI_CMD_KEY_UP, k, 0, 0)
}

/// A view as `pi_init` leaves it, enough for the shortcuts: tempo 120, volume 16, sliders mid.
fn fresh_view() -> [u8; PI_VIEW_SIZE] {
    let mut e = Engine::new(48000, 1);
    let mut ev = [Event::default(); 8];
    e.process(1, &[], &mut ev, None, PI_AUDIO_NONE);
    e.view()
}

struct T {
    host: Host,
    reader: Reader,
}

impl T {
    fn new(mode: Mode) -> T {
        let mut host = Host::new(mode);
        host.view = fresh_view();
        T { host, reader: Reader::new() }
    }
    fn bytes(&mut self, b: &[u8]) -> Vec<Cmd> {
        for i in self.reader.feed(b) {
            match i {
                Input::Key(e) => self.host.key(e),
                Input::FocusOut => self.host.focus_lost(),
                _ => {}
            }
        }
        self.host.take_cmds()
    }
    fn press(&mut self, c: char) -> Vec<Cmd> {
        self.bytes(&kitty_bytes(c, false, Kind::Press))
    }
    fn release(&mut self, c: char) -> Vec<Cmd> {
        self.bytes(&kitty_bytes(c, false, Kind::Release))
    }
    fn shift(&mut self, c: char) -> Vec<Cmd> {
        let mut v = self.bytes(&kitty_bytes(c, true, Kind::Press));
        v.extend(self.bytes(&kitty_bytes(c, true, Kind::Release)));
        v
    }
    fn tap(&mut self, c: char) -> Vec<Cmd> {
        self.bytes(c.to_string().as_bytes())
    }
}

// ---- reading the bytes ----

#[test]
fn the_kitty_protocol_reads_press_repeat_release_shift_and_the_key_s_place() {
    let mut r = Reader::new();
    // As foot sent them (R78).
    let got = r.feed(b"\x1b[113;1:1;113u\x1b[113;1:3u\x1b[55:38;2:1;38u\x1b[57441;2:1u\x1b[113;1:2u");
    let keys: Vec<_> = got.iter().map(|i| match i { Input::Key(k) => (k.key, k.shift, k.kind, k.text), _ => panic!() }).collect();
    assert_eq!(keys[0], (Key::Char('q'), false, Kind::Press, Some('q')));
    assert_eq!(keys[1], (Key::Char('q'), false, Kind::Release, Some('q')));
    assert_eq!(keys[2], (Key::Char('7'), true, Kind::Press, Some('&')));
    assert_eq!(keys[3].0, Key::Other); // left Shift on its own
    assert_eq!(keys[4].2, Kind::Repeat);
    // The base layout key wins: Cyrillic й on the key where q is on a US layout.
    let got = r.feed(b"\x1b[1081::113;1:1u");
    assert!(matches!(got[0], Input::Key(k) if k.key == Key::Char('q')));
    // The query's reply, the device attributes, focus.
    assert_eq!(r.feed(b"\x1b[?15u\x1b[?62;22c\x1b[I\x1b[O"), vec![Input::Flags(15), Input::DeviceAttributes, Input::FocusIn, Input::FocusOut]);
}

#[test]
fn a_sequence_split_across_reads_is_kept_until_it_ends() {
    let mut r = Reader::new();
    assert!(r.feed(b"\x1b[55:38;2").is_empty());
    let got = r.feed(b":1;38u");
    assert!(matches!(got[0], Input::Key(k) if k.key == Key::Char('7') && k.shift));
}

#[test]
fn plain_bytes_are_taps_and_shifted_symbols_are_shift_with_their_key() {
    let mut r = Reader::new();
    let got = r.feed(b"qQ&*(<>?:\x11\x03 ");
    let keys: Vec<_> = got.iter().map(|i| match i { Input::Key(k) => (k.key, k.shift, k.ctrl), _ => panic!() }).collect();
    assert_eq!(
        keys,
        vec![
            (Key::Char('q'), false, false),
            (Key::Char('q'), true, false),
            (Key::Char('7'), true, false),
            (Key::Char('8'), true, false),
            (Key::Char('9'), true, false),
            (Key::Char(','), true, false),
            (Key::Char('.'), true, false),
            (Key::Char('/'), true, false),
            (Key::Char(';'), true, false),
            (Key::Char('q'), false, true),
            (Key::Char('c'), false, true),
            (Key::Char(' '), false, false),
        ]
    );
    // A lone ESC at the end of a read is esc.
    assert!(matches!(r.feed(b"\x1b")[0], Input::Key(k) if k.key == Key::Esc));
}

// ---- hold: the page's logic ----

#[test]
fn hold_every_key_of_the_map_goes_down_and_comes_up() {
    let mut t = T::new(Mode::Hold);
    for (k, c) in KEYMAP.chars().enumerate() {
        assert_eq!(t.press(c), vec![down(k as u8)], "{c}");
        assert_eq!(t.release(c), vec![up(k as u8)], "{c}");
    }
}

#[test]
fn hold_a_held_function_key_and_a_grid_key_are_two_keys_down() {
    let mut t = T::new(Mode::Hold);
    assert_eq!(t.press('7'), vec![down(PI_KEY_SOUND)]);
    assert_eq!(t.press('q'), vec![down(4)]);
    assert_eq!(t.release('q'), vec![up(4)]);
    assert_eq!(t.release('7'), vec![up(PI_KEY_SOUND)]);
    // A repeat is not a second press.
    assert_eq!(t.bytes(&kitty_bytes('q', false, Kind::Repeat)), vec![]);
}

#[test]
fn hold_shift_latches_the_five_and_a_latch_lets_go_after_the_next_key() {
    let mut t = T::new(Mode::Hold);
    for (c, k) in [('7', PI_KEY_SOUND), ('8', PI_KEY_PATTERN), ('9', PI_KEY_BPM), ('u', PI_KEY_FUNCTION)] {
        assert_eq!(t.shift(c), vec![down(k)], "{c}");
        assert!(t.host.is_latched(k));
        assert_eq!(t.press('q'), vec![down(4)]);
        assert_eq!(t.release('q'), vec![up(4), up(k)], "{c} lets go after the next key");
        assert!(!t.host.is_latched(k));
    }
    // Shift again lets it go without a key.
    t.shift('7');
    assert_eq!(t.shift('7'), vec![up(PI_KEY_SOUND)]);
    // Play and write do not latch: Shift and p is p.
    assert_eq!(t.bytes(&kitty_bytes('p', true, Kind::Press)), vec![down(PI_KEY_WRITE)]);
}

#[test]
fn hold_fx_latched_is_down_only_while_a_grid_key_is_and_stays_latched() {
    let mut t = T::new(Mode::Hold);
    assert_eq!(t.shift('i'), vec![], "D78: latching fx holds nothing by itself");
    assert_eq!(t.press('q'), vec![down(PI_KEY_FX), down(4)]);
    assert_eq!(t.release('q'), vec![up(4), up(PI_KEY_FX)]);
    assert!(t.host.is_latched(PI_KEY_FX), "fx stays latched until Shift-pressed again");
    assert_eq!(t.press('w'), vec![down(PI_KEY_FX), down(5)]);
    assert_eq!(t.release('w'), vec![up(5), up(PI_KEY_FX)]);
    assert_eq!(t.shift('i'), vec![]);
    assert!(!t.host.is_latched(PI_KEY_FX));
}

#[test]
fn hold_esc_lets_every_latch_go_and_focus_lost_lets_every_key_go() {
    let mut t = T::new(Mode::Hold);
    t.shift('i');
    t.shift('7');
    assert_eq!(t.bytes(b"\x1b[27u"), vec![up(PI_KEY_SOUND)]);
    assert!(!t.host.is_latched(PI_KEY_FX));
    t.press('q');
    t.press('7');
    assert_eq!(t.bytes(b"\x1b[O"), vec![up(4), up(PI_KEY_SOUND)]);
}

// ---- the one-key shortcuts (D119 item 3, D122 item 2) ----

#[test]
fn the_sliders_move_4096_from_where_the_hand_left_them() {
    let mut t = T::new(Mode::Hold);
    let a = u16_at(&t.host.view, PI_VIEW_A) as u32;
    assert_eq!(t.press('t'), vec![Cmd::new(PI_CMD_PARAM, 0, 0, a + 4096)]);
    assert_eq!(t.bytes(&kitty_bytes('t', false, Kind::Repeat)), vec![Cmd::new(PI_CMD_PARAM, 0, 0, a + 8192)], "a held key repeats");
    assert_eq!(t.release('t'), vec![]);
    assert_eq!(t.press('g'), vec![Cmd::new(PI_CMD_PARAM, 0, 0, a + 4096)]);
    let b = u16_at(&t.host.view, PI_VIEW_B) as u32;
    assert_eq!(t.press('y'), vec![Cmd::new(PI_CMD_PARAM, 1, 0, b + 4096)]);
    assert_eq!(t.press('h'), vec![Cmd::new(PI_CMD_PARAM, 1, 0, b)]);
    for _ in 0..20 {
        t.press('t');
    }
    assert_eq!(t.host.hand[0], Some(65535), "held to the travel");
}

#[test]
fn space_tempo_swing_and_volume() {
    let mut t = T::new(Mode::Hold);
    assert_eq!(t.press(' '), vec![down(PI_KEY_PLAY), up(PI_KEY_PLAY)]);
    assert_eq!(t.release(' '), vec![]);
    assert_eq!(t.press(','), vec![Cmd::new(PI_CMD_TEMPO, 0, 0, 119_000)]);
    assert_eq!(t.press('.'), vec![Cmd::new(PI_CMD_TEMPO, 0, 0, 121_000)]);
    assert_eq!(t.shift(','), vec![Cmd::new(PI_CMD_SWING, 0, 0, 0)], "< at 0 stays 0");
    assert_eq!(t.shift('.')[0], Cmd::new(PI_CMD_SWING, 0, 0, 1));
    // The volume is bpm held and a key, as the page sends it; 16 is the most.
    assert_eq!(t.press('='), vec![]);
    assert_eq!(t.press('-'), vec![down(PI_KEY_BPM), down(14), up(14), up(PI_KEY_BPM)]);
    // The plain bytes too (tap mode, or a terminal with no protocol).
    let mut p = T::new(Mode::Tap);
    assert_eq!(p.tap('<'), vec![Cmd::new(PI_CMD_SWING, 0, 0, 0)]);
    assert_eq!(p.tap('>'), vec![Cmd::new(PI_CMD_SWING, 0, 0, 1)]);
    assert_eq!(p.tap(' '), vec![down(PI_KEY_PLAY), up(PI_KEY_PLAY)]);
}

#[test]
fn the_tempo_is_held_to_60_and_240() {
    let mut t = T::new(Mode::Hold);
    t.host.view[PI_VIEW_TEMPO..PI_VIEW_TEMPO + 4].copy_from_slice(&240_000u32.to_le_bytes());
    assert_eq!(t.press('.'), vec![Cmd::new(PI_CMD_TEMPO, 0, 0, 240_000)]);
    t.host.view[PI_VIEW_TEMPO..PI_VIEW_TEMPO + 4].copy_from_slice(&60_000u32.to_le_bytes());
    assert_eq!(t.press(','), vec![Cmd::new(PI_CMD_TEMPO, 0, 0, 60_000)]);
}

#[test]
fn m_question_colon_esc_and_ctrl_q() {
    let mut t = T::new(Mode::Hold);
    t.press('m');
    assert_eq!(t.host.overlay, Overlay::Sheet);
    t.bytes(b"\x1b[27u");
    assert_eq!(t.host.overlay, Overlay::None);
    t.shift('/');
    assert_eq!(t.host.overlay, Overlay::Helper);
    t.press('q');
    assert_eq!(t.host.take_cmds(), vec![], "keys do nothing under the helper");
    t.shift('/');
    assert_eq!(t.host.overlay, Overlay::None);
    t.shift(';');
    assert_eq!(t.host.overlay, Overlay::Command(String::new()));
    t.bytes(b"\x1b[113;1:1u\x1b[13u");
    assert_eq!(t.host.take_actions(), vec![Action::Quit]);
    t.bytes(b"\x1b[113;5:1u");
    // D125 item 3: Ctrl+Q leaves the program, `:q` only the instrument.
    assert_eq!(t.host.take_actions(), vec![Action::QuitAll], "Ctrl+Q");
    let mut p = T::new(Mode::Tap);
    p.tap(':');
    p.bytes(b"w my.beat\r");
    assert_eq!(p.host.take_actions(), vec![Action::Save(Some("my.beat".into()))]);
    p.bytes(b":e other.beat\r");
    assert_eq!(p.host.take_actions(), vec![Action::Open("other.beat".into())]);
    p.tap('\x11');
    assert_eq!(p.host.take_actions(), vec![Action::QuitAll]);
}

#[test]
fn the_sounds_sheet_switches_code_and_sample_and_the_effects() {
    let mut t = T::new(Mode::Hold);
    t.press('m');
    assert_eq!(t.press('w'), vec![Cmd::new(PI_CMD_VOICE, 5, 0, 1), Cmd::new(PI_CMD_FIRE, 5, 0, 0)]);
    assert_eq!(t.shift('w'), vec![Cmd::new(PI_CMD_FXSOUND, 5, 0, 0)], "every sound goes through at pi_init");
    t.press('m');
    assert_eq!(t.host.overlay, Overlay::None);
}

// ---- tap: D117 item 4, D119 item 5 ----

#[test]
fn tap_a_grid_key_goes_down_and_up_at_once() {
    let mut t = T::new(Mode::Tap);
    for (k, c) in KEYMAP.chars().take(16).enumerate() {
        assert_eq!(t.tap(c), vec![down(k as u8), up(k as u8)]);
    }
    assert_eq!(t.tap('o'), vec![down(PI_KEY_PLAY), up(PI_KEY_PLAY)]);
    assert_eq!(t.tap('p'), vec![down(PI_KEY_WRITE), up(PI_KEY_WRITE)]);
}

#[test]
fn tap_a_function_key_latches_and_lets_go_after_the_next_key_or_a_second_tap() {
    let mut t = T::new(Mode::Tap);
    for (c, k) in [('7', PI_KEY_SOUND), ('8', PI_KEY_PATTERN), ('9', PI_KEY_BPM), ('u', PI_KEY_FUNCTION)] {
        assert_eq!(t.tap(c), vec![down(k)]);
        assert_eq!(t.tap('q'), vec![down(4), up(4), up(k)]);
        assert_eq!(t.tap(c), vec![down(k)]);
        assert_eq!(t.tap(c), vec![up(k)]);
    }
    // Clear is function, then pattern (§13): two latches.
    assert_eq!(t.tap('u'), vec![down(PI_KEY_FUNCTION)]);
    assert_eq!(t.tap('8'), vec![down(PI_KEY_PATTERN)]);
    // Shift latches too, as in hold ('&' is Shift and 7 on a US layout).
    let mut s = T::new(Mode::Tap);
    assert_eq!(s.tap('&'), vec![down(PI_KEY_SOUND)]);
}

#[test]
fn tap_with_fx_latched_a_tap_starts_an_effect_a_second_stops_it_another_switches() {
    let mut t = T::new(Mode::Tap);
    assert_eq!(t.tap('i'), vec![], "fx latched holds nothing by itself (D78)");
    assert_eq!(t.tap('q'), vec![down(PI_KEY_FX), down(4)], "a tap starts effect 5");
    assert_eq!(t.tap('q'), vec![up(4), up(PI_KEY_FX)], "a second tap stops it");
    assert_eq!(t.tap('q'), vec![down(PI_KEY_FX), down(4)]);
    assert_eq!(t.tap('w'), vec![down(5), up(4)], "another key switches; fx stays down");
    assert_eq!(t.tap('\x1b'), vec![up(5), up(PI_KEY_FX)], "esc lets it all go");
    assert!(!t.host.is_latched(PI_KEY_FX));
}

#[test]
fn tap_ignores_releases_and_repeats_that_are_not_real() {
    // herdr answers the query and sends each release at once (R80): tap reads presses only.
    let mut t = T::new(Mode::Tap);
    assert_eq!(t.bytes(b"\x1b[55;;55u\x1b[55;1:3u"), vec![down(PI_KEY_SOUND)]);
    assert_eq!(t.bytes(b"\x1b[113;1:2u"), vec![]);
    assert_eq!(t.bytes(b"\x1b[113;;113u\x1b[113;1:3u"), vec![down(4), up(4), up(PI_KEY_SOUND)]);
}

#[test]
fn foot_s_bytes_for_a_shift_latch_then_a_held_sound() {
    // As foot sent them to the host (R78's probe, the build's end-to-end run).
    let mut t = T::new(Mode::Hold);
    let seq: &[&[u8]] = &[
        b"\x1b[57441;2:1u", b"\x1b[55:38;2:1u", b"\x1b[55:38;2:3u", b"\x1b[57441;1:3u", b"\x1b[119;1:1u", b"\x1b[119;1:3u",
        b"\x1b[122;1:1u", b"\x1b[122;1:3u", b"\x1b[55;1:1u", b"\x1b[115;1:1u", b"\x1b[115;1:3u", b"\x1b[55;1:3u",
    ];
    let mut all = Vec::new();
    for s in seq {
        all.extend(t.bytes(s));
    }
    assert_eq!(
        all,
        vec![down(PI_KEY_SOUND), down(5), up(5), up(PI_KEY_SOUND), down(12), up(12), down(PI_KEY_SOUND), down(9), up(9), up(PI_KEY_SOUND)]
    );
}
