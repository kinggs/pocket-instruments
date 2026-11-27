//! What the reader and the writer share: a message in a fixed buffer, decimal numbers, a sink
//! that counts before it writes, and the snapshot's CRC.

/// A message's bytes: `MSG_MAX` with its zero byte.
const ROOM: usize = crate::MSG_MAX - 1;

/// A refusal's text, at most `MSG_MAX - 1` bytes, never cut inside a character.
#[derive(Clone, Copy, Debug)]
pub struct Msg {
    buf: [u8; ROOM],
    len: usize,
}

impl Msg {
    pub const fn new() -> Msg {
        Msg { buf: [0; ROOM], len: 0 }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Appends what fits, cut before any character that would not.
    pub fn s(mut self, bytes: &[u8]) -> Msg {
        let room = ROOM - self.len;
        let n = cut(bytes, room);
        self.buf[self.len..self.len + n].copy_from_slice(&bytes[..n]);
        self.len += n;
        self
    }

    pub fn n(self, v: u32) -> Msg {
        let mut d = [0u8; 10];
        let k = digits(v, &mut d);
        self.s(&d[10 - k..])
    }

    /// Something the file holds, quoted: at most 24 bytes of it, then `...`.
    pub fn q(self, bytes: &[u8]) -> Msg {
        let n = cut(bytes, 24);
        let m = self.s(b"'").s(&bytes[..n]);
        if n < bytes.len() { m.s(b"...'") } else { m.s(b"'") }
    }

    /// Into a caller's buffer, ended by a zero byte, cut on a character boundary to fit.
    pub fn copy_to(&self, out: &mut [u8]) {
        let Some(room) = out.len().checked_sub(1) else {
            return;
        };
        let n = cut(self.as_bytes(), room);
        out[..n].copy_from_slice(&self.buf[..n]);
        out[n] = 0;
    }
}

impl Default for Msg {
    fn default() -> Msg {
        Msg::new()
    }
}

/// The most of `bytes`, up to `most`, that ends on a character boundary.
pub fn cut(bytes: &[u8], most: usize) -> usize {
    if bytes.len() <= most {
        return bytes.len();
    }
    let mut n = most;
    // A continuation byte is 10xxxxxx: step back to the start of its character.
    while n > 0 && bytes[n] & 0xC0 == 0x80 {
        n -= 1;
    }
    n
}

/// `v` in decimal at the end of `d`; returns how many digits.
pub fn digits(mut v: u32, d: &mut [u8; 10]) -> usize {
    let mut k = 0;
    loop {
        d[9 - k] = b'0' + (v % 10) as u8;
        v /= 10;
        k += 1;
        if v == 0 || k == 10 {
            return k;
        }
    }
}

/// Where text goes: counted only (to measure before writing), or written too.
pub struct Sink<'a> {
    buf: Option<&'a mut [u8]>,
    pub len: usize,
}

impl<'a> Sink<'a> {
    pub fn count() -> Sink<'a> {
        Sink { buf: None, len: 0 }
    }

    pub fn into(buf: &'a mut [u8]) -> Sink<'a> {
        Sink { buf: Some(buf), len: 0 }
    }

    pub fn s(&mut self, bytes: &[u8]) {
        if let Some(buf) = self.buf.as_deref_mut() {
            if let Some(dst) = buf.get_mut(self.len..self.len + bytes.len()) {
                dst.copy_from_slice(bytes);
            }
        }
        self.len += bytes.len();
    }

    pub fn b(&mut self, byte: u8) {
        self.s(&[byte]);
    }

    pub fn n(&mut self, v: u32) {
        let mut d = [0u8; 10];
        let k = digits(v, &mut d);
        self.s(&d[10 - k..]);
    }
}

/// CRC-32, the zip one (`ENGINE.md` §8), bit by bit, as the core computes it.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in bytes {
        crc ^= b as u32;
        let mut i = 0;
        while i < 8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & (crc & 1).wrapping_neg());
            i += 1;
        }
    }
    !crc
}

pub fn u16_at(b: &[u8], at: usize) -> u16 {
    match b.get(at..at + 2) {
        Some(x) => u16::from_le_bytes([x[0], x[1]]),
        None => 0,
    }
}

pub fn u32_at(b: &[u8], at: usize) -> u32 {
    match b.get(at..at + 4) {
        Some(x) => u32::from_le_bytes([x[0], x[1], x[2], x[3]]),
        None => 0,
    }
}

pub fn put(b: &mut [u8], at: usize, bytes: &[u8]) {
    if let Some(dst) = b.get_mut(at..at + bytes.len()) {
        dst.copy_from_slice(bytes);
    }
}
