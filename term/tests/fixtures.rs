//! P2 through the terminal host (docs/TERMINAL.md, "The proofs"; D123 item 8).
//!
//! Every fixture as `harness/harness.cpp` runs it (fixtures/README.md: three rates, three block
//! sizes, both sources, from a snapshot and from a kept snapshot), on the same static library.
//! The difference: every `key_down` and `key_up` goes in as the bytes foot sends for that key
//! (the kitty sequence for press and release, R78) and through this host's reader, key map and
//! latch logic; every `param` through its slider. The report it writes (PI_REPORT) must be the
//! same bytes as the other three harnesses': `make test` compares them.
//!
//! SPDX-License-Identifier: MIT
use pi_term::ffi::*;
use pi_term::host::{Host, Mode, char_of};
use pi_term::input::{Input, Kind, Reader, kitty_bytes};

// ---- just enough JSON for the fixture formats: objects, arrays, strings, whole numbers ----

#[derive(Clone, Debug)]
enum Json {
    Num(u64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, k: &str) -> Option<&Json> {
        match self {
            Json::Obj(o) => o.iter().find(|(n, _)| n == k).map(|(_, v)| v),
            _ => None,
        }
    }
    fn at(&self, k: &str) -> &Json {
        self.get(k).unwrap_or_else(|| panic!("missing field {k}"))
    }
    fn n(&self, k: &str) -> u64 {
        match self.at(k) {
            Json::Num(n) => *n,
            _ => panic!("{k} is not a number"),
        }
    }
    fn n_or(&self, k: &str, d: u64) -> u64 {
        match self.get(k) {
            Some(Json::Num(n)) => *n,
            _ => d,
        }
    }
    fn s(&self, k: &str) -> &str {
        match self.at(k) {
            Json::Str(s) => s,
            _ => panic!("{k} is not a string"),
        }
    }
    fn arr(&self, k: &str) -> &[Json] {
        match self.at(k) {
            Json::Arr(a) => a,
            _ => panic!("{k} is not an array"),
        }
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn space(&mut self) {
        while self.i < self.s.len() && b" \t\r\n".contains(&self.s[self.i]) {
            self.i += 1;
        }
    }
    fn peek(&mut self) -> u8 {
        self.space();
        self.s[self.i]
    }
    fn eat(&mut self, c: u8) {
        assert_eq!(self.peek(), c, "JSON: expected {}", c as char);
        self.i += 1;
    }
    fn string(&mut self) -> String {
        self.eat(b'"');
        let mut out = Vec::new();
        while self.s[self.i] != b'"' {
            if self.s[self.i] == b'\\' {
                self.i += 1;
                out.push(match self.s[self.i] {
                    b'n' => b'\n',
                    b't' => b'\t',
                    c @ (b'"' | b'\\' | b'/') => c,
                    _ => panic!("JSON: an escape that is not in the formats"),
                });
            } else {
                out.push(self.s[self.i]);
            }
            self.i += 1;
        }
        self.i += 1;
        String::from_utf8(out).unwrap()
    }
    fn value(&mut self) -> Json {
        match self.peek() {
            b'{' => {
                self.eat(b'{');
                let mut o = Vec::new();
                if self.peek() != b'}' {
                    loop {
                        let k = self.string();
                        self.eat(b':');
                        o.push((k, self.value()));
                        if self.peek() != b',' {
                            break;
                        }
                        self.eat(b',');
                    }
                }
                self.eat(b'}');
                Json::Obj(o)
            }
            b'[' => {
                self.eat(b'[');
                let mut a = Vec::new();
                if self.peek() != b']' {
                    loop {
                        a.push(self.value());
                        if self.peek() != b',' {
                            break;
                        }
                        self.eat(b',');
                    }
                }
                self.eat(b']');
                Json::Arr(a)
            }
            b'"' => Json::Str(self.string()),
            _ => {
                let mut n = 0u64;
                while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                    n = n * 10 + (self.s[self.i] - b'0') as u64;
                    self.i += 1;
                }
                Json::Num(n)
            }
        }
    }
}

fn fixtures() -> Vec<(String, Json)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures");
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect();
    files.sort();
    assert!(files.len() >= 9, "expected the fixtures");
    files
        .into_iter()
        .map(|p| {
            let text = std::fs::read(&p).unwrap();
            let fx = Parser { s: &text, i: 0 }.value();
            let format = fx.n("format");
            assert!((1..=5).contains(&format) && fx.n("ticks_per_step") == PI_TICKS_PER_STEP);
            assert!(fx.get("beat").is_none(), "a fixture with a beat file waits for the beat reader (D134)");
            (p.file_stem().unwrap().to_string_lossy().into_owned(), fx)
        })
        .collect()
}

fn is_key(name: &str) -> bool {
    matches!(name, "key_down" | "key_up" | "param")
}

fn is_edit(name: &str) -> bool {
    // Format 5's three (ENGINE.md §21) are edits too: at step 0 they come from the snapshot.
    matches!(name, "step" | "prob" | "swing" | "length" | "fx" | "fxsound" | "keep" | "pattern" | "chain" | "copy")
}

fn to_cmd(c: &Json) -> Cmd {
    let name = c.s("cmd");
    let (track, step, value) = (c.n_or("track", 0) as u8, c.n_or("step", 0) as u8, c.n_or("value", 0) as u32);
    match name {
        "play" => Cmd::new(PI_CMD_PLAY, 0, 0, value),
        "reset" => Cmd::new(PI_CMD_RESET, 0, 0, 0),
        "step" => Cmd::new(PI_CMD_STEP, track, step, value),
        "prob" => Cmd::new(PI_CMD_PROB, track, step, value),
        "swing" => Cmd::new(PI_CMD_SWING, 0, 0, value),
        "length" => Cmd::new(PI_CMD_LENGTH, 0, 0, value),
        "fx" => Cmd::new(PI_CMD_FX, step, 0, value),
        "fxsound" => Cmd::new(PI_CMD_FXSOUND, track, 0, value),
        "keep" => Cmd::new(PI_CMD_KEEP, track, step, (c.n("a") as u32) << 16 | c.n("b") as u32),
        "pattern" => Cmd::new(PI_CMD_PATTERN, 0, 0, value),
        "chain" => Cmd::new(PI_CMD_CHAIN, 0, 0, value),
        "copy" => Cmd::new(PI_CMD_COPY, 0, 0, value),
        other => panic!("not an edit: {other}"),
    }
}

/// A finger, through the terminal: the bytes foot sends, read and mapped by this host.
fn through_host(host: &mut Host, reader: &mut Reader, c: &Json) -> Vec<Cmd> {
    match c.s("cmd") {
        "param" => host.slide(c.n("slider") as usize, c.n_or("value", 0) as i32),
        name => {
            let kind = if name == "key_down" { Kind::Press } else { Kind::Release };
            let bytes = kitty_bytes(char_of(c.n("key") as u8), false, kind);
            for i in reader.feed(&bytes) {
                match i {
                    Input::Key(e) => host.key(e),
                    other => panic!("not a key: {other:?}"),
                }
            }
        }
    }
    host.take_cmds()
}

type Timed = (u64, Cmd);

/// What to send and when, the views to read, and the frame the run ends before
/// (fixtures/README.md, "How a harness runs one").
fn schedule(fx: &Json, rate: u32, source: u32, from_snapshot: bool) -> (Vec<Timed>, Vec<(u64, String)>, u64) {
    let (mbpm, steps) = (fx.n("mbpm"), fx.n("steps"));
    let f = |n: u64| (n * rate as u64 * 15000).div_ceil(mbpm);
    let (mut anchor, mut anchor_step) = (0u64, 0u64);
    let mut cmds = Vec::new();
    let mut step_frames = Vec::new();
    let mut host = Host::new(Mode::Hold);
    let mut reader = Reader::new();
    if !from_snapshot {
        cmds.push((0, Cmd::new(PI_CMD_TEMPO, 0, 0, mbpm as u32)));
    }
    cmds.push((0, Cmd::new(PI_CMD_SOURCE, 0, 0, source)));
    for m in 0..steps {
        let at = anchor + f(m - anchor_step);
        step_frames.push(at);
        for c in fx.arr("commands") {
            if c.n("at") != m || is_key(c.s("cmd")) {
                continue;
            }
            if from_snapshot && m == 0 && is_edit(c.s("cmd")) {
                continue;
            }
            cmds.push((at, to_cmd(c)));
            if c.s("cmd") == "reset" {
                anchor = at;
                anchor_step = m;
            }
        }
        if source == PI_SOURCE_EDGES {
            cmds.push((at, Cmd::new(PI_CMD_CLOCK, 0, 0, 0)));
        }
        for c in fx.arr("commands") {
            if c.n("at") == m && is_key(c.s("cmd")) {
                for k in through_host(&mut host, &mut reader, c) {
                    cmds.push((at + 1, k));
                }
            }
        }
    }
    let end = anchor + f(steps - anchor_step);
    let mut views = Vec::new();
    if let Some(Json::Arr(vs)) = fx.get("views") {
        for v in vs {
            let at = v.n("at");
            let mut text = v.s("view").to_string();
            if source == PI_SOURCE_EDGES {
                text = text.replacen(" edges=0 ", " edges=1 ", 1);
            }
            views.push((if at + 1 < steps { step_frames[at as usize + 1] } else { end }, text));
        }
    }
    (cmds, views, end)
}

fn view_text(b: &[u8; PI_VIEW_SIZE], format: u64) -> String {
    let flags = b[PI_VIEW_FLAGS];
    let fl = |bit: u8| if flags & bit != 0 { 1 } else { 0 };
    let step = if flags & PI_VIEW_FLAG_STEP != 0 { b[PI_VIEW_STEP].to_string() } else { "-".into() };
    let mut s = format!(
        "view play={} write={} edges={} sounds={} step={} sel={} vol={} tempo={} len={} swing={} held={:06x} field={:04x} stitch={:04x} bar={:04x} sampled={:04x} a={} b={} fires=",
        fl(PI_VIEW_FLAG_PLAYING),
        fl(PI_VIEW_FLAG_WRITE),
        fl(PI_VIEW_FLAG_EDGES),
        fl(PI_VIEW_FLAG_SOUNDS),
        step,
        b[PI_VIEW_SEL],
        b[PI_VIEW_VOLUME],
        u32_at(b, PI_VIEW_TEMPO),
        b[PI_VIEW_LENGTH],
        b[PI_VIEW_SWING],
        u32_at(b, PI_VIEW_HELD),
        u16_at(b, PI_VIEW_FIELD),
        u16_at(b, PI_VIEW_STITCH),
        u16_at(b, PI_VIEW_BAR),
        u16_at(b, PI_VIEW_SAMPLED),
        u16_at(b, PI_VIEW_A),
        u16_at(b, PI_VIEW_B)
    );
    s += &(0..16).map(|t| b[PI_VIEW_FIRES + t].to_string()).collect::<Vec<_>>().join(",");
    if format >= 3 {
        s += &format!(" fx={} fxsteps={:04x} fxsounds={:04x}", b[PI_VIEW_HEARD], u16_at(b, PI_VIEW_FX_STEPS), u16_at(b, PI_VIEW_FX_SOUNDS));
    }
    if format >= 4 {
        s += &format!(" ea={} eb={}", u16_at(b, PI_VIEW_HIT_A), u16_at(b, PI_VIEW_HIT_B));
    }
    if format >= 5 {
        // §21 (ABI 16): the patterns, the chain and the next chain a hex digit an entry.
        let nibbles = |at: usize, len: u8| -> String { (0..len as usize).map(|i| format!("{:x}", b[at + i / 2] >> (4 * (i & 1)) & 15)).collect() };
        let next = if b[PI_VIEW_NEXT_LEN] == 0 { "-".to_string() } else { nibbles(PI_VIEW_NEXT, b[PI_VIEW_NEXT_LEN]) };
        s += &format!(
            " pat={} pos={} chain={} next={} used={:04x}",
            b[PI_VIEW_PATTERN],
            b[PI_VIEW_POS],
            nibbles(PI_VIEW_CHAIN, b[PI_VIEW_CHAIN_LEN]),
            next,
            u16_at(b, PI_VIEW_USED)
        );
    }
    s
}

/// The zip CRC-32, bit by bit (ENGINE.md §8).
fn crc32(bytes: &[u8]) -> u32 {
    let mut c = !0u32;
    for b in bytes {
        c ^= *b as u32;
        for _ in 0..8 {
            c = c >> 1 ^ (0xEDB8_8320 & (c & 1).wrapping_neg());
        }
    }
    !c
}

/// Runs to `end` in blocks of `block`, ending a block wherever a view is read.
fn run(e: &mut Engine, cmds: &[Timed], end: u64, block: u64, views: &[(u64, String)], format: u64) -> (String, String, bool) {
    let mut out = String::new();
    let mut seen = String::new();
    let mut ok = true;
    let mut ev = vec![Event::default(); 256];
    let (mut next, mut view) = (0, 0);
    let mut start = 0u64;
    while start < end {
        let stop = if view < views.len() { views[view].0 } else { end };
        let n = block.min(stop - start);
        let mut these = Vec::new();
        while next < cmds.len() && cmds[next].0 < start + n {
            let mut c = cmds[next].1;
            c.frame = (cmds[next].0 - start) as u32;
            these.push(c);
            next += 1;
        }
        let got = e.process(n as u32, &these, &mut ev, None, PI_AUDIO_NONE);
        for v in &ev[..got] {
            match v.ty {
                PI_EV_STEP => out += &format!("{} step {}\n", v.tick, v.step),
                PI_EV_TRIG => out += &format!("{} trig {} {}\n", v.tick, v.track, v.step),
                _ => panic!("unknown event"),
            }
        }
        start += n;
        while view < views.len() && views[view].0 == start {
            let t = view_text(&e.view(), format);
            if t != views[view].1 {
                ok = false;
            }
            seen += &t;
            seen += "\n";
            view += 1;
        }
    }
    assert_eq!(view, views.len(), "a view was never read");
    assert_eq!(e.dropped(), 0, "events were dropped");
    (out, seen, ok)
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

#[test]
fn every_fixture_through_the_terminal_host() {
    let mut report = String::new();
    let (mut runs, mut failed) = (0, Vec::new());
    let all = fixtures();
    // Issue 143: the four patterns fixtures (ENGINE.md §21) are among them, their keys played
    // through this host's reader, map and latches like every other's.
    for want in ["patterns-chain", "patterns-change-playing", "patterns-choose-and-play", "patterns-copy-and-clear"] {
        assert!(all.iter().any(|(n, _)| n == want), "{want} is not in fixtures/");
    }
    for (name, fx) in &all {
        let seed = fx.n("seed") as u32;
        let format = fx.n("format");
        let want: String = fx.arr("expect").iter().map(|l| if let Json::Str(s) = l { format!("{s}\n") } else { panic!() }).collect();
        let mut check = |label: String, got: String, seen: String, ok: bool| {
            runs += 1;
            if got != want || !ok {
                failed.push(format!("{label}{}", if ok { "" } else { " (a view)" }));
            }
            report += &format!("# {label}\n{got}{seen}");
        };
        for (source_name, source) in [("table", PI_SOURCE_TABLE), ("edges", PI_SOURCE_EDGES)] {
            for rate in [44100u32, 48000, 96000] {
                let where_ = format!("{name} {source_name} {rate}");
                for block in [1u64, 128, 512] {
                    let (cmds, views, end) = schedule(fx, rate, source, false);
                    let mut e = Engine::new(rate, seed);
                    let (got, seen, ok) = run(&mut e, &cmds, end, block, &views, format);
                    check(format!("{where_} {block}"), got, seen, ok);
                }
                // Save, load into a fresh engine with another seed, same trace.
                let snap = {
                    let mut a = Engine::new(rate, seed);
                    let mut edits = vec![(0u64, Cmd::new(PI_CMD_TEMPO, 0, 0, fx.n("mbpm") as u32))];
                    for c in fx.arr("commands") {
                        if c.n("at") == 0 && is_edit(c.s("cmd")) {
                            edits.push((0, to_cmd(c)));
                        }
                    }
                    run(&mut a, &edits, 1, 1, &[], format);
                    a.save()
                };
                let mut b = Engine::new(rate, seed + 12345);
                assert_eq!(b.load(&snap), PI_OK);
                let (cmds, views, end) = schedule(fx, rate, source, true);
                let (got, seen, ok) = run(&mut b, &cmds, end, 128, &views, format);
                check(format!("{where_} snapshot {}", hex(&snap)), got, seen, ok);
                // §20: from the snapshot, every step that is on keeping its sound's own A and B.
                // Since ABI 16 written into version 5's bytes, in each pattern written, as a
                // KEEP there would write them; then loaded and saved again, so the bytes are the
                // core's (core/src/tests/fixtures.rs).
                let kept = {
                    let mut k = snap.clone();
                    let written = u16_at(&k, 92);
                    let mut at = PI_SNAPSHOT_SIZE_MIN - 4;
                    for _ in 0..written.count_ones() {
                        for t in 0..16usize {
                            let (on, has) = (u16_at(&k, at + 2 * t), u16_at(&k, at + 304 + 2 * t));
                            for st in (0..16usize).filter(|st| on >> st & 1 != 0 && has >> st & 1 == 0) {
                                k.copy_within(24 + 4 * t..28 + 4 * t, at + 336 + 4 * (16 * t + st));
                            }
                            k[at + 304 + 2 * t..at + 306 + 2 * t].copy_from_slice(&(on | has).to_le_bytes());
                        }
                        at += 1364;
                    }
                    let n = k.len() - 4;
                    let crc = crc32(&k[..n]);
                    k[n..].copy_from_slice(&crc.to_le_bytes());
                    let mut a = Engine::new(rate, seed);
                    assert_eq!(a.load(&k), PI_OK);
                    assert_eq!(a.save(), k, "the core saves the same bytes");
                    k
                };
                let mut c = Engine::new(rate, seed + 12345);
                assert_eq!(c.load(&kept), PI_OK);
                let (got, seen, ok) = run(&mut c, &cmds, end, 128, &views, format);
                check(format!("{where_} kept snapshot {}", hex(&kept)), got, seen, ok);
                // One flipped bit is refused.
                let mut bad = snap.clone();
                bad[40] ^= 1;
                assert_eq!(b.load(&bad), PI_ERR_CRC);
            }
        }
    }
    if let Ok(path) = std::env::var("PI_REPORT") {
        std::fs::write(path, &report).unwrap();
    }
    assert!(failed.is_empty(), "{} of {runs} runs did not match: {:?}", failed.len(), &failed[..failed.len().min(10)]);
    assert_eq!(runs, all.len() * 2 * 3 * 5);
}
