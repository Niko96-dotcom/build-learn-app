#!/usr/bin/env python3
"""Renders the app icon to a 1024 px PNG with a transparent margin.

A dark rounded square with a 4 x 4 grid of quiet dots, three bright dots and
one accent dot: one bar of practice, one hit in the right place. Same palette
as the app (theme.rs). Signed-distance anti-aliasing, stdlib only. Do not use
Quick Look to render SVG icons: it adds a white background. Change MARKS to
draw the app's own subject; keep the palette, no glow, no gradient. Look at
the PNG before bundling.

Usage: tools/make-icon.py [out.png]"""
import math
import struct
import sys
import zlib

S = 1024
OUT = sys.argv[1] if len(sys.argv) > 1 else "assets/icon/icon_1024.png"
BG = (0x0A, 0x0A, 0x0B)
EDGE = (0x2B, 0x2B, 0x31)
DIM = (0x38, 0x38, 0x3E)
INK = (0xF2, 0xF2, 0xF3)
ACC = (0xFF, 0x4D, 0x2E)
X0, Y0, W, R = 100.0, 100.0, 824.0, 190.0
STEP = 132.0
GRID0 = S / 2 - 1.5 * STEP
# (column, row, colour, radius) of each dot; later dots paint over earlier ones.
MARKS = [(c, r, DIM, 22.0) for r in range(4) for c in range(4)]
MARKS += [(0, 3, INK, 30.0), (1, 2, INK, 30.0), (3, 0, INK, 30.0)]
MARKS += [(2, 1, ACC, 40.0)]


def rrect_sd(x, y, cx, cy, hw, hh, r):
    qx, qy = abs(x - cx) - (hw - r), abs(y - cy) - (hh - r)
    return math.hypot(max(qx, 0), max(qy, 0)) + min(max(qx, qy), 0) - r


def cov(d):
    return max(0.0, min(1.0, 0.5 - d))


def over(dst, src, a):
    return tuple(dst[i] * (1 - a) + src[i] * a for i in range(3))


def render():
    rows = []
    for py in range(S):
        row = bytearray([0])
        y = py + 0.5
        for px in range(S):
            x = px + 0.5
            d = rrect_sd(x, y, X0 + W / 2, Y0 + W / 2, W / 2, W / 2, R)
            a_shape = cov(d)
            if a_shape <= 0:
                row += b"\x00\x00\x00\x00"
                continue
            c = over(BG, EDGE, max(0.0, min(1.0, 1.5 - abs(d + 1.5))) * 0.9)
            for col, rw, colour, rad in MARKS:
                cx, cy = GRID0 + col * STEP, GRID0 + rw * STEP
                if abs(x - cx) < rad + 2 and abs(y - cy) < rad + 2:
                    c = over(c, colour, cov(math.hypot(x - cx, y - cy) - rad))
            row += bytes([round(c[0]), round(c[1]), round(c[2]), round(255 * a_shape)])
        rows.append(bytes(row))
    return rows


def chunk(t, data):
    return struct.pack(">I", len(data)) + t + data + struct.pack(">I", zlib.crc32(t + data) & 0xFFFFFFFF)


def main():
    raw = b"".join(render())
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", S, S, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    with open(OUT, "wb") as f:
        f.write(png)
    print(OUT)


if __name__ == "__main__":
    main()
