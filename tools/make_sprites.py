"""Draws the game's sprites into assets/sprites.

Run from the project folder:  python3 tools/make_sprites.py
Needs Pillow (pip install pillow).

Each sprite is drawn on a 512 x 512 canvas and shrunk to 128 x 128, which
smooths the edges. To use your own art instead, replace any of the PNG files
with one of the same name.
"""

import math
from pathlib import Path

from PIL import Image, ImageDraw

BIG = 512
SMALL = 128
OUT = Path(__file__).resolve().parent.parent / "assets" / "sprites"

INK = (70, 48, 42, 255)
FLOOR = (226, 214, 190, 255)
FLOOR_LIGHT = (237, 227, 205, 255)
WHITE = (250, 249, 244, 255)
GREY_LINE = (150, 150, 162, 255)

TOMATO = (228, 62, 50, 255)
TOMATO_DARK = (150, 32, 30, 255)
TOMATO_FLESH = (246, 124, 104, 255)
LEAF = (74, 160, 72, 255)
LEAF_DARK = (40, 104, 48, 255)
SOUP = (236, 98, 44, 255)
SOUP_DARK = (190, 70, 34, 255)
CREAM = (255, 228, 194, 255)
CRUST = (200, 140, 78, 255)
CRUST_DARK = (120, 76, 40, 255)
CRUMB = (243, 215, 165, 255)
CHEESE = (250, 206, 72, 255)
CHEESE_DARK = (196, 144, 30, 255)
TOAST = (176, 110, 54, 255)
TOAST_DARK = (104, 60, 30, 255)
WOOD = (206, 160, 104, 255)
WOOD_DARK = (128, 86, 50, 255)
STEEL = (74, 80, 94, 255)
STEEL_DARK = (40, 42, 52, 255)
SKIN = (255, 214, 170, 255)


def canvas(background=(0, 0, 0, 0)):
    image = Image.new("RGBA", (BIG, BIG), background)
    return image, ImageDraw.Draw(image)


def save(image, name):
    OUT.mkdir(parents=True, exist_ok=True)
    image.resize((SMALL, SMALL), Image.LANCZOS).save(OUT / f"{name}.png")


def circle(draw, x, y, r, fill, outline=None, width=0):
    draw.ellipse((x - r, y - r, x + r, y + r), fill=fill, outline=outline, width=width)


def floor_canvas():
    image, draw = canvas(FLOOR)
    draw.rounded_rectangle((8, 8, BIG - 8, BIG - 8), radius=28, fill=FLOOR_LIGHT)
    return image, draw


def floor():
    image, _ = floor_canvas()
    save(image, "floor")


def wall():
    image, draw = canvas((52, 56, 72, 255))
    draw.rounded_rectangle((10, 10, BIG - 10, BIG - 10), radius=26, fill=(82, 88, 110, 255))
    draw.rounded_rectangle((10, 10, BIG - 10, 150), radius=26, fill=(100, 107, 131, 255))
    draw.rectangle((10, 110, BIG - 10, 150), fill=(82, 88, 110, 255))
    save(image, "wall")


def chopping_board():
    image, draw = floor_canvas()
    # Handle with a hole, then the board itself.
    draw.rounded_rectangle((206, 30, 306, 150), radius=30, fill=WOOD, outline=WOOD_DARK, width=12)
    circle(draw, 256, 76, 16, FLOOR_LIGHT, WOOD_DARK, 8)
    draw.rounded_rectangle((52, 112, 460, 470), radius=44, fill=WOOD, outline=WOOD_DARK, width=12)
    # Knife marks.
    for x, y in ((110, 190), (150, 380), (330, 400), (360, 180), (100, 290)):
        draw.line((x, y, x + 54, y + 22), fill=(176, 130, 80, 255), width=8)
    save(image, "chopping_board")


def stove():
    image, draw = floor_canvas()
    draw.rounded_rectangle((30, 30, 482, 482), radius=50, fill=STEEL, outline=STEEL_DARK, width=12)
    circle(draw, 256, 256, 176, (50, 52, 62, 255))
    circle(draw, 256, 256, 146, None, (240, 112, 48, 255), 26)
    circle(draw, 256, 256, 92, None, (252, 176, 72, 255), 18)
    circle(draw, 256, 256, 36, (255, 222, 130, 255))
    save(image, "stove")


def hatch():
    image, draw = floor_canvas()
    circle(draw, 256, 256, 216, WHITE, GREY_LINE, 10)
    circle(draw, 256, 256, 160, None, (220, 220, 226, 255), 8)
    save(image, "hatch")


def conveyor():
    """Pointing up. The game turns the picture for the other directions."""
    image, draw = canvas((58, 60, 72, 255))
    draw.rectangle((0, 0, 52, BIG), fill=(124, 130, 146, 255))
    draw.rectangle((BIG - 52, 0, BIG, BIG), fill=(124, 130, 146, 255))
    draw.rectangle((64, 0, BIG - 64, BIG), fill=(82, 86, 100, 255))
    for y in (96, 226, 356):
        draw.line((150, y + 70, 256, y, 362, y + 70), fill=(250, 202, 72, 255), width=32, joint="curve")
    save(image, "conveyor")


def ice():
    image, draw = canvas((178, 220, 242, 255))
    draw.rounded_rectangle((8, 8, BIG - 8, BIG - 8), radius=28, fill=(208, 238, 252, 255))
    for x, y, length in ((90, 200, 130), (150, 250, 70), (300, 390, 120), (330, 110, 60)):
        draw.line((x, y, x + length, y - length), fill=(248, 253, 255, 255), width=16)
    save(image, "ice")


def bin_():
    image, draw = floor_canvas()
    circle(draw, 256, 256, 200, (126, 132, 148, 255), (60, 64, 78, 255), 12)
    circle(draw, 256, 256, 146, (34, 36, 46, 255))
    draw.arc((126, 126, 386, 386), start=200, end=260, fill=(70, 74, 90, 255), width=14)
    save(image, "bin")


def star(x, y, outer, inner, points=5):
    corners = []
    for i in range(points * 2):
        radius = outer if i % 2 == 0 else inner
        angle = math.pi * i / points - math.pi / 2
        corners.append((x + radius * math.cos(angle), y + radius * math.sin(angle)))
    return corners


def tomato():
    image, draw = canvas()
    circle(draw, 256, 284, 176, TOMATO, TOMATO_DARK, 14)
    draw.ellipse((150, 190, 240, 250), fill=(255, 150, 130, 255))
    draw.polygon(star(256, 130, 84, 30), fill=LEAF, outline=LEAF_DARK, width=10)
    save(image, "tomato")


def chopped_tomato():
    image, draw = canvas()
    for x, y in ((166, 196), (346, 206), (256, 356)):
        circle(draw, x, y, 104, TOMATO, TOMATO_DARK, 12)
        circle(draw, x, y, 72, TOMATO_FLESH)
        for i in range(6):
            angle = math.pi * i / 3
            circle(draw, x + 44 * math.cos(angle), y + 44 * math.sin(angle), 11, (252, 226, 140, 255))
    save(image, "chopped_tomato")


def tomato_soup():
    image, draw = canvas()
    circle(draw, 256, 256, 206, WHITE, GREY_LINE, 12)
    circle(draw, 256, 256, 158, SOUP, SOUP_DARK, 10)
    draw.arc((176, 176, 336, 336), start=200, end=40, fill=CREAM, width=18)
    draw.arc((216, 216, 316, 316), start=20, end=220, fill=CREAM, width=18)
    for x, y in ((330, 320), (196, 340), (316, 190)):
        circle(draw, x, y, 12, LEAF)
    save(image, "tomato_soup")


def bread_shape(draw, grow, fill):
    """A slice of bread, `grow` pixels bigger (or smaller) all round."""
    draw.rounded_rectangle((104 - grow, 200 - grow, 408 + grow, 448 + grow), radius=44 + grow, fill=fill)
    circle(draw, 186, 196, 92 + grow, fill)
    circle(draw, 326, 196, 92 + grow, fill)


def bread():
    image, draw = canvas()
    bread_shape(draw, 12, CRUST_DARK)
    bread_shape(draw, 0, CRUST)
    bread_shape(draw, -26, CRUMB)
    save(image, "bread")


def cheese():
    image, draw = canvas()
    # The side of the wedge, then its top.
    draw.polygon(((76, 372), (436, 372), (436, 440), (76, 440)), fill=(232, 180, 52, 255), outline=CHEESE_DARK, width=12)
    draw.polygon(((76, 372), (436, 372), (436, 150)), fill=CHEESE, outline=CHEESE_DARK, width=12)
    for x, y, r in ((360, 300, 30), (280, 336, 18), (390, 222, 18), (210, 344, 12)):
        circle(draw, x, y, r, (226, 172, 44, 255))
    save(image, "cheese")


def sandwich_shape(draw, bread_fill, bread_line):
    draw.rounded_rectangle((92, 306, 420, 404), radius=30, fill=bread_fill, outline=bread_line, width=12)
    draw.rounded_rectangle((72, 262, 440, 318), radius=18, fill=CHEESE, outline=CHEESE_DARK, width=10)
    draw.rounded_rectangle((92, 140, 420, 274), radius=64, fill=bread_fill, outline=bread_line, width=12)


def sandwich():
    image, draw = canvas()
    sandwich_shape(draw, CRUST, CRUST_DARK)
    save(image, "sandwich")


def toastie():
    image, draw = canvas()
    sandwich_shape(draw, TOAST, TOAST_DARK)
    # Grill lines on top, and cheese melting out of the side.
    for x in (150, 216, 282, 348):
        draw.line((x, 236, x + 40, 172), fill=TOAST_DARK, width=14)
    for x, drop in ((130, 44), (236, 66), (350, 36)):
        draw.rounded_rectangle((x, 300, x + 40, 318 + drop), radius=20, fill=CHEESE, outline=CHEESE_DARK, width=8)
    save(image, "toastie")


def chef():
    image, draw = canvas()
    # Face.
    draw.ellipse((126, 214, 386, 474), fill=SKIN, outline=INK, width=12)
    circle(draw, 206, 350, 16, INK)
    circle(draw, 306, 350, 16, INK)
    circle(draw, 174, 392, 22, (255, 170, 150, 255))
    circle(draw, 338, 392, 22, (255, 170, 150, 255))
    draw.arc((212, 360, 300, 430), start=20, end=160, fill=INK, width=12)
    # Hat: three puffs on a band.
    circle(draw, 170, 150, 82, WHITE, INK, 12)
    circle(draw, 342, 150, 82, WHITE, INK, 12)
    circle(draw, 256, 118, 96, WHITE, INK, 12)
    draw.rounded_rectangle((142, 176, 370, 262), radius=20, fill=WHITE, outline=INK, width=12)
    draw.rectangle((160, 150, 352, 190), fill=WHITE)
    save(image, "chef")


if __name__ == "__main__":
    for make in (floor, wall, chopping_board, stove, hatch, conveyor, ice, bin_,
                 tomato, chopped_tomato, tomato_soup, bread, cheese, sandwich, toastie, chef):
        make()
    print(f"sprites written to {OUT}")
