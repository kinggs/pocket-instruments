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
    draw_at(v, cores, host, true)
}

/// The same with the flicker in the half given (`flicker_lit`): lit, or dark.
fn draw_at(v: &[u8; PI_VIEW_SIZE], cores: u16, host: &Host, flicker: bool) -> Drawn {
    let weave = [0u8; PI_WEAVE_CELLS];
    let s = Scene { view: v, weave: &weave, cores, flicker, host, message: None };
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
    // Since patterns (§21) key 1 here is also the active pattern, whose field flickers: read in
    // the flicker's dark half, where the face is as it was before.
    assert_eq!(main_key(&draw_at(&v, 0, &host(), false).main, 0)[2][0], Cell::Half { top: Tok::Tile, bottom: Tok::Ochre });
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

// ---- patterns: ENGINE.md §21, docs/WORKFLOW.md (issue 143) ----

/// Playing, step 6 under the playhead, effect 6 heard, and the view's pattern bytes: pattern 3
/// plays, second in the chain 1 3, pattern 5 waits; patterns 1, 2 and 6 are used. With pattern
/// held the core lights field (used), stitch (the next chain, which waits) and bar (pattern 3).
fn patterns_view(held: bool) -> [u8; PI_VIEW_SIZE] {
    let mut v = blank();
    v[PI_VIEW_FLAGS] |= PI_VIEW_FLAG_PLAYING | PI_VIEW_FLAG_STEP;
    v[PI_VIEW_STEP] = 5;
    v[PI_VIEW_HEARD] = 6;
    set16(&mut v, PI_VIEW_FX_SOUNDS, 0xffff);
    v[PI_VIEW_PATTERN] = 2;
    v[PI_VIEW_POS] = 1;
    v[PI_VIEW_CHAIN_LEN] = 2;
    v[PI_VIEW_CHAIN] = 2 << 4; // entries 0 and 2
    v[PI_VIEW_NEXT_LEN] = 1;
    v[PI_VIEW_NEXT] = 4;
    set16(&mut v, PI_VIEW_USED, 0b10_0011);
    if held {
        set32(&mut v, PI_VIEW_HELD, 1 << PI_KEY_PATTERN);
        v[PI_VIEW_FLAGS] &= !PI_VIEW_FLAG_SOUNDS;
        set16(&mut v, PI_VIEW_FIELD, 0b10_0011);
        set16(&mut v, PI_VIEW_STITCH, 1 << 4);
        set16(&mut v, PI_VIEW_BAR, 1 << 2);
    }
    v
}

#[test]
fn pattern_held_draws_no_frame_core_or_twill_on_either_screen() {
    let v = patterns_view(true);
    let cores = 1 << 5 | 1 << 1;
    for flicker in [true, false] {
        let d = draw_at(&v, cores, &host(), flicker);
        // Key 6 is under the playhead, sounding, and its effect is heard: none of it shows.
        let k = main_key(&d.main, 5);
        assert_eq!(k[0][0], Cell::Half { top: Tok::Ground, bottom: Tok::Ecru }, "no ochre frame: the frame is the face (used: lit)");
        assert_eq!(k[1][2], Cell::text('w', Tok::Ground, Tok::Ecru), "no core");
        assert!(k[1].iter().all(|c| c.ch() != '╱'), "no twill");
        let p = pocket_key(&d.pocket, 5);
        assert_eq!((bg(p[0]), bg(p[2])), (Tok::Ecru, Tok::Ecru));
        assert!(p.iter().all(|c| c.ch() != '╱'));
        // Key 2 sounds and is used: lit, no core.
        assert_eq!(main_key(&d.main, 1)[1][2], Cell::text('2', Tok::Ground, Tok::Ecru));
        // Unused and not active: dark.
        assert_eq!(main_key(&d.main, 3)[1][2], Cell::text('4', Tok::Ecru, Tok::Tile));
        // No colour: key 6 reads as a lit field (reverse, not bold), with no bold frame.
        let m = mono(&main_key(&d.main, 5).concat());
        assert!(m.iter().all(|(_, a)| !a.bold), "no ochre (bold) anywhere on key 6: {m:?}");
        assert_eq!(m[7], ('w', Attr { reverse: true, ..Attr::default() }));
    }
    // The same view with pattern let go shows them all again: the frame, the core, the twill.
    let d = draw(&patterns_view(false), cores, &host());
    let k = main_key(&d.main, 5);
    assert_eq!(k[0][0], Cell::Half { top: Tok::Ground, bottom: Tok::Ochre });
    assert_eq!(k[1][1], Cell::text('╱', Tok::Fern, Tok::Ground));
    assert_eq!(bg(main_key(&d.main, 1)[1][2]), Tok::Fern, "the core, through the effect");
}

#[test]
fn pattern_held_the_used_are_lit_the_chain_stitched_the_active_barred_and_flickering() {
    let v = patterns_view(true);
    let lit = draw_at(&v, 0, &host(), true);
    let dark = draw_at(&v, 0, &host(), false);
    for d in [&lit, &dark] {
        // Used: patterns 1, 2 and 6, steady in both halves.
        for n in [0, 1, 5] {
            assert_eq!(bg(main_key(&d.main, n)[1][2]), Tok::Ecru, "pattern {} used", n + 1);
            assert_eq!(bg(pocket_key(&d.pocket, n)[2]), Tok::Ecru);
        }
        // The next chain (pattern 5) stitched; the bar ochre under pattern 3.
        assert_eq!(main_key(&d.main, 4)[1][3], Cell::text('▪', Tok::Ecru, Tok::Tile));
        assert_eq!(pocket_key(&d.pocket, 4)[3], Cell::text('▪', Tok::Ecru, Tok::Tile));
        assert_eq!(main_key(&d.main, 2)[2][0].ch(), '▀');
        assert!(matches!(main_key(&d.main, 2)[2][0], Cell::Half { bottom: Tok::Ochre, .. }));
        assert!(pocket_key(&d.pocket, 2)[1..4].iter().all(|c| matches!(c, Cell::Text { under: Some(Tok::Ochre), .. })));
    }
    // Pattern 3 is empty and active: it flickers, lit then dark (⚠ open (empty flicker)'s default).
    assert_eq!(main_key(&lit.main, 2)[1][2], Cell::text('3', Tok::Ground, Tok::Ecru));
    assert_eq!(main_key(&dark.main, 2)[1][2], Cell::text('3', Tok::Ecru, Tok::Tile));
    assert_eq!(bg(pocket_key(&lit.pocket, 2)[2]), Tok::Ecru);
    assert_eq!(bg(pocket_key(&dark.pocket, 2)[2]), Tok::Tile);
    // With no colour the two halves differ: reverse, then not; the bar under it in both.
    let (ml, md) = (mono(&main_key(&lit.main, 2).concat()), mono(&main_key(&dark.main, 2).concat()));
    assert_ne!(ml, md);
    assert_eq!((ml[7].1.reverse, md[7].1.reverse), (true, false));
    assert_eq!(md[10], ('▄', Attr { bold: true, ..Attr::default() }), "the bar, ochre, is bold");
    let (pl, pd) = (mono(&pocket_key(&lit.pocket, 2)), mono(&pocket_key(&dark.pocket, 2)));
    assert_eq!((pl[2].1.reverse, pd[2].1.reverse), (true, false));
    assert!(pd[1..4].iter().all(|(_, a)| a.underline));
    // A used active pattern flickers too: dark in the dark half though the core lit its field.
    let mut v = patterns_view(true);
    v[PI_VIEW_PATTERN] = 0;
    assert_eq!(bg(main_key(&draw_at(&v, 0, &host(), false).main, 0)[1][2]), Tok::Tile);
    assert_eq!(bg(main_key(&draw_at(&v, 0, &host(), true).main, 0)[1][2]), Tok::Ecru);
}

#[test]
fn pattern_latched_by_this_host_shows_the_patterns_as_held_does() {
    // The view's held bit off (as between a tap and the engine's next block); the host's latch on.
    let mut v = patterns_view(true);
    set32(&mut v, PI_VIEW_HELD, 0);
    let mut h = Host::new(Mode::Tap);
    h.view = v;
    let mut r = pi_term::input::Reader::new();
    for i in r.feed(b"8") {
        if let pi_term::input::Input::Key(e) = i {
            h.key(e);
        }
    }
    assert!(h.is_latched(PI_KEY_PATTERN));
    let d = draw_at(&v, 1 << 5, &h, false);
    assert_eq!(main_key(&d.main, 5)[0][0], Cell::Half { top: Tok::Ground, bottom: Tok::Ecru }, "no frame");
    assert_eq!(main_key(&d.main, 2)[1][2], Cell::text('3', Tok::Ecru, Tok::Tile), "the flicker's dark half");
    assert!(d.main.plain().contains("pattern 16"), "the display lists the patterns");
}

#[test]
fn the_flicker_is_a_quarter_second_lit_and_a_quarter_dark_and_still_with_reduced_motion() {
    assert!(flicker_lit(0, false) && flicker_lit(249, false));
    assert!(!flicker_lit(250, false) && !flicker_lit(499, false));
    assert!(flicker_lit(500, false));
    assert!((0..2000).step_by(50).all(|t| flicker_lit(t, true)), "nothing flashes with reduced motion");
}

#[test]
fn the_status_line_says_the_pattern_its_place_in_the_chain_and_the_one_waiting() {
    let mut v = blank();
    let top = |v: &[u8; PI_VIEW_SIZE]| -> (String, String) {
        let d = draw(v, 0, &host());
        (d.main.plain().lines().next().unwrap().into(), d.pocket.plain().lines().next().unwrap().into())
    };
    let (m, p) = top(&v);
    assert!(m.contains("rhythm  pattern 1 ") && !m.contains('/') && !m.contains('→'), "{m}");
    assert!(p.contains("rhythm pattern 1 "), "{p}");
    // Pattern 3, second of a chain of four, pattern 5 waiting.
    v[PI_VIEW_PATTERN] = 2;
    v[PI_VIEW_POS] = 1;
    v[PI_VIEW_CHAIN_LEN] = 4;
    v[PI_VIEW_NEXT_LEN] = 2;
    v[PI_VIEW_NEXT] = 4 | 6 << 4;
    let (m, p) = top(&v);
    assert!(m.contains("rhythm  pattern 3 2/4 → 5 "), "{m}");
    assert!(p.contains("rhythm pattern 3 2/4 → 5 "), "{p}");
    // The pocket size beside write, an effect and a long tempo keeps what fits: the pattern first.
    v[PI_VIEW_FLAGS] |= PI_VIEW_FLAG_WRITE | PI_VIEW_FLAG_PLAYING | PI_VIEW_FLAG_STEP;
    v[PI_VIEW_STEP] = 12;
    v[PI_VIEW_HEARD] = 14;
    let (m, p) = top(&v);
    assert!(m.contains("pattern 3 2/4 → 5") && m.contains(" fx 14 ") && m.contains(" write "), "{m}");
    assert!(p.contains("rhythm pattern 3 ") && p.contains(" write ") && p.contains(" fx 14 ") && p.contains("13"), "{p}");
    // With no colour the words are the same characters (the status line is lettering).
    let d = draw(&v, 0, &host());
    assert!(d.main.plain_mono().lines().next().unwrap().contains("pattern 3 2/4 → 5"));
}

#[test]
fn while_pattern_is_held_the_display_lists_the_sixteen_by_their_keys() {
    let v = patterns_view(true);
    let d = draw(&v, 0, &host());
    let text = d.main.plain();
    for n in 1..=16 {
        assert!(text.contains(&format!("pattern {n}")), "pattern {n}");
    }
    assert!(text.contains("chain 1 3, playing 2 of 2; next 5"), "{text}");
    // Each by its key, the marks as words under the name: in the display's rows 3 to 10.
    let row = |y: usize| -> String { (0..80).map(|x| d.main.at(x, y).ch()).collect() };
    assert!(row(3).contains("1 pattern 1") && row(3).contains("3 pattern 3") && row(3).contains("4 pattern 4"), "{}", row(3));
    let marks = row(4);
    assert!(marks.contains("used chain"), "pattern 1: used and in the chain: {marks}");
    assert_eq!(marks.matches("used").count(), 2, "patterns 1 and 2: {marks}");
    assert!(marks.contains("chain") && !marks.contains("next"), "{marks}");
    assert!(row(6).contains("next") && row(6).contains("used"), "pattern 5 waits, 6 is used: {}", row(6));
    // The playing one: its key lit in ochre (reverse and bold with no colour), its name underlined.
    let x = 1 + 2 * 19 + 1; // the third column's cap: pattern 3, on key 3
    assert_eq!(d.main.at(x, 3), Cell::text('3', Tok::Ground, Tok::Ochre));
    assert_eq!(d.main.at(x, 3).mono().1, Attr { reverse: true, bold: true, ..Attr::default() });
    assert!(matches!(d.main.at(x + 2, 3), Cell::Text { ch: 'p', under: Some(Tok::Ochre), .. }));
    assert!(d.main.at(x + 2, 3).mono().1.underline);
    let y = (0..80).find(|&x| d.main.at(x, 3).ch() == '1').unwrap();
    assert_eq!(d.main.at(y, 3).mono().1, Attr::default(), "another key's cap is plain");
    // The pocket size: the marks' row says the pattern, the chain and the next; no playhead.
    let p = d.pocket.plain();
    let line = p.lines().nth(1).unwrap();
    assert!(line.contains("pattern 3  chain 1 3  next 5"), "{line}");
    assert!(!line.contains('█'), "no playhead mark: {line}");
    // Function held names the pattern its clear empties.
    let mut f = blank();
    f[PI_VIEW_PATTERN] = 6;
    set32(&mut f, PI_VIEW_HELD, 1 << PI_KEY_FUNCTION);
    assert!(draw(&f, 0, &host()).main.plain().contains("clears pattern 7"));
}

#[test]
fn the_pattern_footer_says_how_to_choose_chain_and_copy_in_each_mode() {
    let v = patterns_view(true);
    let foot = |mode: Mode| -> String {
        let mut h = Host::new(mode);
        h.view = v;
        draw(&v, 0, &h).main.plain().lines().nth(23).unwrap().to_string()
    };
    let hold = foot(Mode::Hold);
    assert!(hold.contains("choose, then chain") && hold.contains("p+key copy"), "{hold}");
    let tap = foot(Mode::Tap);
    assert!(tap.contains("shift+key chain") && tap.contains("p, key copy") && tap.trim_end().ends_with("tap"), "{tap}");
}
