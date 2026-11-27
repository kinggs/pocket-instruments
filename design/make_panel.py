# /// script
# requires-python = ">=3.10"
# dependencies = ["fonttools"]
# ///
"""Draw the static Rack panel for the Loom design as a flat SVG, in millimetres.

    uv run design/make_panel.py              writes panel.svg beside this file
    uv run design/make_panel.py --positions  prints the widget positions
    uv run design/make_panel.py --check      no label, plate or stitch touches its neighbour

The panel holds only what never changes: the body, the display's frame, the stitched
dividers, the slider threads, a pale plate under each output, the stitches round the
sixteen-channel jacks and the lettering. Keys, slider caps, the display and the jack
sockets are widgets the module draws over it (DESIGN.md, "The Rack panel").

Why a script and not a drawing: Rack's SVG reader draws no text, no dashes, no filters. So
lettering is turned into outlines here, and each stitch of a divider is its own rectangle.
Every number below is in mm from the panel's top-left corner.
"""
import math
import sys
from pathlib import Path

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

# The design's cut, ExtraBold (800), kept in the repo with its licence (fonts/OFL.txt) since
# Session 13; the Rack module draws its lettering in the same file. Until then this was the
# machine's Bold (700), the only cut installed. Pass another font file as the last argument.
FONT = str(Path(__file__).with_name("fonts") / "JetBrainsMono-ExtraBold.ttf")

# Layout B at 27 HP (D114, D119 item 1; issue 101). Until then 24 HP, 121.92 mm, with two rows
# of jacks under the keys (D76): every position below that moved says what it was.
W, H = 137.16, 128.5

GROUND, DISPLAY, ECRU, THREAD_C = "#14131F", "#0B0A12", "#F2E8D0", "#6E6A92"

MARGIN = 5.0
FRAME = 0.35
# The display's glass runs from 17 mm to 17 mm short of the right edge (D114), with CLOCK and
# RESET to its left and AUDIO to its right. The frame is FRAME outside the glass. Was
# (MARGIN, 7.0, W - MARGIN - 4.92, 25.0) at 24 HP.
GLASS_IN = 17.0
DISPLAY_BOX = (GLASS_IN - FRAME, 7.0, W - 2 * GLASS_IN + 2 * FRAME, 25.0)  # x, y, w, h

# The keys, the sliders and the function keys keep their places among themselves, 6 mm lower
# and centred: 7.62 mm right of where they were at 24 HP (half of three HP).
DX, DY = (W - 121.92) / 2, 6.0
MID_Y, MID_H = 38.0 + DY, 48.0  # the band the grid, sliders and function keys share

GRID_X, PITCH, KEY = 5.0 + DX, 12.0, 11.7  # 4 x 4 at 12 mm pitch
# The sliders were 7 mm apart, at 61.5 and 68.5, with the dividers at 56 and 74. Kenny
# widened them to 10 mm (2026-11-19) for the guide's thumb room (R33). The space came from the
# clearances beside the dividers, so the grid and the function keys did not move.
DIVIDERS = (55.25 + DX, 74.75 + DX)
SLIDERS = {"A": 60.0 + DX, "B": 70.0 + DX}  # centre lines
SLIDER_TOP, SLIDER_BOTTOM = 38.0 + DY, 81.0 + DY
THREAD = 0.67
LETTER_Y = 84.2 + DY  # centre of the A and B under the threads

FN_X, FN_W, FN_GAP = 77.0 + DX, 40.0, 0.6  # function keys: 10 mm pitch along the bottom row
FN_TOP = ("sound", "pattern", "bpm")
FN_BOTTOM = ("function", "fx", "play", "write")

# The jacks: two rows, thirteen jacks, grouped by what they carry (D76; drawn in Session 18).
# Until then one row of six at y 116.3: CLOCK, RESET and RETURN on the dark body at the left,
# TRIGGERS, VOICES and AUDIO on one pale plate at the right (D62).
#
# D76 proposed positions by arithmetic. What moved once it was drawn, and why (D80): the left
# three columns to 12.4, 25.4 and 38.4 (TRIGGERS' lettering touched VOICES'); FX SEND and FX
# RETURN to 84.0 and 96.0 with FX over the word; FX GATES to 90.0. Session 24 (D87, D88)
# stitched FX SEND and FX RETURN; nothing moved.
#
# Layout B (D114, D119 items 1 and 2; issue 101): CLOCK over RESET left of the display, AUDIO
# right of it, and the ten sixteen-channel jacks in one row of four groups, each a group's name
# over a short word. Port numbers do not change (docs/RACK-RULES.md); only the picture moves.
# The design session's arithmetic (design/DESIGN.md, "Layout B") put the row's ends 2.0 mm from
# the panel's edges, inside this file's 5 mm margin, which RACK-RULES.md holds every drawing to.
# So the row is laid out here by the same pitches, centred, and the gap between groups is what
# is left: `--check` prints it. ⚠ That gap is the build's, not the design's 6.95 mm.
ROW = 116.3
LABEL_ABOVE = 6.6   # a label's centre above its socket's
LINE = 3.1          # between the two lines of a two-line label
PITCH_OUT_IN, PITCH_OUT_OUT = 11.3, 11.2  # stitches 5.2 and a plate 5.6 from a centre, 0.5 clear
# Each group: its name, then (name in code, is an output, the word on the panel).
GROUPS = (
    ("SOUNDS", (("TRIGGERS", True, "TRIG"), ("VOICES", True, "OUT"), ("RETURN", False, "IN"))),
    ("SLIDER A", (("A OUT", True, "OUT"), ("A IN", False, "IN"))),
    ("SLIDER B", (("B OUT", True, "OUT"), ("B IN", False, "IN"))),
    ("FX", (("FX GATES", True, "GATES"), ("FX SEND", True, "SEND"), ("FX RETURN", False, "RETURN"))),
)
GROUP_NAME_Y, GROUP_RULE_Y, GROUP_RULE_H = 103.3, 105.6, 0.5  # the name, and a thread rule under it


def row_layout(gap):
    """The ten jacks' centres for a gap between groups: [(name, x)]."""
    xs, x, prev = [], 0.0, None
    for gi, (_, jacks) in enumerate(GROUPS):
        for ji, (name, out, _) in enumerate(jacks):
            if prev is not None:
                x += (PITCH_OUT_OUT if prev and out else PITCH_OUT_IN) + (gap if ji == 0 else 0.0)
            xs.append((name, x))
            prev = out
    return xs


def place_row():
    """The gap that fills the row from margin to margin: the first output's plate and the
    last input's label or stitches each end on the margin. Widths are the font's (label_w)."""
    span0 = row_layout(0.0)
    first, last = span0[0][1], span0[-1][1]
    left_reach = PLATE_HALF  # TRIGGERS' plate (its word is narrower than the plate)
    right_reach = max(TICK_OUT, RETURN_LABEL_HALF)
    room = (W - 2 * MARGIN) - (left_reach + (last - first) + right_reach)
    gap = room / (len(GROUPS) - 1)
    x0 = MARGIN + left_reach
    return gap, [(n, x0 + x) for n, x in row_layout(gap)]


# Each output on its own patch of the inverted ground (D76 item 3; VCV's panel guide asks for
# an inverted background on output ports, R33). A patch is as wide as its stitches or its
# label, whichever is wider, so TRIGGERS and VOICES, 13 mm apart, share an edge and read as
# one plate. Until Session 18 one plate lay under all three outputs at the right.
PLATE_HALF = 5.6                    # the stitches reach 5.2 mm from the centre
PLATE_UP, PLATE_DOWN = 1.7, 5.7     # above the top label line's centre; below the socket's
LABEL_PAD = 0.8                     # plate past its label, each side
# AUDIO's plate had a half of 6.9 mm, wider than the rest. At 27 HP in layout B that reached 2.6 mm
# from the edge and 0.25 mm from the display's frame; Kenny chose a narrower plate
# (2026-11-26, issue 101): as wide as the room between the margin and the frame leaves, 0.5 mm
# clear of the frame. It keeps its inner frame: the main output.
AUDIO_PLATE_HALF = (GLASS_IN - FRAME - 0.5 - MARGIN) / 2
AUDIO_FRAME_IN, AUDIO_FRAME_T = 0.6, 0.6  # 0.6 mm inside it, 0.6 mm thick: the main output
TICK_IN, TICK_OUT, TICK_W = 4.4, 5.2, 0.5
# "RETURN" at 2.6 mm, tracked, is wider than its stitches: half its width (font-measured by
# --check, which fails if this is short).
RETURN_LABEL_HALF = 5.2

# name: (x, y, is an output, sixteen channels, the label's lines)
JACKS = {
    "CLOCK": (9.5, 13.5, False, False, ("CLOCK",)),
    "RESET": (9.5, 27.5, False, False, ("RESET",)),
    "AUDIO": (W - MARGIN - (GLASS_IN - FRAME - 0.5 - MARGIN) / 2, 21.0, True, False, ("AUDIO",)),  # was W - 9.5 on the board
}

# The wordmark sat at x 63 until the outputs became three (2026-11-19), then at x 49, 3.5 mm,
# until RETURN (D62), when it dropped to 3.0 mm to fit between RETURN's stitches and the
# output plate. Since Session 18 it sits midway between RETURN's stitches and FX SEND's plate,
# still 3.0 mm. `--check` prints the clearances.
WORDMARK_SIZE = 3.0
# Layout B: no room in the full row, so the wordmark sits centred in the band between the keys'
# bottom (and the A and B under the threads) and the group names. ⚠ Its place is the build's:
# the board did not draw it (design/DESIGN.md, "Layout B").
WORDMARK_Y = None  # worked out by wordmark()

# Jack lettering was 2.17 mm. At Rack's 100% zoom (about 2.95 px per mm) that is under 5 px of
# capital height, and the guide asks for text readable at 100% (R33). 2.6 mm is about 5.6 px.
JACK_TYPE, AUDIO_TYPE, JACK_TRACKING = 2.6, 2.8, 0.04

ROW_GAP, _row = place_row()
_x = dict(_row)
for _g, _jacks in GROUPS:
    for _name, _out, _word in _jacks:
        JACKS[_name] = (_x[_name], ROW, _out, True, (_word,))
GROUP_OF = {name: g for g, jacks in GROUPS for name, _, _ in jacks}

# Rack's four screws (RACK_GRID_WIDTH, 15 px = 5.08 mm square, at the corners one HP in).
SCREW = 5.08
SCREWS = ((SCREW, 0.0), (W - 2 * SCREW, 0.0), (SCREW, H - SCREW), (W - 2 * SCREW, H - SCREW))

# The display's right column, as Rhythm.cpp's RhythmDisplay draws it: 2.67 mm in, the tempo at
# 10 mm left-aligned RIGHT_COL in from that edge, "bpm" at 2.5 mm right-aligned on the edge,
# 3 mm between the weave and the column. --check says what is left for the weave.
DISPLAY_PAD, RIGHT_COL, WEAVE_GAP, TEMPO_TYPE, BPM_TYPE = 2.67, 25.0, 3.0, 10.0, 2.5

def num(v):
    return f"{v:.3f}".rstrip("0").rstrip(".")


def rect(x, y, w, h, fill):
    return f'<rect x="{num(x)}" y="{num(y)}" width="{num(w)}" height="{num(h)}" fill="{fill}"/>'


def outline(font, text, cx, cy, size, fill, tracking=0.0):
    """The text as one filled path, centred on (cx, cy) by its capital height."""
    glyphs, cmap = font.getGlyphSet(), font.getBestCmap()
    scale = size / font["head"].unitsPerEm
    cap = font["OS/2"].sCapHeight * scale
    names = [cmap[ord(c)] for c in text]
    widths = [glyphs[n].width * scale + tracking * size for n in names]
    x = cx - (sum(widths) - tracking * size) / 2
    baseline = cy + cap / 2
    parts = []
    for name, width in zip(names, widths):
        pen = SVGPathPen(glyphs, ntos=num)
        glyphs[name].draw(TransformPen(pen, (scale, 0, 0, -scale, x, baseline)))
        parts.append(pen.getCommands())
        x += width
    return f'<path fill="{fill}" d="{"".join(parts)}"/>'


def ticks(cx, cy, fill):
    """Sixteen stitches round a socket, as one path. No rotate(): corners are worked out here."""
    d = []
    for k in range(16):
        a = math.tau * k / 16
        rx, ry = math.sin(a), -math.cos(a)  # outward
        tx, ty = -ry, rx  # along the tick's width
        pts = [(cx + rx * r + tx * s * TICK_W / 2, cy + ry * r + ty * s * TICK_W / 2)
               for r, s in ((TICK_IN, -1), (TICK_OUT, -1), (TICK_OUT, 1), (TICK_IN, 1))]
        d.append("M" + "L".join(f"{num(x)} {num(y)}" for x, y in pts) + "Z")
    return f'<path fill="{fill}" d="{"".join(d)}"/>'


def fn_keys():
    """(label, x, y, w, h) for the seven function keys."""
    h = (MID_H - FN_GAP) / 2
    out = []
    for row, labels in enumerate((FN_TOP, FN_BOTTOM)):
        w = (FN_W - FN_GAP * (len(labels) - 1)) / len(labels)
        for i, label in enumerate(labels):
            out.append((label, FN_X + i * (w + FN_GAP), MID_Y + row * (h + FN_GAP), w, h))
    return out


def advance(font, text, size, tracking=0.0):
    """The text's width in mm, by its glyphs' advances, as outline() lays it out."""
    glyphs, cmap = font.getGlyphSet(), font.getBestCmap()
    scale = size / font["head"].unitsPerEm
    return sum(glyphs[cmap[ord(c)]].width * scale + tracking * size for c in text) - tracking * size


def label_box(font, name):
    """A jack's lettering: (left, right, top, bottom) in mm, over all its lines."""
    x, y, _, _, lines = JACKS[name]
    size = AUDIO_TYPE if name == "AUDIO" else JACK_TYPE
    w = max(advance(font, line, size, JACK_TRACKING) for line in lines)
    cap = font["OS/2"].sCapHeight * size / font["head"].unitsPerEm
    top_line = y - LABEL_ABOVE - LINE * (len(lines) - 1)
    return x - w / 2, x + w / 2, top_line - cap / 2, y - LABEL_ABOVE + cap / 2


def plate(font, name):
    """An output's patch of inverted ground: (x, y, w, h)."""
    x, y, _, _, lines = JACKS[name]
    left, right, _, _ = label_box(font, name)
    half = AUDIO_PLATE_HALF if name == "AUDIO" else max(PLATE_HALF, (right - left) / 2 + LABEL_PAD)
    rows = 2 if name == "AUDIO" else len(lines)  # AUDIO's is two lines tall, its label clear of its frame
    top = y - LABEL_ABOVE - LINE * (rows - 1) - PLATE_UP
    return x - half, top, 2 * half, y + PLATE_DOWN - top


def extent(font, name):
    """Everything a jack draws, left to right: its label, and its plate or its stitches."""
    x, _, out, multi, _ = JACKS[name]
    left, right, _, _ = label_box(font, name)
    if out:
        px, _, pw, _ = plate(font, name)
        return min(left, px), max(right, px + pw)
    reach = TICK_OUT if multi else 4.0
    return min(left, x - reach), max(right, x + reach)


def wordmark(font):
    """The wordmark's (text, centre x, y, size, the band it sits in): centred on the panel,
    midway between the lowest of the keys and the A and B, and the group names' tops."""
    cap = font["OS/2"].sCapHeight * JACK_TYPE / font["head"].unitsPerEm
    top = max(MID_Y + MID_H, LETTER_Y + 3.0 * 0.73 / 2)
    bottom = GROUP_NAME_Y - cap / 2
    return "Pocket Instruments", W / 2, (top + bottom) / 2, WORDMARK_SIZE, (top, bottom)


def cap_of(font, size):
    return font["OS/2"].sCapHeight * size / font["head"].unitsPerEm


def jack_box(font, name):
    """Everything a jack draws, as (left, right, top, bottom)."""
    x, y, out, multi, _ = JACKS[name]
    l, r = extent(font, name)
    _, _, ltop, _ = label_box(font, name)
    if out:
        _, py, _, ph = plate(font, name)
        return l, r, min(ltop, py), py + ph
    reach = TICK_OUT if multi else 4.0
    return l, r, ltop, y + reach


def check(font):
    """Nothing a jack draws touches its neighbour's in the row, except two outputs' plates,
    which may share an edge; no lettering within 1 mm of its neighbour's; everything inside the
    panel's margin and clear of the screws and the display's frame; the group names clear of
    each other; the wordmark clear of the keys and the names (D62 item 4, D76, D114)."""
    ok = True
    names = [n for n, _ in _row]
    print(f"the row at {ROW}: groups {ROW_GAP:.2f} mm further apart than the jacks within a group "
          f"(the design's arithmetic had 6.95 at 27 HP, with the row 2.0 mm from the edges)")
    for a, b in zip(names, names[1:]):
        gap = extent(font, b)[0] - extent(font, a)[1]
        both_out = JACKS[a][2] and JACKS[b][2]
        letters = label_box(font, b)[0] - label_box(font, a)[1]
        good = letters >= 1.0 - 1e-6 and (gap >= 0.5 - 1e-6 or both_out)
        ok = ok and good
        print(f"  {a} to {b}: {gap:.2f} mm between them, {letters:.2f} mm between their lettering" + ("" if good else "  <-- too close"))
    edge_l, edge_r = extent(font, names[0])[0], W - extent(font, names[-1])[1]
    good = edge_l >= MARGIN - 0.01 and edge_r >= MARGIN - 0.01
    ok = ok and good
    print(f"  {edge_l:.2f} mm from the left edge, {edge_r:.2f} mm from the right (the margin is {MARGIN})" + ("" if good else "  <-- past the margin"))
    # The group names and their rules.
    spans = []
    for g, jacks in GROUPS:
        boxes = [extent(font, n) for n, _, _ in jacks]
        left, right = min(b[0] for b in boxes), max(b[1] for b in boxes)
        w = advance(font, g, JACK_TYPE, JACK_TRACKING)
        spans.append((g, (left + right) / 2 - w / 2, (left + right) / 2 + w / 2, left, right))
    for (ga, _, ra, _, rra), (gb, lb, _, llb, _) in zip(spans, spans[1:]):
        good = lb - ra >= 1.0 and llb - rra >= 0.5
        ok = ok and good
        print(f"  {ga} to {gb}: names {lb - ra:.2f} mm apart, rules {llb - rra:.2f} mm" + ("" if good else "  <-- too close"))
    cap = cap_of(font, JACK_TYPE)
    under = min(jack_box(font, n)[2] for n in names) - (GROUP_RULE_Y + GROUP_RULE_H)
    over = GROUP_RULE_Y - (GROUP_NAME_Y + cap / 2)
    good = under >= 0.5 and over >= 0.5
    ok = ok and good
    print(f"  the rules are {over:.2f} mm under the names and {under:.2f} mm over the jacks' lettering and plates" + ("" if good else "  <-- too close"))
    # The side jacks: CLOCK over RESET, AUDIO; against the margin, the screws and the display.
    dx, dy, dw, dh = DISPLAY_BOX
    for name in ("CLOCK", "RESET", "AUDIO"):
        l, r, t, b = jack_box(font, name)
        to_display = (dx - r) if name != "AUDIO" else (l - (dx + dw))
        to_edge = l if name != "AUDIO" else W - r
        screw = min(max(sx - r, l - (sx + SCREW), sy - b, t - (sy + SCREW)) for sx, sy in SCREWS)
        good = to_display >= 0.5 - 1e-6 and to_edge >= MARGIN - 0.01 and screw >= 0.5
        ok = ok and good
        print(f"{name}: {to_edge:.2f} mm from the edge, {to_display:.2f} mm from the display's frame, {screw:.2f} mm from the nearest screw"
              + ("" if good else "  <-- too close"))
    _, _, _, rb = jack_box(font, "CLOCK")
    _, _, rt, _ = jack_box(font, "RESET")
    good = rt - rb >= 0.5
    ok = ok and good
    print(f"CLOCK to RESET: {rt - rb:.2f} mm" + ("" if good else "  <-- too close"))
    for name in names:
        l, r, t, b = jack_box(font, name)
        screw = min(max(sx - r, l - (sx + SCREW), sy - b, t - (sy + SCREW)) for sx, sy in SCREWS)
        if screw < 0.5:
            ok = False
            print(f"{name}: {screw:.2f} mm from a screw  <-- too close")
    text, x, y, size, (top, bottom) = wordmark(font)
    w = advance(font, text, size)
    c = cap_of(font, size)
    good = (y - c / 2) - top >= 1.0 and bottom - (y + c / 2) >= 1.0 and x - w / 2 >= MARGIN
    ok = ok and good
    print(f"wordmark {size} mm at y {y:.2f}: {(y - c / 2) - top:.2f} mm under the keys, {bottom - (y + c / 2):.2f} mm over the group names"
          + ("" if good else "  <-- too close"))
    # The display's inside, as the module draws it (not the panel's, but laid out from its width).
    glass = dw - 2 * FRAME
    weave = glass - 2 * DISPLAY_PAD - RIGHT_COL - WEAVE_GAP
    tempo = advance(font, "240", TEMPO_TYPE)
    label = advance(font, "bpm", BPM_TYPE)
    room = RIGHT_COL - tempo - label
    good = room >= 0.5
    ok = ok and good
    print(f"the display: glass {glass:.2f} mm wide; the weave {weave:.2f} mm for its 78 cells ({weave / 78:.3f} mm a cell; 1.000 at 24 HP); "
          f"the tempo's column {RIGHT_COL} mm holds 240 ({tempo:.2f} mm) and bpm ({label:.2f} mm) with {room:.2f} mm between"
          + ("" if good else "  <-- too close"))
    return ok


def positions():
    rows = ["| Widget | Centre x | Centre y | Width | Height |", "|---|---|---|---|---|"]

    def row(name, cx, cy, w, h):
        rows.append(f"| {name} | {cx:.2f} | {cy:.2f} | {w:.2f} | {h:.2f} |")

    dx, dy, dw, dh = DISPLAY_BOX
    row("display (inside the frame)", dx + dw / 2, dy + dh / 2, dw - 2 * FRAME, dh - 2 * FRAME)
    for n in range(16):
        r, c = divmod(n, 4)
        row(f"key {n + 1}", GRID_X + PITCH / 2 + c * PITCH, MID_Y + PITCH / 2 + r * PITCH, KEY, KEY)
    for name, x in SLIDERS.items():
        row(f"slider {name} (travel)", x, (SLIDER_TOP + SLIDER_BOTTOM) / 2, 5.0, SLIDER_BOTTOM - SLIDER_TOP)
    for label, x, y, w, h in fn_keys():
        row(label, x + w / 2, y + h / 2, w, h)
    for name, (x, y, out, multi, _) in JACKS.items():
        row(f"jack {name} ({'out' if out else 'in'}{', 16 channels' if multi else ''})", x, y, 8.0, 8.0)
    return "\n".join(rows)


def panel(font):
    dx, dy, dw, dh = DISPLAY_BOX
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}mm" height="{H}mm" viewBox="0 0 {W} {H}">',
        rect(0, 0, W, H, GROUND),
        rect(dx, dy, dw, dh, ECRU),
        rect(dx + FRAME, dy + FRAME, dw - 2 * FRAME, dh - 2 * FRAME, DISPLAY),
    ]
    for x in DIVIDERS:  # a stitch every 2.5 mm
        y = MID_Y
        while y + 1.5 <= MID_Y + MID_H + 0.001:
            out.append(rect(x - 0.25, y, 0.5, 1.5, ECRU))
            y += 2.5
    for name, x in SLIDERS.items():
        out.append(rect(x - THREAD / 2, SLIDER_TOP, THREAD, SLIDER_BOTTOM - SLIDER_TOP, ECRU))
        out.append(outline(font, name, x, LETTER_Y, 3.0, ECRU))
    for name, (x, y, is_out, multi, lines) in JACKS.items():
        if is_out:
            out.append(rect(*plate(font, name), ECRU))
    px, py, pw, ph = plate(font, "AUDIO")
    fx, fy, fw, fh, ft = px + AUDIO_FRAME_IN, py + AUDIO_FRAME_IN, pw - 2 * AUDIO_FRAME_IN, ph - 2 * AUDIO_FRAME_IN, AUDIO_FRAME_T
    for r in ((fx, fy, fw, ft), (fx, fy + fh - ft, fw, ft), (fx, fy, ft, fh), (fx + fw - ft, fy, ft, fh)):
        out.append(rect(*r, GROUND))
    for name, (x, y, is_out, multi, lines) in JACKS.items():
        fill = GROUND if is_out else ECRU
        if multi:
            out.append(ticks(x, y, fill))
        size = AUDIO_TYPE if name == "AUDIO" else JACK_TYPE
        for i, line in enumerate(lines):
            out.append(outline(font, line, x, y - LABEL_ABOVE - LINE * (len(lines) - 1 - i), size, fill, tracking=JACK_TRACKING))
    for g, jacks in GROUPS:  # a group's name over a thread rule (D119 item 2)
        boxes = [extent(font, n) for n, _, _ in jacks]
        left, right = min(b[0] for b in boxes), max(b[1] for b in boxes)
        out.append(rect(left, GROUP_RULE_Y, right - left, GROUP_RULE_H, THREAD_C))
        out.append(outline(font, g, (left + right) / 2, GROUP_NAME_Y, JACK_TYPE, ECRU, tracking=JACK_TRACKING))
    text, x, y, size, _ = wordmark(font)
    out.append(outline(font, text, x, y, size, ECRU))
    out.append("</svg>")
    return "\n".join(out) + "\n"


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if "--positions" in sys.argv:
        print(positions())
    elif "--check" in sys.argv:
        sys.exit(0 if check(TTFont(args[0] if args else FONT)) else 1)
    else:
        target = Path(__file__).with_name("panel.svg")
        target.write_text(panel(TTFont(args[0] if args else FONT)))
        print(f"wrote {target}")
