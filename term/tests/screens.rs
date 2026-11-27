//! Every light of design/DESIGN.md's "A key's lights" on both screens, from the view, in
//! truecolour and with no colour (docs/TERMINAL.md, "What each light is in cells"; D115, D120).
//!
//! SPDX-License-Identifier: MIT
use pi_term::ffi::*;
use pi_term::host::{Host, Mode, Overlay};
use pi_term::screen::*;

/// A view with nothing lit: stopped, write off, volume 16, 120 bpm.
fn blank() -> [u8; PI_VIEW_SIZE] {
    let mut v = [0u8; PI_VIEW_SIZE];
    v[PI_VIEW_TEMPO..PI_VIEW_TEMPO + 4].copy_from_slice(&120_000u32.to_le_bytes());
    v[PI_VIEW_VOLUME] = 16;
    v[PI_VIEW_LENGTH] = 16;
    v[PI_VIEW_FLAGS] = PI_VIEW_FLAG_SOUNDS;
    v
}
fn set16(v: &mut [u8; PI_VIEW_SIZE], at: usize, m: u16) {
    v[at..at + 2].copy_from_slice(&m.to_le_bytes());
}
fn set32(v: &mut [u8; PI_VIEW_SIZE], at: usize, m: u32) {
    v[at..at + 4].copy_from_slice(&m.to_le_bytes());
}

struct Drawn {
    main: Grid,
    pocket: Grid,
}

fn draw(v: &[u8; PI_VIEW_SIZE], cores: u16, host: &Host) -> Drawn {
    let weave = [0u8; PI_WEAVE_CELLS];
    let s = Scene { view: v, weave: &weave, cores, host, message: None };
    Drawn { main: main_screen(&s), pocket: pocket_screen(&s) }
}

/// Key n's 15 cells on the main screen (5 by 3, rows top to bottom) and its 5 on the pocket.
fn main_key(g: &Grid, n: usize) -> [[Cell; 5]; 3] {
    let (x, y) = (2 + (n % 4) * 7, 11 + (n / 4) * 3);
    std::array::from_fn(|r| std::array::from_fn(|c| g.at(x + c, y + r)))
}
fn pocket_key(g: &Grid, n: usize) -> [Cell; 5] {
    let (x, y) = (1 + (n % 4) * 7, 2 + (n / 4) * 2);
    std::array::from_fn(|c| g.at(x + c, y))
}
fn mono(cells: &[Cell]) -> Vec<(char, Attr)> {
    cells.iter().map(|c| c.mono()).collect()
}
fn bg(c: Cell) -> Tok {
    match c {
        Cell::Text { bg, .. } => bg,
        _ => panic!("not text: {c:?}"),
    }
}
fn host() -> Host {
    let mut h = Host::new(Mode::Hold);
    h.view = blank();
    h
}

#[test]
fn an_unlit_key_is_tile_with_its_legend_and_draws_nothing_with_no_colour() {
    let v = blank();
    let d = draw(&v, 0, &host());
    let k = main_key(&d.main, 4);
    assert_eq!(k[0][0], Cell::Half { top: Tok::Ground, bottom: Tok::Tile });
    assert_eq!(k[1][0], Cell::Side { left: Tok::Tile, right: Tok::Tile });
    assert_eq!(k[1][2], Cell::text('q', Tok::Ecru, Tok::Tile), "the legend is the key you press");
    assert_eq!(k[2][2], Cell::Half { top: Tok::Tile, bottom: Tok::Ground });
    let m: Vec<char> = k.iter().flatten().map(|c| c.mono().0).collect();
    assert_eq!(m.iter().collect::<String>(), "       q       ", "with no colour only the legend shows");
    assert_eq!(pocket_key(&d.pocket, 4)[2], Cell::text('q', Tok::Ecru, Tok::Tile));
}

#[test]
fn frame_the_playhead_is_ochre_and_bold() {
    let mut v = blank();
    v[PI_VIEW_FLAGS] |= PI_VIEW_FLAG_PLAYING | PI_VIEW_FLAG_STEP;
    v[PI_VIEW_STEP] = 12;
    let d = draw(&v, 0, &host());
    let k = main_key(&d.main, 12);
    for c in 0..5 {
        assert_eq!(k[0][c], Cell::Half { top: Tok::Ground, bottom: Tok::Ochre });
        assert_eq!(k[2][c], Cell::Half { top: Tok::Ochre, bottom: Tok::Ground });
    }
    assert_eq!(k[1][0], Cell::Side { left: Tok::Ochre, right: Tok::Tile });
    assert_eq!(k[1][4], Cell::Side { left: Tok::Tile, right: Tok::Ochre });
    let a = Attr { bold: true, ..Attr::default() };
    assert_eq!(mono(&[k[0][0], k[1][0], k[1][4], k[2][0]]), vec![('▄', a), ('▌', a), ('▐', a), ('▀', a)]);
    let p = pocket_key(&d.pocket, 12);
    assert_eq!((bg(p[0]), bg(p[4])), (Tok::Ochre, Tok::Ochre));
    assert_eq!(mono(&[p[0]])[0].1, Attr { bold: true, reverse: true, ..Attr::default() });
    // Stopped, or before the first step, there is no playhead (§14, flag 3).
    v[PI_VIEW_FLAGS] &= !PI_VIEW_FLAG_STEP;
    assert_eq!(main_key(&draw(&v, 0, &host()).main, 12)[0][0], Cell::Half { top: Tok::Ground, bottom: Tok::Tile });
}

#[test]
fn field_a_lit_face_is_ecru_and_reversed() {
    let mut v = blank();
    set16(&mut v, PI_VIEW_FIELD, 1 << 4);
    let d = draw(&v, 0, &host());
    let k = main_key(&d.main, 4);
    assert_eq!(k[1][2], Cell::text('q', Tok::Ground, Tok::Ecru));
    assert_eq!(k[0][0], Cell::Half { top: Tok::Ground, bottom: Tok::Ecru }, "the frame takes the face's colour");
    assert_eq!(mono(&[k[1][2]])[0], ('q', Attr { reverse: true, ..Attr::default() }));
    assert_eq!(pocket_key(&d.pocket, 4)[2], Cell::text('q', Tok::Ground, Tok::Ecru));
    // Lit and on the playhead: the frame is told from the field by lightness (D115 item 4).
    v[PI_VIEW_FLAGS] |= PI_VIEW_FLAG_PLAYING | PI_VIEW_FLAG_STEP;
    v[PI_VIEW_STEP] = 4;
    let k = main_key(&draw(&v, 0, &host()).main, 4);
    assert_eq!(k[1][0], Cell::Side { left: Tok::Ochre, right: Tok::Ecru });
    assert_eq!(k[1][0].mono(), ('█', Attr { bold: true, ..Attr::default() }));
}

#[test]
fn core_white_while_sounding_fern_through_the_effect_with_dark_neighbours_on_a_lit_face() {
    let mut v = blank();
    let d = draw(&v, 1 << 4, &host());
    let k = main_key(&d.main, 4);
    assert_eq!(k[1][2], Cell::text('q', Tok::Ground, Tok::White));
    assert_eq!(bg(k[1][1]), Tok::Tile, "on a dark face no flank is needed");
    assert_eq!(k[1][2].mono(), ('q', Attr { reverse: true, bold: true, ..Attr::default() }));
    // Through the effect being heard: fern (heard 6, sound 4 in the effects).
    v[PI_VIEW_HEARD] = 6;
    set16(&mut v, PI_VIEW_FX_SOUNDS, 1 << 4);
    let k = main_key(&draw(&v, 1 << 4, &host()).main, 4);
    assert_eq!(bg(k[1][2]), Tok::Fern);
    // Not in the effects: white though one is heard.
    set16(&mut v, PI_VIEW_FX_SOUNDS, 0);
    assert_eq!(bg(main_key(&draw(&v, 1 << 4, &host()).main, 4)[1][2]), Tok::White);
    // On a lit face the two beside it go dark, for Loom's dark line.
    let mut v = blank();
    set16(&mut v, PI_VIEW_FIELD, 1 << 4);
    let d = draw(&v, 1 << 4, &host());
    let k = main_key(&d.main, 4);
    assert_eq!((bg(k[1][1]), bg(k[1][2]), bg(k[1][3])), (Tok::Ground, Tok::White, Tok::Ground));
    let p = pocket_key(&d.pocket, 4);
    assert_eq!((bg(p[1]), bg(p[2]), bg(p[3])), (Tok::Ground, Tok::White, Tok::Ground));
    // No colour: reverse bold between two cells that are not reversed.
    assert_eq!(mono(&k[1][1..4]).iter().map(|(_, a)| a.reverse).collect::<Vec<_>>(), vec![false, true, false]);
}

#[test]
fn the_core_lasts_half_a_step_from_the_fires_counter() {
    let mut c = Cores::default();
    let mut v = blank();
    assert_eq!(c.update(&v, 0), 0, "the first view only sets the counters");
    v[PI_VIEW_FIRES + 4] = 1;
    assert_eq!(c.update(&v, 1000), 1 << 4);
    // 120 bpm: half a step is 62 ms.
    assert_eq!(c.update(&v, 1061), 1 << 4);
    assert_eq!(c.update(&v, 1063), 0);
    // Held to 40 at 300 bpm and 120 at 30.
    set32(&mut v, PI_VIEW_TEMPO, 300_000);
    v[PI_VIEW_FIRES + 4] = 2;
    c.update(&v, 2000);
    assert_eq!((c.update(&v, 2039), c.update(&v, 2040)), (1 << 4, 0));
}

#[test]
fn twill_the_effect_heard_is_hatched_fern_either_side_of_the_legend_and_hides_the_stitch() {
    let mut v = blank();
    v[PI_VIEW_HEARD] = 5; // effect 5: key 5, index 4
    set16(&mut v, PI_VIEW_STITCH, 1 << 4);
    let d = draw(&v, 0, &host());
    let k = main_key(&d.main, 4);
    let tw = Cell::text('╱', Tok::Fern, Tok::Ground);
    assert_eq!((k[1][1], k[1][3]), (tw, tw));
    assert_eq!(k[1][3].mono(), ('╱', Attr::default()), "hatched with no colour too");
    let p = pocket_key(&d.pocket, 4);
    assert_eq!((p[1], p[3]), (tw, tw));
    assert_ne!(main_key(&d.main, 5)[1][1], tw, "only effect n's key");
}

#[test]
fn stitch_another_sound_s_step_is_a_square_dark_on_ecru_ecru_on_dark() {
    let mut v = blank();
    set16(&mut v, PI_VIEW_STITCH, 1 << 4 | 1 << 5);
    set16(&mut v, PI_VIEW_FIELD, 1 << 5);
    let d = draw(&v, 0, &host());
    assert_eq!(main_key(&d.main, 4)[1][3], Cell::text('▪', Tok::Ecru, Tok::Tile));
    assert_eq!(main_key(&d.main, 5)[1][3], Cell::text('▪', Tok::Ground, Tok::Ecru));
    assert_eq!(main_key(&d.main, 4)[1][3].mono(), ('▪', Attr::default()));
    assert_eq!(main_key(&d.main, 5)[1][3].mono(), ('▪', Attr { reverse: true, ..Attr::default() }));
    assert_eq!(pocket_key(&d.pocket, 4)[3], Cell::text('▪', Tok::Ecru, Tok::Tile));
}

#[test]
fn bar_madder_under_the_chosen_sound_ochre_while_pattern_is_held() {
    let mut v = blank();
    set16(&mut v, PI_VIEW_BAR, 1 << 2);
    let d = draw(&v, 0, &host());
    let k = main_key(&d.main, 2);
    for c in 0..5 {
        assert_eq!(k[2][c], Cell::Half { top: Tok::Tile, bottom: Tok::Madder });
    }
    assert_eq!(k[2][0].mono(), ('▄', Attr::default()));
    // The pocket size underlines the three between the sides (⚠ D123 item 6).
    let p = pocket_key(&d.pocket, 2);
    assert!(p[1..4].iter().all(|c| matches!(c, Cell::Text { under: Some(Tok::Madder), .. })));
    assert!(p[1..4].iter().all(|c| c.mono().1.underline));
    set32(&mut v, PI_VIEW_HELD, 1 << PI_KEY_PATTERN);
    set16(&mut v, PI_VIEW_BAR, 1);
    assert_eq!(main_key(&draw(&v, 0, &host()).main, 0)[2][0], Cell::Half { top: Tok::Tile, bottom: Tok::Ochre });
}

#[test]
fn pressed_is_white_with_a_ground_edge() {
    let mut v = blank();
    set32(&mut v, PI_VIEW_HELD, 1 << 4);
    let d = draw(&v, 0, &host());
    let k = main_key(&d.main, 4);
    assert_eq!(k[1][2], Cell::text('q', Tok::Ground, Tok::White));
    assert_eq!(k[0][0], Cell::Half { top: Tok::Ground, bottom: Tok::Ground });
    assert_eq!(k[1][0], Cell::Side { left: Tok::Ground, right: Tok::White });
    assert_eq!(k[1][2].mono().1, Attr { reverse: true, bold: true, ..Attr::default() });
    assert_eq!(k[0][0].mono().0, ' ', "with no colour the edge is the dark around a bold face");
}

#[test]
fn source_mark_a_dash_for_a_sample_while_sound_is_held_or_latched() {
    let mut v = blank();
    set16(&mut v, PI_VIEW_SAMPLED, 1 << 4);
    let h = host();
    assert_eq!(main_key(&draw(&v, 0, &h).main, 4)[1][1].ch(), ' ', "not shown unless sound is down");
    set32(&mut v, PI_VIEW_HELD, 1 << PI_KEY_SOUND);
    let d = draw(&v, 0, &h);
    assert_eq!(main_key(&d.main, 4)[1][1], Cell::text('-', Tok::Ecru, Tok::Tile));
    assert_eq!(pocket_key(&d.pocket, 4)[1].ch(), '-');
    assert_eq!(main_key(&d.main, 4)[1][1].mono(), ('-', Attr::default()));
}

#[test]
fn every_light_differs_from_an_unlit_key_with_no_colour() {
    // D115 item 2: no state told by hue alone. Each light alone, against the unlit key.
    let off = {
        let d = draw(&blank(), 0, &host());
        (mono(&main_key(&d.main, 4).concat()), mono(&pocket_key(&d.pocket, 4)))
    };
    let mut cases: Vec<(&str, [u8; PI_VIEW_SIZE], u16)> = Vec::new();
    let mut v = blank();
    v[PI_VIEW_FLAGS] |= PI_VIEW_FLAG_PLAYING | PI_VIEW_FLAG_STEP;
    v[PI_VIEW_STEP] = 4;
    cases.push(("frame", v, 0));
    let mut v = blank();
    set16(&mut v, PI_VIEW_FIELD, 1 << 4);
    cases.push(("field", v, 0));
    cases.push(("core", blank(), 1 << 4));
    let mut v = blank();
    v[PI_VIEW_HEARD] = 5;
    cases.push(("twill", v, 0));
    let mut v = blank();
    set16(&mut v, PI_VIEW_STITCH, 1 << 4);
    cases.push(("stitch", v, 0));
    let mut v = blank();
    set16(&mut v, PI_VIEW_BAR, 1 << 4);
    cases.push(("bar", v, 0));
    let mut v = blank();
    set32(&mut v, PI_VIEW_HELD, 1 << 4);
    cases.push(("pressed", v, 0));
    let mut v = blank();
    set32(&mut v, PI_VIEW_HELD, 1 << PI_KEY_SOUND);
    set16(&mut v, PI_VIEW_SAMPLED, 1 << 4);
    cases.push(("source mark", v, 0));
    for (name, v, cores) in cases {
        let d = draw(&v, cores, &host());
        assert_ne!(mono(&main_key(&d.main, 4).concat()), off.0, "main: {name}");
        assert_ne!(mono(&pocket_key(&d.pocket, 4)), off.1, "pocket: {name}");
    }
}

#[test]
fn function_keys_idle_pressed_latched_and_lit() {
    let mut v = blank();
    let mut h = host();
    assert_eq!(fn_state(&v, 0, PI_KEY_SOUND), FnState::Idle);
    set32(&mut v, PI_VIEW_HELD, 1 << PI_KEY_SOUND);
    assert_eq!(fn_state(&v, 0, PI_KEY_SOUND), FnState::Pressed);
    assert_eq!(fn_state(&v, 1 << PI_KEY_SOUND, PI_KEY_SOUND), FnState::Latched);
    v[PI_VIEW_FLAGS] |= PI_VIEW_FLAG_PLAYING | PI_VIEW_FLAG_WRITE;
    assert_eq!(fn_state(&v, 0, PI_KEY_PLAY), FnState::Lit);
    assert_eq!(fn_state(&v, 0, PI_KEY_WRITE), FnState::Lit);
    // Drawn: sound's band madder, fx's fern (D112 item 3), the letter in the face.
    let d = draw(&blank(), 0, &h);
    assert_eq!(d.main.at(39, 11), Cell::Half { top: Tok::Ground, bottom: Tok::Madder });
    assert_eq!(d.main.at(49, 17), Cell::Half { top: Tok::Ground, bottom: Tok::Fern });
    assert_eq!(d.main.at(40, 12), Cell::text('7', Tok::Lilac, Tok::Tile));
    assert_eq!(bg(d.pocket.at(33, 6)), Tok::Fern, "the pocket's fx band is fern, not the board's ecru");
    // Latched: its own face, the letter reversed.
    h.view = blank();
    let mut v = blank();
    set32(&mut v, PI_VIEW_HELD, 1 << PI_KEY_SOUND);
    let mut t = pi_term::input::Reader::new();
    for i in t.feed(&pi_term::input::kitty_bytes('7', true, pi_term::input::Kind::Press)) {
        if let pi_term::input::Input::Key(e) = i {
            h.key(e);
        }
    }
    let d = draw(&v, 0, &h);
    assert_eq!(d.main.at(40, 12), Cell::text('7', Tok::Madder, Tok::Ground));
    assert_eq!(bg(d.main.at(41, 13)), Tok::Madder);
}

#[test]
fn the_display_lists_what_each_key_means_while_a_function_key_is_down() {
    let mut v = blank();
    set32(&mut v, PI_VIEW_HELD, 1 << PI_KEY_SOUND);
    v[PI_VIEW_SEL] = 2;
    let d = draw(&v, 0, &host());
    let text = d.main.plain();
    for name in NAMES {
        assert!(text.contains(name), "{name}");
    }
    assert!(text.contains(" sound "), "the status line's block");
    let mut v = blank();
    set32(&mut v, PI_VIEW_HELD, 1 << PI_KEY_FX);
    let text = draw(&v, 0, &host()).main.plain();
    assert!(text.contains("low sample rate") && text.contains("feedback"));
}

#[test]
fn the_status_line_says_write_the_effect_heard_the_step_and_the_tempo() {
    let mut v = blank();
    v[PI_VIEW_FLAGS] = PI_VIEW_FLAG_PLAYING | PI_VIEW_FLAG_STEP | PI_VIEW_FLAG_WRITE;
    v[PI_VIEW_STEP] = 12;
    v[PI_VIEW_HEARD] = 3;
    let d = draw(&v, 0, &host());
    let top: String = d.main.plain().lines().next().unwrap().into();
    assert!(top.contains("rhythm") && top.contains("pattern 1") && top.contains(" write ") && top.contains(" fx 3 ") && top.contains("step 13") && top.contains("120 bpm"), "{top}");
    let top: String = d.pocket.plain().lines().next().unwrap().into();
    assert!(top.contains("write") && top.contains("13") && top.contains("120"), "{top}");
}

#[test]
fn the_helper_and_the_sheet_open_over_the_instrument() {
    let mut h = host();
    h.overlay = Overlay::Helper;
    let text = draw(&blank(), 0, &h).main.plain();
    assert!(text.contains("1234 qwer asdf zxcv") && text.contains("latch") && text.contains("esc or ? closes"));
    h.overlay = Overlay::Sheet;
    let mut v = blank();
    set16(&mut v, PI_VIEW_SAMPLED, 1);
    set16(&mut v, PI_VIEW_FX_SOUNDS, 0xffff);
    let text = draw(&v, 0, &h).main.plain();
    assert!(text.contains("bass drum") && text.contains("sample") && text.contains("code") && text.contains("through fx"));
}

#[test]
fn colour_is_truecolour_and_no_colour_sends_no_colour() {
    let d = draw(&blank(), 0, &host());
    let colour = Painter::new(false).paint(d.main);
    assert!(colour.contains("48;2;20;19;31"), "ground as truecolour");
    let d = draw(&blank(), 0, &host());
    let plain = Painter::new(true).paint(d.main);
    assert!(!plain.contains("38;2") && !plain.contains("48;2") && !plain.contains("58;2"), "no colour at all");
    // Only what changed is sent the second time.
    let mut p = Painter::new(false);
    p.paint(draw(&blank(), 0, &host()).main);
    assert_eq!(p.paint(draw(&blank(), 0, &host()).main), "\x1b[0m");
}
