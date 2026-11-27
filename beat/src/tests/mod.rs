//! The beat crate on its own: the worked example byte by byte, the canonical form, every
//! refusal, the worst text, and a mutation fuzz that nothing panics. What needs the core
//! (`pi_load` and `pi_save` as the referee) is in the core's tests: `core/src/tests/beat.rs`.

mod example;
mod fuzz;
mod refusals;
mod size;

use crate::layout::*;
use crate::{MSG_MAX, TEXT_MAX, read, write};

pub const EXAMPLE: &str = include_str!("../../../fixtures/beat-four-and-a-fill.beat");

/// Reads text into a fresh snapshot; panics with the message if it is refused.
pub fn bytes_of(text: &str) -> Vec<u8> {
    let mut snap = vec![0u8; SIZE];
    match read(text.as_bytes(), &mut snap) {
        Ok(r) => {
            snap.truncate(r.len);
            snap
        }
        Err((_, m)) => panic!("refused: {}", String::from_utf8_lossy(m.as_bytes())),
    }
}

/// Writes a snapshot as text; panics if it is refused.
pub fn text_of(snap: &[u8], own: u16) -> String {
    let mut out = vec![0u8; TEXT_MAX];
    let n = write(snap, own, &mut out).unwrap_or_else(|e| panic!("pib_write refused: {e}"));
    out.truncate(n);
    String::from_utf8(out).unwrap()
}

/// The refusal for `text`: its code and message.
pub fn refusal(text: &str) -> (i32, String) {
    let mut snap = vec![0xEEu8; SIZE];
    match read(text.as_bytes(), &mut snap) {
        Ok(_) => panic!("read, not refused:\n{text}"),
        Err((code, m)) => {
            assert!(snap.iter().all(|b| *b == 0xEE), "a refusal writes nothing");
            assert!(m.as_bytes().len() < MSG_MAX, "a message and its zero byte fit PIB_MSG_MAX");
            (code, String::from_utf8(m.as_bytes().to_vec()).unwrap())
        }
    }
}

/// `include/pi_beat.h` says the same numbers as the crate: a host reads them from there.
#[test]
fn the_header_s_numbers_are_the_crate_s() {
    let h = include_str!("../../include/pi_beat.h");
    let define = |name: &str| -> i64 {
        let line = h.lines().find(|l| l.starts_with(&format!("#define {name} "))).unwrap_or_else(|| panic!("no {name}"));
        line.split_whitespace().nth(2).unwrap().trim_end_matches('u').parse().unwrap()
    };
    assert_eq!(define("PIB_BEAT_VERSION"), crate::BEAT_VERSION as i64);
    assert_eq!(define("PIB_TEXT_MAX"), TEXT_MAX as i64);
    assert_eq!(define("PIB_MSG_MAX"), MSG_MAX as i64);
    assert_eq!(define("PIB_SNAPSHOT_MAX"), SIZE as i64);
    assert_eq!(define("PIB_ERR_TEXT"), crate::ERR_TEXT as i64);
    assert_eq!(define("PIB_ERR_ROOM"), crate::ERR_ROOM as i64);
    assert_eq!(define("PIB_ERR_NEWER"), crate::ERR_NEWER as i64);
    assert_eq!(define("PIB_ERR_SNAPSHOT"), crate::ERR_SNAPSHOT as i64);
}
