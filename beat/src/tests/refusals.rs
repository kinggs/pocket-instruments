//! Every way a file can be wrong, with its line, its column and what the line takes
//! (`docs/BEAT.md`: "Strict on purpose: an agent's slip is caught at once, not played wrong").
//! Each message is typed in here in full.

use super::*;
use crate::{ERR_NEWER, ERR_ROOM, ERR_TEXT};

const H: &str = "pocket-instruments beat 1\n";

fn says(text: &str, code: i32, msg: &str) {
    assert_eq!(refusal(text), (code, msg.to_string()), "{text}");
}

#[test]
fn the_six_the_issue_names() {
    says("pocket-instruments beat 2\ntempo 99\n", ERR_NEWER, "line 1: this is beat 2; this build reads beat 1");
    says(
        &format!("{H}tempo 124\nbpm 120\n"),
        ERR_TEXT,
        "line 3, column 1: unknown word 'bpm'; a line starts with name, credit, tempo, swing, volume, seed, chosen, sliders, sound, own, value, chain, pattern, fx, or a key",
    );
    says(&format!("{H}value a 1 1\n\npattern 1\n1  kick  a...a...a...a..\n"), ERR_TEXT, "line 5, column 10: a row takes sixteen steps; this has 15");
    says(&format!("{H}\npattern 1\n1 snare x...............\n"), ERR_TEXT, "line 4, column 3: key 1 is the kick, not the snare");
    says(
        &format!("{H}value a 70000 1\n"),
        ERR_TEXT,
        "line 2, column 9: value takes a letter (a to w or A to W) and A and B, each 0 to 65535; found '70000'",
    );
    says(&format!("{H}\npattern 1\n1 kick ..c.............\n"), ERR_TEXT, "line 4, column 10: no value line gives the letter c");
}

#[test]
fn the_first_line() {
    let starts = "line 1: a beat file starts with 'pocket-instruments beat 1'";
    says("", ERR_TEXT, starts);
    says("\n", ERR_TEXT, starts);
    says("tempo 100\n", ERR_TEXT, starts);
    says("# a comment first\npocket-instruments beat 1\n", ERR_TEXT, starts);
    says("pocket-instruments beat\n", ERR_TEXT, starts);
    says("pocket-instruments beat 1 2\n", ERR_TEXT, starts);
    says("\u{feff}pocket-instruments beat 1\n", ERR_TEXT, "line 1, column 1: a character the beat file does not use outside name, credit and comments");
    says("pocket-instruments beat 0\n", ERR_TEXT, "line 1, column 25: a beat file starts with 'pocket-instruments beat 1'; found beat '0'");
    says("pocket-instruments beat one\n", ERR_TEXT, "line 1, column 25: a beat file starts with 'pocket-instruments beat 1'; found beat 'one'");
    says("pocket-instruments beat 4294967295\n", ERR_NEWER, "line 1: this is beat 4294967295; this build reads beat 1");
    says(&format!("{H}{H}"), ERR_TEXT, "line 2, column 1: 'pocket-instruments beat 1' comes once, on line 1");
}

#[test]
fn the_beat_s_own_lines() {
    says(&format!("{H}tempo 301\n"), ERR_TEXT, "line 2, column 7: tempo takes 30 to 300, up to three decimals; found '301'");
    says(&format!("{H}tempo 29.999\n"), ERR_TEXT, "line 2, column 7: tempo takes 30 to 300, up to three decimals; found '29.999'");
    says(&format!("{H}tempo 97.5000\n"), ERR_TEXT, "line 2, column 7: tempo takes 30 to 300, up to three decimals; found '97.5000'");
    says(&format!("{H}tempo 97.\n"), ERR_TEXT, "line 2, column 7: tempo takes 30 to 300, up to three decimals; found '97.'");
    says(&format!("{H}tempo\n"), ERR_TEXT, "line 2: tempo takes 30 to 300, up to three decimals");
    says(&format!("{H}tempo 100 110\n"), ERR_TEXT, "line 2, column 11: tempo takes 30 to 300, up to three decimals; found more: '110'");
    says(&format!("{H}tempo 100\ntempo 101\n"), ERR_TEXT, "line 3, column 1: tempo comes once; it was given on line 2");
    says(&format!("{H}swing 49\n"), ERR_TEXT, "line 2, column 7: swing takes 0 to 48 (of a step's 96 ticks); found '49'");
    says(&format!("{H}volume 0\n"), ERR_TEXT, "line 2, column 8: volume takes 1 to 16; found '0'");
    says(&format!("{H}seed 0\n"), ERR_TEXT, "line 2, column 6: seed takes 1 to 4294967295; found '0'");
    says(&format!("{H}seed 4294967296\n"), ERR_TEXT, "line 2, column 6: seed takes 1 to 4294967295; found '4294967296'");
    says(&format!("{H}seed -1\n"), ERR_TEXT, "line 2, column 6: seed takes 1 to 4294967295; found '-1'");
    says(&format!("{H}chosen 17\n"), ERR_TEXT, "line 2, column 8: chosen takes a key, 1 to 16; found '17'");
    says(&format!("{H}sliders 1\n"), ERR_TEXT, "line 2: sliders takes A and B, each 0 to 65535");
    says(&format!("{H}sliders 1 65536\n"), ERR_TEXT, "line 2, column 11: sliders takes A and B, each 0 to 65535; found '65536'");
    says(&format!("{H}name\n"), ERR_TEXT, "line 2: name takes text after a space, to the end of the line");
    says(&format!("{H}credit   \n"), ERR_TEXT, "line 2: credit takes text after a space, to the end of the line");
    says(&format!("{H}name#1\n"), ERR_TEXT, "line 2, column 1: name takes text after a space, to the end of the line");
    says(&format!("{H}name a\u{1}b\n"), ERR_TEXT, "line 2, column 7: name takes printable text: no tabs or control characters");
    says(&format!("{H}name a\tb\n"), ERR_TEXT, "line 2, column 7: name takes printable text: no tabs or control characters");
    says(&format!("{H}name a\nname b\n"), ERR_TEXT, "line 3, column 1: name comes once; it was given on line 2");
    says(&format!("{H}tempo\t100\n"), ERR_TEXT, "line 2, column 6: a tab: the beat file separates words with spaces");
    says(&format!("{H}tempo 100 é\n"), ERR_TEXT, "line 2, column 11: a character the beat file does not use outside name, credit and comments");
}

#[test]
fn text_that_is_not_utf8() {
    let mut text = format!("{H}name ").into_bytes();
    text.extend_from_slice(b"ab\xffc\n");
    let mut snap = vec![0u8; SIZE];
    let Err((code, m)) = read(&text, &mut snap) else { panic!("read") };
    assert_eq!((code, m.as_bytes()), (ERR_TEXT, &b"line 2, column 8: not UTF-8 text"[..]));
    // A comment is the file's text too.
    let mut text = format!("{H}tempo 100 # ").into_bytes();
    text.extend_from_slice(b"\xc3\n");
    let Err((_, m)) = read(&text, &mut snap) else { panic!("read") };
    assert_eq!(m.as_bytes(), b"line 2, column 13: not UTF-8 text");
}

#[test]
fn sounds_own_values_and_the_chain() {
    let sound = "sound takes a key, then any of: sample or code, dry or fx, a <A>, b <B>, in that order";
    says(&format!("{H}sound\n"), ERR_TEXT, &format!("line 2: {sound}"));
    says(&format!("{H}sound 0 dry\n"), ERR_TEXT, &format!("line 2, column 7: {sound}; found '0'"));
    says(&format!("{H}sound 1 dry sample\n"), ERR_TEXT, &format!("line 2, column 13: {sound}; 'sample' is out of place"));
    says(&format!("{H}sound 1 sample code\n"), ERR_TEXT, &format!("line 2, column 16: {sound}; 'code' is out of place"));
    says(&format!("{H}sound 1 wet\n"), ERR_TEXT, &format!("line 2, column 9: {sound}; found 'wet'"));
    says(&format!("{H}sound 1 a\n"), ERR_TEXT, &format!("line 2, column 9: {sound}; 'a' takes 0 to 65535 after it"));
    says(&format!("{H}sound 1 b 65536\n"), ERR_TEXT, "line 2, column 11: a and b take 0 to 65535; found '65536'");
    says(&format!("{H}sound 2 dry\nsound 2 sample\n"), ERR_TEXT, "line 3, column 1: sound 2 comes once; it was given on line 2");
    says(&format!("{H}own\n"), ERR_TEXT, "line 2: own takes keys, 1 to 16, each once");
    says(&format!("{H}own 3 3\n"), ERR_TEXT, "line 2, column 7: own takes keys, 1 to 16, each once; key 3 is named twice");
    says(&format!("{H}own 17\n"), ERR_TEXT, "line 2, column 5: own takes keys, 1 to 16, each once; found '17'");
    says(&format!("{H}sound 3 code\nown 3\n"), ERR_TEXT, "line 3: own names key 3, which line 2 says is code: own is for a key that plays a sample");
    let value = "value takes a letter (a to w or A to W) and A and B, each 0 to 65535";
    says(&format!("{H}value x 1 2\n"), ERR_TEXT, &format!("line 2, column 7: {value}; x is a step on keeping nothing, not a letter"));
    says(&format!("{H}value X 1 2\n"), ERR_TEXT, &format!("line 2, column 7: {value}; x is a step on keeping nothing, not a letter"));
    says(&format!("{H}value ab 1 2\n"), ERR_TEXT, &format!("line 2, column 7: {value}; found 'ab'"));
    says(&format!("{H}value a 1\n"), ERR_TEXT, &format!("line 2: {value}"));
    says(&format!("{H}value a 1 2\nvalue a 1 2\n"), ERR_TEXT, "line 3, column 7: value a comes once; it was given on line 2");
    let chain = "chain takes one to sixteen patterns, each 1 to 16";
    says(&format!("{H}chain\n"), ERR_TEXT, &format!("line 2: {chain}"));
    says(&format!("{H}chain{}\n", " 1".repeat(17)), ERR_TEXT, &format!("line 2: {chain}"));
    says(&format!("{H}chain 1 0\n"), ERR_TEXT, &format!("line 2, column 9: {chain}; found '0'"));
}

#[test]
fn patterns_rows_and_lanes() {
    let pattern = "pattern takes a number, 1 to 16, then length 1 to 16 if not 16";
    says(&format!("{H}pattern\n"), ERR_TEXT, &format!("line 2: {pattern}"));
    says(&format!("{H}pattern 17\n"), ERR_TEXT, &format!("line 2, column 9: {pattern}; found '17'"));
    says(&format!("{H}pattern 1 length 0\n"), ERR_TEXT, &format!("line 2, column 18: {pattern}; found '0'"));
    says(&format!("{H}pattern 1 long 4\n"), ERR_TEXT, &format!("line 2, column 11: {pattern}; found 'long'"));
    says(&format!("{H}pattern 1 length\n"), ERR_TEXT, &format!("line 2, column 11: {pattern}; found 'length'"));
    says(&format!("{H}pattern 1\npattern 1\n"), ERR_TEXT, "line 3, column 1: pattern 1 comes once; it was given on line 2");
    says(&format!("{H}1 kick x...............\n"), ERR_TEXT, "line 2: a row belongs to a pattern: a 'pattern <n>' line comes first");
    says(&format!("{H}fx 1=1\n"), ERR_TEXT, "line 2: an fx line belongs to a pattern: a 'pattern <n>' line comes first");
    let p = format!("{H}\npattern 1\n");
    says(&format!("{p}17 x...............\n"), ERR_TEXT, "line 4, column 1: a row starts with a key, 1 to 16; found '17'");
    says(&format!("{p}1 snar x...............\n"), ERR_TEXT, "line 4, column 3: no sound is called 'snar'; key 1 is the kick");
    says(&format!("{p}1 kick\n"), ERR_TEXT, "line 4: a row takes sixteen steps after its key and name");
    says(&format!("{p}1\n"), ERR_TEXT, "line 4: a row takes a key, its sound's name if given, sixteen steps, then prob and keep lists");
    says(&format!("{p}1 kick x................\n"), ERR_TEXT, "line 4, column 8: a row takes sixteen steps; this has 17");
    says(&format!("{p}1 kick ..y.............\n"), ERR_TEXT, "line 4, column 10: step 3: 'y' is not a step: . off, x on, a value's letter (a to w, A to W), or *");
    says(&format!("{p}1 kick ..X.............\n"), ERR_TEXT, "line 4, column 10: step 3: 'X' is not a step: . off, x on, a value's letter (a to w, A to W), or *");
    says(&format!("{p}1 kick *...............\n"), ERR_TEXT, "line 4, column 8: step 1 is * and keep gives it no pair");
    says(&format!("{p}1 kick x............... keep 1=1,2\n"), ERR_TEXT, "line 4, column 30: step 1 is 'x': keep is for a * step or an off one");
    says(&format!("value a 1 1\n{p}"), ERR_TEXT, "line 1: a beat file starts with 'pocket-instruments beat 1'");
    says(&format!("{H}value a 1 1\n\npattern 1\n1 kick a............... keep 1=1,2\n"), ERR_TEXT, "line 5, column 30: step 1 is 'a': keep is for a * step or an off one");
    let prob = "prob takes <step>=<0 to 100>, steps 1 to 16, each once";
    says(&format!("{p}1 kick x............... prob 1=101\n"), ERR_TEXT, &format!("line 4, column 30: {prob}; found '1=101'"));
    says(&format!("{p}1 kick x............... prob 0=1\n"), ERR_TEXT, &format!("line 4, column 30: {prob}; found '0=1'"));
    says(&format!("{p}1 kick x............... prob 1=100 1=50\n"), ERR_TEXT, &format!("line 4, column 36: {prob}; step 1 is given twice"));
    says(&format!("{p}1 kick x............... prob\n"), ERR_TEXT, "line 4, column 25: prob takes at least one <step>=");
    says(&format!("{p}1 kick x............... prob 1=5 prob 2=5\n"), ERR_TEXT, "line 4, column 34: prob comes once in a row");
    says(&format!("{p}1 kick x............... loud\n"), ERR_TEXT, "line 4, column 25: after the sixteen steps a row takes prob and keep lists; found 'loud'");
    let keep = "keep takes <step>=<A>,<B>, steps 1 to 16, each once, A and B 0 to 65535";
    says(&format!("{p}1 kick *............... keep 1=1\n"), ERR_TEXT, &format!("line 4, column 30: {keep}; found '1=1'"));
    says(&format!("{p}1 kick *............... keep 1=1,2 1=3,4\n"), ERR_TEXT, &format!("line 4, column 36: {keep}; step 1 is given twice"));
    says(&format!("{p}1 kick ................\n1 kick ................\n"), ERR_TEXT, "line 5, column 1: key 1 has a row in pattern 1 already");
    let fx = "fx takes <step>=<effect>, steps 1 to 16, effects 1 to 16, each step once";
    says(&format!("{p}fx 3=0\n"), ERR_TEXT, &format!("line 4, column 4: {fx}; found '3=0'"));
    says(&format!("{p}fx 3=17\n"), ERR_TEXT, &format!("line 4, column 4: {fx}; found '3=17'"));
    says(&format!("{p}fx 3=1 3=2\n"), ERR_TEXT, &format!("line 4, column 8: {fx}; step 3 is given twice"));
    says(&format!("{p}fx\n"), ERR_TEXT, &format!("line 4: {fx}"));
    says(&format!("{p}fx 1=1\nfx 2=1\n"), ERR_TEXT, "line 5, column 1: fx comes once a pattern; pattern 1's was given on line 4");
}

#[test]
fn too_little_room_and_a_message_cut_to_its_buffer() {
    let mut snap = vec![0u8; 3003];
    let Err((code, m)) = read(EXAMPLE.as_bytes(), &mut snap) else { panic!("read") };
    assert_eq!((code, m.as_bytes()), (ERR_ROOM, &b"the snapshot takes 3004 bytes; the room given is 3003"[..]));
    assert!(snap.iter().all(|b| *b == 0), "nothing written");
    // Through the C ABI: the message ends with a zero byte, cut on a character boundary.
    let text = format!("{H}value a 1 2\n\npattern 1\n1 kick ..c.............\nbpmé\n");
    let mut msg = [0xEEu8; 8];
    let mut own = 77u32;
    let n = unsafe { crate::pib_read(text.as_ptr(), text.len() as u32, snap.as_mut_ptr(), snap.len() as u32, &mut own, msg.as_mut_ptr(), 8) };
    assert_eq!(n, ERR_TEXT);
    assert_eq!(&msg, b"line 6,\0"); // line 6, bpmé, is refused before the letter c is looked for
    assert_eq!(own, 77, "own untouched by a refusal");
    let quoted = format!("{H}é{}\n", "é".repeat(30));
    let (_, m) = refusal(&quoted);
    assert_eq!(m, "line 2, column 1: a character the beat file does not use outside name, credit and comments");
    // A null message and a null own are allowed.
    let n = unsafe { crate::pib_read(EXAMPLE.as_ptr(), EXAMPLE.len() as u32, snap.as_mut_ptr(), 3004.min(snap.len()) as u32, core::ptr::null_mut(), core::ptr::null_mut(), 0) };
    assert_eq!(n, ERR_ROOM);
    let mut big = vec![0u8; SIZE];
    let n = unsafe { crate::pib_read(EXAMPLE.as_ptr(), EXAMPLE.len() as u32, big.as_mut_ptr(), SIZE as u32, core::ptr::null_mut(), core::ptr::null_mut(), 0) };
    assert_eq!(n, 3004);
}

#[test]
fn a_quote_is_cut_after_24_bytes_on_a_character_boundary() {
    let (_, m) = refusal(&format!("{H}chosen {}\n", "9".repeat(30)));
    assert_eq!(m, format!("line 2, column 8: chosen takes a key, 1 to 16; found '{}...'", "9".repeat(24)));
}
