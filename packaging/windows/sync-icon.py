from __future__ import annotations

import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw


SIZES = (16, 24, 32, 48, 64, 128, 256)
MASTER_SIZE = 1024

BACKGROUND = "#8A552A"
RIM = "#6E411C"
PRIMARY = "#F6EFE5"
ACCENT = "#D8C1A2"


def scale(value: float) -> float:
    return value * MASTER_SIZE / 256.0


def point(x: float, y: float) -> tuple[float, float]:
    return scale(x), scale(y)


def draw_round_cap(draw: ImageDraw.ImageDraw, xy: tuple[float, float], width: float, fill: str) -> None:
    radius = width / 2.0
    x, y = xy
    draw.ellipse((x - radius, y - radius, x + radius, y + radius), fill=fill)


def draw_round_line(
    draw: ImageDraw.ImageDraw,
    points: list[tuple[float, float]],
    width: float,
    fill: str,
) -> None:
    draw.line(points, fill=fill, width=round(width), joint="curve")
    draw_round_cap(draw, points[0], width, fill)
    draw_round_cap(draw, points[-1], width, fill)


def draw_round_arc(
    draw: ImageDraw.ImageDraw,
    bbox: tuple[float, float, float, float],
    start: float,
    end: float,
    width: float,
    fill: str,
) -> None:
    draw.arc(bbox, start=start, end=end, fill=fill, width=round(width))
    center_x = (bbox[0] + bbox[2]) / 2.0
    center_y = (bbox[1] + bbox[3]) / 2.0
    radius_x = (bbox[2] - bbox[0]) / 2.0
    radius_y = (bbox[3] - bbox[1]) / 2.0
    for angle in (start, end):
        radians = math.radians(angle)
        cap = (
            center_x + radius_x * math.cos(radians),
            center_y + radius_y * math.sin(radians),
        )
        draw_round_cap(draw, cap, width, fill)


def render_master() -> Image.Image:
    image = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)

    draw.ellipse(
        (
            scale(8),
            scale(8),
            scale(248),
            scale(248),
        ),
        fill=BACKGROUND,
    )
    draw.ellipse(
        (
            scale(16),
            scale(16),
            scale(240),
            scale(240),
        ),
        outline=RIM,
        width=round(scale(4)),
    )

    primary_width = scale(28)
    accent_width = scale(12)

    draw_round_arc(
        draw,
        (
            scale(38),
            scale(38),
            scale(218),
            scale(218),
        ),
        45,
        315,
        primary_width,
        PRIMARY,
    )
    draw_round_line(
        draw,
        [
            point(192, 64),
            point(128, 64),
            point(128, 192),
        ],
        primary_width,
        PRIMARY,
    )

    loop_box = (
        scale(96),
        scale(64),
        scale(160),
        scale(128),
    )
    draw_round_arc(draw, loop_box, 90, 270, primary_width, PRIMARY)
    draw_round_line(
        draw,
        [
            point(128, 64),
            point(180, 192),
            point(220, 192),
        ],
        primary_width,
        PRIMARY,
    )
    draw_round_arc(draw, loop_box, 90, 270, accent_width, ACCENT)
    draw_round_line(
        draw,
        [
            point(128, 64),
            point(180, 192),
            point(220, 192),
        ],
        accent_width,
        ACCENT,
    )

    return image.resize((512, 512), Image.Resampling.LANCZOS)


def write_ico(svg_path: Path, ico_path: Path) -> None:
    if not svg_path.is_file():
        raise FileNotFoundError(f"input SVG not found: {svg_path}")

    image = render_master()
    ico_path.parent.mkdir(parents=True, exist_ok=True)
    image.save(ico_path, format="ICO", sizes=[(size, size) for size in SIZES])


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print("usage: sync-icon.py <input.svg> <output.ico>", file=sys.stderr)
        return 2

    svg_path = Path(argv[1])
    ico_path = Path(argv[2])
    write_ico(svg_path, ico_path)
    print(f"synchronized icon: {ico_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
