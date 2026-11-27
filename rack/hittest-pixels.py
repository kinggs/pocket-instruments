"""The Rhythm module's screenshot read back at points worked out from D139's numbers (issue 150).

    python3 hittest-pixels.py <Rhythm.png>      exits 0 only if every point is the colour it must be

Screenshot mode draws a module with no engine, so every key is dark (tile) and each slider's
cap sits in the middle of its box. The points, in mm from the panel's top left corner, are
D139's (design/DESIGN.md, "Larger keys and less empty space"), typed here from the decision and
not read from make_panel.py or Rhythm.cpp, so a size drawn wrong in either is caught:

- each grid key's face 0.3 mm inside each corner, and the middle of every gap the keys of
  11.7 mm at a 12 mm pitch left until D139: tile;
- the middle of each gap between two keys now (0.4 mm): not tile;
- each function key 0.3 mm inside its corners: its band's colour at the top, tile at the bottom;
- each slider cap 0.3 mm inside its left and right edges, in its top third: ecru; and 0.3 mm
  outside them: not ecru.

A pure reader of 8-bit RGB or RGBA PNG, so nothing need be installed.
SPDX-License-Identifier: MIT
"""
import struct
import sys
import zlib

W_MM, H_MM = 137.16, 128.5
TILE, ECRU, GROUND = (0x2B, 0x29, 0x42), (0xF2, 0xE8, 0xD0), (0x14, 0x13, 0x1F)
BANDS = {"sound": (0xF0, 0x65, 0x4A), "pattern": (0xF0, 0xB4, 0x29), "bpm": (0x8F, 0xAA, 0xF7),
         "function": ECRU, "fx": (0x6F, 0xD6, 0xA0), "play": ECRU, "write": ECRU}


def read_png(path):
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    pos, idat, w = 8, b"", 0
    while pos < len(data):
        n, kind = struct.unpack(">I4s", data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + n]
        if kind == b"IHDR":
            w, h, depth, colour, _, _, interlace = struct.unpack(">IIBBBBB", body)
            assert depth == 8 and colour in (2, 6) and interlace == 0, "only 8-bit RGB or RGBA, not interlaced"
            bpp = 3 if colour == 2 else 4
        elif kind == b"IDAT":
            idat += body
        pos += 12 + n
    raw, stride = zlib.decompress(idat), w * bpp
    rows, prev, i = [], bytearray(stride), 0
    for _ in range(h):
        f, line = raw[i], bytearray(raw[i + 1:i + 1 + stride])
        i += 1 + stride
        for x in range(stride):
            a = line[x - bpp] if x >= bpp else 0
            b, c = prev[x], prev[x - bpp] if x >= bpp else 0
            if f == 1:
                line[x] = (line[x] + a) & 255
            elif f == 2:
                line[x] = (line[x] + b) & 255
            elif f == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(line)
        prev = line
    return w, h, bpp, rows


def main(path):
    w, h, bpp, rows = read_png(path)
    sx, sy = w / W_MM, h / H_MM  # the screenshot is the panel, edge to edge

    def at(x, y):
        px, py = int(x * sx), int(y * sy)
        return tuple(rows[py][px * bpp:px * bpp + 3])

    def near(c, want, tol=24):
        return all(abs(a - b) <= tol for a, b in zip(c, want))

    bad, n = [], 0

    def expect(what, x, y, want, yes=True):
        nonlocal n
        n += 1
        c = at(x, y)
        if near(c, want) != yes:
            bad.append(f"{what} at ({x:.2f}, {y:.2f}) mm is #{bytes(c).hex()}, {'not ' if yes else ''}#{bytes(want).hex()}")

    key, pitch, x0, y0, inset = 13.6, 14.0, 12.0, 51.0, 0.3
    for r in range(4):
        for c in range(4):
            cx, cy = x0 + pitch * c, y0 + pitch * r
            for dx in (-1, 1):
                for dy in (-1, 1):
                    expect(f"key {4 * r + c + 1}'s corner", cx + dx * (key / 2 - inset), cy + dy * (key / 2 - inset), TILE)
            if c < 3:
                expect(f"the gap right of key {4 * r + c + 1}", cx + pitch / 2, cy, GROUND)
            if r < 3:
                expect(f"the gap under key {4 * r + c + 1}", cx, cy + pitch / 2, GROUND)

    def on_key(x, y):
        return any(abs(x - (x0 + pitch * c)) < key / 2 and abs(y - (y0 + pitch * r)) < key / 2 for r in range(4) for c in range(4))

    # The old gaps (until D139: 11.7 mm keys at a 12 mm pitch from (18.62, 50)): 21 of the 24 are
    # on a key now; three, on the old bottom row's centre line (y 86), are in the new row gap.
    old = [(18.62 + 6.0 + 12.0 * c, 50.0 + 12.0 * r) for r in range(4) for c in range(3)]
    old += [(18.62 + 12.0 * c, 56.0 + 12.0 * r) for r in range(3) for c in range(4)]
    for x, y in old:  # on a key: its face, or its numeral where one lies across the old gap
        if on_key(x, y):
            expect("an old gap", x, y, GROUND, yes=False)
        else:
            expect("an old gap, in a gap now", x, y, GROUND)
    if sum(on_key(x, y) for x, y in old) != 21:
        bad.append(f"{sum(on_key(x, y) for x, y in old)} of the old gaps on a key, not 21")
    # The function keys: 0.6 apart from x 87.16 to 132.16, rows 27.7 tall from y 44.0.
    fh, gap = 27.7, 0.6
    for row, (names, fw) in enumerate(((("sound", "pattern", "bpm"), 14.6), (("function", "fx", "play", "write"), 10.8))):
        top = 44.0 + row * (fh + gap)
        for i, name in enumerate(names):
            left = 87.16 + i * (fw + gap)
            for x in (left + inset, left + fw - inset):
                expect(f"{name}'s band", x, top + inset, BANDS[name])
                expect(f"{name}'s foot", x, top + fh - inset, TILE)
    # The caps: 8 by 4.5, in the middle of a box 51.5 tall from y 44.0 (screenshot mode's value).
    cap_w, cap_h = 8.0, 4.5
    cap_top = 44.0 + (51.5 - cap_h) / 2
    for name, x in (("A", 68.0), ("B", 80.0)):
        y = cap_top + cap_h * 0.15
        expect(f"cap {name}'s left edge", x - cap_w / 2 + inset, y, ECRU)
        expect(f"cap {name}'s right edge", x + cap_w / 2 - inset, y, ECRU)
        expect(f"left of cap {name}", x - cap_w / 2 - inset, y, ECRU, yes=False)
        expect(f"right of cap {name}", x + cap_w / 2 + inset, y, ECRU, yes=False)
        expect(f"cap {name}'s foot", x - cap_w / 2 + inset, cap_top + cap_h - inset, ECRU)
    for b in bad:
        print("  " + b)
    print(f"the drawing: {n - len(bad)} of {n} points as D139 draws them ({w} by {h} px, {sx:.2f} px a mm)")
    return not bad


if __name__ == "__main__":
    sys.exit(0 if main(sys.argv[1]) else 1)
