//! The bank (`docs/BANK.md`, D141 item 4): `beats/bank.txt` held to the folder. Every `.beat`
//! in `beats/` listed once, every line a file there, and each one read by this crate. The order
//! is the menu's and the first line is the default (D142): which beat that is, is not held here,
//! so a ruling that moves it changes `bank.txt` alone.

use super::bytes_of;

const BEATS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../beats");

#[test]
fn bank_txt_lists_every_beat_in_the_folder_once_and_every_line_is_a_file_there() {
    let list = std::fs::read_to_string(format!("{BEATS}/bank.txt")).expect("beats/bank.txt");
    assert!(list.ends_with('\n') && !list.contains('\r'), "bank.txt: one file a line, each ended by a newline");
    let lines: Vec<&str> = list.lines().collect();
    assert!(!lines.is_empty(), "bank.txt names no beat");
    let mut folder: Vec<String> = std::fs::read_dir(BEATS)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".beat"))
        .collect();
    folder.sort();
    for (i, l) in lines.iter().enumerate() {
        assert!(!l.is_empty() && *l == l.trim() && !l.contains('/'), "bank.txt line {}: {l:?} is not a file name", i + 1);
        assert!(folder.iter().any(|f| f == l), "bank.txt line {}: {l} is not in beats/", i + 1);
        assert_eq!(lines.iter().filter(|x| *x == l).count(), 1, "bank.txt: {l} listed more than once");
    }
    for f in &folder {
        assert!(lines.contains(&f.as_str()), "beats/{f} is not in bank.txt");
    }
    // Each reads, and its name is unique in the bank (D141 item 3: the menu shows it).
    let mut names = Vec::new();
    for l in &lines {
        let snap = bytes_of(&std::fs::read_to_string(format!("{BEATS}/{l}")).unwrap());
        let name = &snap[112..144];
        let name = &name[..name.iter().position(|b| *b == 0).unwrap_or(32)];
        assert!(!name.is_empty(), "{l} has no name");
        assert!(!names.contains(&name.to_vec()), "{l}: its name is another beat's");
        names.push(name.to_vec());
    }
}
