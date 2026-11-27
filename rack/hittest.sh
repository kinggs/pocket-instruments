#!/bin/sh
# Issue 150 (D139): the larger keys answer where they are drawn. Rack's headless mode makes no
# widgets, so this runs Rack's screenshot mode on a scratch build (PI_HIT_TEST, Rhythm.cpp) in
# a throwaway user folder; Kenny's own Rack folder is not touched. Two checks:
#
# 1. The presses. Every key, function key and slider is pressed through Rack's own dispatch at
#    its centre and 0.3 mm inside each corner of the box `design/make_panel.py --positions`
#    gives it, and must answer; a press in a gap between neighbours must reach no widget; a
#    press in the middle of each of the 24 gaps the keys of 11.7 mm at a 12 mm pitch left
#    (until D139) must answer as the table says. 21 of them are on a key now: the three on the
#    old bottom row's centre line (y 86) fall in the new gap between rows 3 and 4, a fact of the
#    ruled pitch found by this test's first run. Rack's log must say "0 not".
# 2. The drawing. The screenshot (a module with no engine, so every key dark) is read back at
#    points worked out from D139's numbers (hittest-pixels.py): each key's face at its corners
#    and in the old gaps, the ground in the gaps now, each slider cap's ecru 8 mm wide.
#
#   hittest.sh [out dir]     the screenshot is copied there as rhythm-hit.png
#
# Needs a desktop (screenshot mode opens a window), Rack (RACK_APP) and uv. Not part of CI.
# SPDX-License-Identifier: MIT
set -eu
here=$(cd "$(dirname "$0")" && pwd)
app=${RACK_APP:-$HOME/.local/opt/Rack2Free-2.6.6}
[ -x "$app/Rack" ] || { echo "hittest: no Rack at $app (set RACK_APP)"; exit 2; }
user=$(mktemp -d)
# make does not rebuild when only a flag changes (docs/LEARNINGS.md): the test's object is
# removed before and after, so neither this build nor the next ordinary one takes the other's.
clean() { rm -f "$here/build/src/Rhythm.cpp.o" "$here/build/hit-points.inc"; rm -rf "$user"; }
trap clean EXIT
rm -f "$here/build/src/Rhythm.cpp.o"
make --no-print-directory -C "$here" install PI_HIT_TEST=1 RACK_USER_DIR="$user" >/dev/null
(cd "$app" && ./Rack -u "$user" -t 3 >/dev/null 2>&1) || { echo "hittest: Rack exited badly"; tail -5 "$user/log.txt"; exit 1; }
grep 'Pocket Instruments hit test' "$user/log.txt" | sed 's/^.*Pocket Instruments/Pocket Instruments/'
line=$(grep 'Pocket Instruments hit test: [0-9]* presses' "$user/log.txt" || true)
[ -n "$line" ] || { echo "hittest: FAILED (the module did not run the hit test)"; exit 1; }
echo "$line" | grep -qE ' [1-9][0-9]* presses answered as make_panel.py --positions says, 0 not; 25 widgets at centre and corners, 24 in the old gaps \(21 on a key now\), 29 in the gaps now' ||
    { echo "hittest: FAILED (a press did not land where the positions table says)"; exit 1; }
shot=$(find "$user/screenshots" -name 'Rhythm.png' | head -1)
[ -n "$shot" ] || { echo "hittest: FAILED (no screenshot of Rhythm)"; exit 1; }
[ -z "${1:-}" ] || { mkdir -p "$1"; cp "$shot" "$1/rhythm-hit.png"; }
python3 -I "$here/hittest-pixels.py" "$shot" || { echo "hittest: FAILED (the drawing is not D139's)"; exit 1; }
echo "hittest: every key, function key and slider answers at its centre and corners as make_panel.py --positions says; the old gaps are keys; the drawing holds D139's sizes."
