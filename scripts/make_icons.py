"""
Professional B&W constellation icon set — BOLD EDITION.

Design: equilateral triangle of three agent-nodes + links.
Key fix: much bolder proportions so the icon is VISIBLE at 16-32px,
not an invisible gray smudge.

Changes from previous:
  - fig_r 30% -> 38% of canvas (figure fills most of the space)
  - node_r 18% -> 30% of fig_r (nodes are chunky dots)
  - stroke  8.5% -> 14% of fig_r (strokes are thick lines)
  - Smaller sizes get progressive boldness boost (optical correction)
  - Minimum feature sizes enforced in supersampled coordinates
"""

from __future__ import annotations

import io
import math
import struct
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]


def _equilateral_points(
    cx: float, cy: float, radius: float
) -> tuple[tuple[float, float], tuple[float, float], tuple[float, float]]:
    """Three vertices of equilateral triangle, flat base, point up, around center."""
    pts = []
    for deg in (-90.0, 30.0, 150.0):
        rad = math.radians(deg)
        pts.append((cx + radius * math.cos(rad), cy + radius * math.sin(rad)))
    return pts[0], pts[1], pts[2]  # top, br, bl


def draw_mark(size: int) -> Image.Image:
    """
    Draw logo at exact pixel size with optical-size-corrected boldness.

    The ratios are chosen so the smallest sizes (16-32px) are chunky
    and recognizable, while larger sizes (128-512px) stay elegant.
    """
    # ---- per-size boldness correction ----
    # Small sizes need proportionally bolder features to be visible.
    # Large sizes can keep refined proportions.
    if size <= 24:
        bold = 1.25
    elif size <= 32:
        bold = 1.18
    elif size <= 48:
        bold = 1.10
    elif size <= 64:
        bold = 1.05
    else:
        bold = 1.0

    # Supersample scaling
    if size >= 64:
        scale = 4
    elif size >= 32:
        scale = 3
    else:
        scale = 2

    S = size * scale
    im = Image.new("RGBA", (S, S), (0, 0, 0, 255))
    d = ImageDraw.Draw(im)

    # Base figure occupies 38% of canvas radius (was 30%).
    # The triangle + nodes fills ~92% of canvas width and ~84% of height.
    # bold multiplier fattens everything for small sizes.
    fig_r = S * 0.38 * bold
    cx = cy = S / 2.0
    # Optical vertical center: equilateral with flat base sits a hair high; nudge down
    cy += S * 0.02

    top, br, bl = _equilateral_points(cx, cy, fig_r)

    # Node radius: 30% of figure radius (was 18%) — chunky dots.
    # Minimum hard floor in scaled coords so tiny sizes don't vanish.
    node_r = max(scale * 1.5, fig_r * 0.30)

    # Stroke width: 14% of figure radius (was 8.5%) — thick connecting lines.
    stroke = max(scale * 1.2, fig_r * 0.14)

    def disk(x: float, y: float, r: float) -> None:
        d.ellipse([x - r, y - r, x + r, y + r], fill=(255, 255, 255, 255))

    def line_seg(a, b, w: float) -> None:
        x0, y0 = a
        x1, y1 = b
        dx, dy = x1 - x0, y1 - y0
        length = math.hypot(dx, dy) or 1.0
        px, py = -dy / length, dx / length
        half = w / 2.0
        d.polygon(
            [
                (x0 + px * half, y0 + py * half),
                (x0 - px * half, y0 - py * half),
                (x1 - px * half, y1 - py * half),
                (x1 + px * half, y1 + py * half),
            ],
            fill=(255, 255, 255, 255),
        )
        # round caps
        disk(x0, y0, half)
        disk(x1, y1, half)

    def inset(a, b, gap: float):
        x0, y0 = a
        x1, y1 = b
        dx, dy = x1 - x0, y1 - y0
        length = math.hypot(dx, dy) or 1.0
        ux, uy = dx / length, dy / length
        return (x0 + ux * gap, y0 + uy * gap), (x1 - ux * gap, y1 - uy * gap)

    # Links meet node rims (not center)
    gap = node_r * 0.92
    for a, b in ((top, br), (top, bl), (bl, br)):
        a2, b2 = inset(a, b, gap)
        line_seg(a2, b2, stroke)

    for p in (top, br, bl):
        disk(p[0], p[1], node_r)

    # Downscale with LANCZOS → soft AA edges
    out = im.resize((size, size), Image.Resampling.LANCZOS)

    # Composite onto pure black (no partial alpha from edge pixels)
    bg = Image.new("RGBA", (size, size), (0, 0, 0, 255))
    bg.alpha_composite(out)
    out = bg

    return out


def write_ico(path: Path, sizes: list[int]) -> None:
    images = [draw_mark(s).convert("RGBA") for s in sizes]
    blobs: list[bytes] = []
    for im in images:
        buf = io.BytesIO()
        im.save(buf, format="PNG", optimize=True)
        blobs.append(buf.getvalue())

    count = len(images)
    header = struct.pack("<HHH", 0, 1, count)
    offset = 6 + 16 * count
    entries = bytearray()
    for im, blob in zip(images, blobs):
        w = 0 if im.width >= 256 else im.width
        h = 0 if im.height >= 256 else im.height
        entries += struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(blob), offset)
        offset += len(blob)
    path.write_bytes(header + bytes(entries) + b"".join(blobs))


def verify_ico(path: Path, expected: list[int]) -> None:
    data = path.read_bytes()
    _r, typ, count = struct.unpack_from("<HHH", data, 0)
    assert typ == 1
    sizes: list[int] = []
    for i in range(count):
        w, h, _c, _rs, _pl, _bc, nbytes, off = struct.unpack_from(
            "<BBBBHHII", data, 6 + i * 16
        )
        w = 256 if w == 0 else w
        sizes.append(w)
        assert data[off : off + 8] == b"\x89PNG\r\n\x1a\n", f"frame {i} not PNG"
        frame = Image.open(io.BytesIO(data[off : off + nbytes]))
        assert frame.size[0] == w, f"size mismatch {frame.size} vs {w}"
    got = sorted(sizes)
    exp = sorted(expected)
    print(f"ICO: {count} frames {got} ({path.stat().st_size} bytes)")
    if got != exp:
        raise SystemExit(f"ICO FAIL expected {exp} got {got}")
    print("ICO VERIFY OK")


def main() -> None:
    public = ROOT / "public"
    assets = ROOT / "src" / "assets"
    icons = ROOT / "src-tauri" / "icons"
    public.mkdir(exist_ok=True)
    assets.mkdir(parents=True, exist_ok=True)
    icons.mkdir(parents=True, exist_ok=True)

    # Previews at each critical size for visual audit
    for s in (16, 24, 32, 48, 64, 128, 256, 512):
        img = draw_mark(s)
        img.save(assets / f"preview-{s}.png")

    draw_mark(512).save(assets / "logo.png")
    draw_mark(512).save(public / "logo.png")
    draw_mark(32).save(public / "favicon-32.png")
    draw_mark(16).save(public / "favicon-16.png")

    for name, size in [
        ("32x32.png", 32),
        ("64x64.png", 64),
        ("128x128.png", 128),
        ("128x128@2x.png", 256),
        ("icon.png", 512),
        ("Square30x30Logo.png", 30),
        ("Square44x44Logo.png", 44),
        ("Square71x71Logo.png", 71),
        ("Square89x89Logo.png", 89),
        ("Square107x107Logo.png", 107),
        ("Square142x142Logo.png", 142),
        ("Square150x150Logo.png", 150),
        ("Square284x284Logo.png", 284),
        ("Square310x310Logo.png", 310),
        ("StoreLogo.png", 50),
    ]:
        draw_mark(size).save(icons / name)

    # ICO with sizes in priority order (Windows picks best match)
    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    write_ico(icons / "icon.ico", ico_sizes)
    verify_ico(icons / "icon.ico", ico_sizes)

    # Dump ICO frames for visual audit
    data = (icons / "icon.ico").read_bytes()
    count = struct.unpack_from("<H", data, 4)[0]
    for i in range(count):
        w, h, _c, _r, _pl, _bc, nbytes, off = struct.unpack_from(
            "<BBBBHHII", data, 6 + i * 16
        )
        w = 256 if w == 0 else w
        Image.open(io.BytesIO(data[off : off + nbytes])).save(
            assets / f"verify-ico-{w}.png"
        )

    print("Wrote previews to src/assets/preview-*.png and verify-ico-*.png")
    print("ALL OK")


if __name__ == "__main__":
    main()
