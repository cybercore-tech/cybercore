#!/usr/bin/env python3
"""Map a 16-colour terminal theme onto cybercore's 11 semantic roles.

Input shapes accepted (Darkstar's theme-gen / sub-cyber format):

    { "name": "...", "background": "#..", "foreground": "#..",
      "cursor": "#..", "colors": ["#..", x16] }        # or "color"

Usage:
    ansi2cybergrid.py THEME.json [--slug my-theme]              # print one entry
    ansi2cybergrid.py A.json B.json ... --write schema/themes/dystopian

Since v0.4.0, cybercore's theme data is chunked: one 11-role palette per
file under schema/themes/<family>/<slug>.json, merged into the embedded
schema by build.rs at compile time. `--write DIR` drops one file per input
theme straight into that family folder — no single cybergrid.json to merge
into anymore. Refuses to overwrite an existing file unless `--force` is
given, and never touches build.rs or cybergrid.json itself.

These palettes don't follow the black/red/green/... ANSI ordering — they're
artistic gradients — so roles are assigned by **hue classification** of every
candidate colour (the 16 + foreground + cursor), picking the most vivid match
for each role and synthesising anything missing from a neighbour.
"""
from __future__ import annotations

import argparse
import colorsys
import json
import os
import re
import sys
from collections import OrderedDict


def _hex(s: str) -> str:
    s = (s or "").strip().lstrip("#").lower()
    if re.fullmatch(r"[0-9a-f]{3}", s):
        s = "".join(c * 2 for c in s)
    if not re.fullmatch(r"[0-9a-f]{6}", s):
        raise ValueError(f"not a hex colour: {s!r}")
    return s


def _rgb(h):            # 0..1 tuple
    h = _hex(h)
    return tuple(int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))


def _hexof(rgb):
    return "".join(f"{max(0, min(255, round(c * 255))):02x}" for c in rgb)


def _hsl(h):
    r, g, b = _rgb(h)
    hh, ll, ss = colorsys.rgb_to_hls(r, g, b)
    return hh * 360, ss, ll


def _mix(a, b, t):
    ra = _rgb(a)
    rb = _rgb(b)
    return _hexof(tuple(x + (y - x) * t for x, y in zip(ra, rb)))


def _rotate(h, deg, *, s=None, l=None):
    hh, ss, ll = _hsl(h)
    r, g, b = colorsys.hls_to_rgb(((hh + deg) % 360) / 360,
                                  ll if l is None else l,
                                  ss if s is None else s)
    return _hexof((r, g, b))


# hue windows (deg) for each accent role
_HUES = {
    "red":        [(348, 361), (0, 12)],
    "orange":     [(12, 45)],
    "acid_green": [(70, 165)],
    "cyan":       [(165, 210)],
    "purple":     [(245, 292)],
    "hot_pink":   [(292, 348)],
}


def _in(hue, windows):
    return any(lo <= hue < hi for lo, hi in windows)


def convert(theme: dict) -> "OrderedDict[str, str]":
    cols = theme.get("colors") or theme.get("color") or []
    cols = [_hex(c) for c in cols]
    if len(cols) < 16:
        raise ValueError(f"expected 16 colours, got {len(cols)}")

    bg = _hex(theme.get("background") or cols[0])
    fg = theme.get("foreground")
    cursor = theme.get("cursor")

    cands = list(cols)
    for extra in (fg, cursor):
        if extra:
            try:
                cands.append(_hex(extra))
            except ValueError:
                pass

    bg_l = _hsl(bg)[2]

    # --- neutrals: chosen by lightness + low saturation, relative to bg ---
    def lowsat_near(target_l, tol):
        best, bd = None, 99
        for c in cands:
            _, s, l = _hsl(c)
            if s > 0.30:
                continue
            d = abs(l - target_l)
            if d < bd and d <= tol:
                best, bd = c, d
        return best

    panel = lowsat_near(bg_l + 0.05, 0.10) or _mix(bg, "ffffff", 0.06)
    if _hsl(panel)[2] <= bg_l:            # must read as raised
        panel = _mix(bg, "ffffff", 0.06)
    line = lowsat_near(bg_l + 0.16, 0.14) or _mix(bg, "ffffff", 0.17)
    muted = lowsat_near(0.55, 0.22) or _mix("ffffff", bg, 0.42)
    # white = lightest candidate that isn't a screaming accent
    white = max(
        (c for c in cands if _hsl(c)[1] < 0.55),
        key=lambda c: _hsl(c)[2],
        default=_mix(bg, "ffffff", 0.9),
    )
    if _hsl(white)[2] < 0.75:
        white = _mix(white, "ffffff", 0.4)

    # --- accents: most-saturated candidate whose hue lands in the window ---
    def role(name):
        pool = [c for c in cands if _in(_hsl(c)[0], _HUES[name]) and _hsl(c)[1] > 0.25]
        if pool:
            return max(pool, key=lambda c: _hsl(c)[1] * (0.6 + 0.4 * _hsl(c)[2]))
        return None

    picked = {k: role(k) for k in _HUES}

    # fill gaps by rotating a present neighbour to the target hue
    _neighbour = {
        "red": "orange", "orange": "red", "acid_green": "cyan", "cyan": "acid_green",
        "purple": "hot_pink", "hot_pink": "purple",
    }
    _target_hue = {"red": 353, "orange": 28, "acid_green": 95,
                   "cyan": 185, "purple": 270, "hot_pink": 320}
    for k, v in picked.items():
        if v:
            continue
        src = picked.get(_neighbour[k]) or next((x for x in picked.values() if x), None) or white
        cur = _hsl(src)[0]
        picked[k] = _rotate(src, _target_hue[k] - cur, s=max(_hsl(src)[1], 0.7),
                            l=min(max(_hsl(src)[2], 0.5), 0.65))

    # cursor often *is* the signature pink
    if cursor:
        try:
            ch = _hsl(_hex(cursor))
            if _in(ch[0], _HUES["hot_pink"]) and ch[1] > 0.4:
                picked["hot_pink"] = _hex(cursor)
        except ValueError:
            pass
    if picked["purple"] == picked["hot_pink"]:
        picked["purple"] = _rotate(picked["hot_pink"], -40, s=0.7)

    return OrderedDict(
        bg=bg, white=white,
        acid_green=picked["acid_green"], hot_pink=picked["hot_pink"],
        purple=picked["purple"], cyan=picked["cyan"],
        orange=picked["orange"], red=picked["red"],
        panel=panel, line=line, muted=muted,
    )


def _slug(name):
    s = re.sub(r"[^a-z0-9]+", "-", (name or "theme").lower()).strip("-")
    return s or "theme"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="+")
    ap.add_argument("--slug")
    ap.add_argument("--write", metavar="schema/themes/<family>",
                     help="Write one <slug>.json per theme into this family "
                          "directory instead of printing to stdout.")
    ap.add_argument("--force", action="store_true",
                     help="With --write, overwrite a file that already exists "
                          "instead of skipping it.")
    args = ap.parse_args()

    entries: "OrderedDict[str, OrderedDict[str, str]]" = OrderedDict()
    for path in args.files:
        with open(path) as fh:
            data = json.load(fh)
        slug = args.slug if (args.slug and len(args.files) == 1) else _slug(data.get("name") or path)
        entries[slug] = convert(data)

    if args.write:
        os.makedirs(args.write, exist_ok=True)
        written, skipped = [], []
        for slug, pal in entries.items():
            out_path = os.path.join(args.write, f"{slug}.json")
            if os.path.exists(out_path) and not args.force:
                skipped.append(slug)
                continue
            with open(out_path, "w") as fh:
                json.dump(pal, fh, indent=2)
                fh.write("\n")
            written.append(slug)
        if written:
            print(f"wrote {len(written)} to {args.write}/: {', '.join(written)}")
        if skipped:
            print(f"skipped {len(skipped)} (already exist, use --force to overwrite): "
                  f"{', '.join(skipped)}", file=sys.stderr)
        if not written and skipped:
            return 1
    else:
        for slug, pal in entries.items():
            print(json.dumps({slug: pal}, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
