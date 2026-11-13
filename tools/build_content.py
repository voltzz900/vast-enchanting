#!/usr/bin/env python3
"""Builds Vast Enchanting's content mod (`vastenchanting`): the Vast Enchanting Table, the shelves around it, their
art (drawn here, pixel by pixel), recipes, loot and names; and writes the shelves' numbers and the infusions as Rust
(between the markers in `mixins/vastenchanting_native/client.rs`), so the native mod and the content read one table.

The way the table works (power, chaos, rarity and clues gathered from the blocks around it, offers up to level 100,
infusions) follows Apothic Enchanting's MIT code by Shadows_of_Fire; the names, art and recipes are this mod's own.

Usage: build_content.py <output dir>   (writes <output dir>/vastenchanting)
"""
import json
import math
import random
import shutil
import struct
import sys
import zlib
from pathlib import Path

NS = "vastenchanting"
VERSION = "0.1.0"
ROOT = Path(__file__).resolve().parent.parent

# ---- the shelves ------------------------------------------------------------------------------------------------------
# id, english, portuguese, family, stats (max power, power, chaos, rarity, clues)
SHELVES = [
    ("ember_shelf", "Ember Shelf", "Estante de Brasa", "ember", (45, 3, 3, 0, 0)),
    ("infused_ember_shelf", "Infused Ember Shelf", "Estante de Brasa Infusa", "ember_infused", (60, 5, 5, 0, 0)),
    ("blazing_ember_shelf", "Blazing Ember Shelf", "Estante de Brasa Ardente", "ember_blazing", (65, 10, 10, 0, -1)),
    ("tide_shelf", "Tide Shelf", "Estante da Maré", "tide", (45, 3, 0, 3, 0)),
    ("infused_tide_shelf", "Infused Tide Shelf", "Estante da Maré Infusa", "tide_infused", (60, 5, 0, 5, 0)),
    ("heart_tide_shelf", "Heart of the Tide Shelf", "Estante do Coração da Maré", "tide_heart", (60, 15, 0, 20, 0)),
    ("dormant_abyss_shelf", "Dormant Abyss Shelf", "Estante do Abismo Adormecida", "abyss_dormant", (30, 2, 0, 0, 0)),
    ("abyss_shelf", "Abyss Shelf", "Estante do Abismo", "abyss", (70, 5, 5, 5, 0)),
    ("echoing_abyss_shelf", "Echoing Abyss Shelf", "Estante do Abismo Ecoante", "abyss_echo", (75, 5, 0, 15, 0)),
    ("void_shelf", "Void Shelf", "Estante do Vazio", "void", (90, 5, 5, 5, 0)),
    ("draconic_void_shelf", "Draconic Void Shelf", "Estante do Vazio Dracônica", "void_draconic", (100, 20, 0, 0, 0)),
    ("sight_shelf", "Shelf of Sight", "Estante da Visão", "sight", (0, 0, 0, 0, 1)),
    ("sight_shelf_ii", "Shelf of Sight II", "Estante da Visão II", "sight_ii", (0, 0, 0, 0, 2)),
    # Vast Depths' metals: the top of the climb (their recipes need Vast Depths).
    ("stellarite_shelf", "Stellarite Shelf", "Estante de Estelarita", "stellarite", (100, 12, 5, 5, 0)),
    ("infernium_shelf", "Infernium Shelf", "Estante de Infernium", "infernium", (100, 8, 15, 0, 0)),
    ("voidium_shelf", "Voidium Shelf", "Estante de Voidium", "voidium", (100, 8, 0, 15, 0)),
]
# Vanilla blocks the table also reads: fifteen bookshelves reach power 30, as at vanilla's table.
VANILLA_STATS = [
    ("minecraft:bookshelf", (30, 2, 0, 0, 0)),
    ("minecraft:chiseled_bookshelf", (30, 2, 0, 0, 0)),
]
# Infusions at the table: the item in its slot becomes the result when the table's stats are in range.
# input, result, minimums (power, chaos, rarity), maximums (-1: none), how many
INFUSIONS = [
    ("vastenchanting:ember_shelf", "vastenchanting:infused_ember_shelf", (45, 30, 0), (-1, -1, -1), 1),
    ("vastenchanting:tide_shelf", "vastenchanting:infused_tide_shelf", (45, 15, 10), (-1, -1, -1), 1),
    ("vastenchanting:dormant_abyss_shelf", "vastenchanting:abyss_shelf", (60, 40, 40), (-1, -1, -1), 1),
    ("vastenchanting:sight_shelf", "vastenchanting:sight_shelf_ii", (60, 20, 30), (-1, -1, -1), 1),
    ("minecraft:carrot", "minecraft:golden_carrot", (20, 10, 0), (20, 30, -1), 1),
    ("minecraft:amethyst_block", "minecraft:budding_amethyst", (60, 30, 50), (-1, 50, -1), 1),
    ("vastenchanting:tattered_tome", "vastenchanting:bound_tome", (40, 20, 30), (-1, 50, -1), 2),
    ("vastenchanting:bound_tome", "vastenchanting:siphon_tome", (55, 25, 45), (-1, 70, -1), 2),
]

# ---- the tomes ------------------------------------------------------------------------------------------------------
# Used at an anvil with an enchanted item on the left: they take its enchantments into an enchanted book.
# id, english, portuguese, cover, trim
TOMES = [
    ("tattered_tome", "Tattered Tome", "Tomo Esfarrapado", 0x7A4A2A, 0xC8A27A),
    ("bound_tome", "Bound Tome", "Tomo Encadernado", 0x1E3A7A, 0xF0B33A),
    ("siphon_tome", "Siphon Tome", "Tomo de Sifão", 0x4A1E6A, 0x3AE0C0),
]
TOME_TIPS = {
    "tattered_tome": ("At an anvil, with an enchanted item: about half its enchantments go into a book. The item is lost.",
                      "Na bigorna, com um item encantado: cerca de metade dos encantamentos vai para um livro. O item se perde."),
    "bound_tome": ("At an anvil, with an enchanted item: all its enchantments go into a book. The item is lost.",
                   "Na bigorna, com um item encantado: todos os encantamentos vão para um livro. O item se perde."),
    "siphon_tome": ("At an anvil, with an enchanted item: all its enchantments go into a book, and the item comes back plain and worn.",
                    "Na bigorna, com um item encantado: todos os encantamentos vão para um livro, e o item volta sem encanto e gasto."),
}

VAST_DEPTHS = {"stellarite_shelf": "stellarite", "infernium_shelf": "infernium", "voidium_shelf": "voidium"}

# ---- the archive ----------------------------------------------------------------------------------------------------
# Stores enchanted books as points (a level-L book is worth 2^(L-1)) and writes them back onto books.
# id, english, portuguese, highest level it keeps
ARCHIVES = [
    ("arcane_archive", "Arcane Archive", "Arquivo Arcano", 16),
]

# ---- art ------------------------------------------------------------------------------------------------------------

CLEAR = (0, 0, 0, 0)


def rgb(h):
    return ((h >> 16) & 255, (h >> 8) & 255, h & 255)


def shade(c, k):
    return tuple(max(0, min(255, round(v * k))) for v in c)


def mix(a, b, t):
    return tuple(round(x + (y - x) * t) for x, y in zip(a, b))


def png(pixels):
    raw = b"".join(b"\x00" + bytes(c for p in row for c in p) for row in pixels)

    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", len(pixels[0]), len(pixels), 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


# Each family: frame (the case), its dark edge, the end texture's stone, the books' colours, a glow for infused ones.
FAMILIES = {
    "ember": dict(frame=0x4A1C1C, stone=0x5A2226, books=[0xB33A1E, 0xE0702A, 0x7A2A12, 0xD9A23A], glow=None),
    "ember_infused": dict(frame=0x4A1C1C, stone=0x5A2226, books=[0xB33A1E, 0xE0702A, 0x7A2A12, 0xD9A23A], glow=0xFF9A3C),
    "ember_blazing": dict(frame=0x6A2412, stone=0x7A3418, books=[0xFFB23A, 0xFF7A1E, 0xE0401E, 0xFFE07A], glow=0xFFD24A),
    "tide": dict(frame=0x2E6B66, stone=0x3E8A80, books=[0x3FA7D6, 0x5ED6C8, 0x1E5E8A, 0xA8E6E0], glow=None),
    "tide_infused": dict(frame=0x2E6B66, stone=0x3E8A80, books=[0x3FA7D6, 0x5ED6C8, 0x1E5E8A, 0xA8E6E0], glow=0x8AF0FF),
    "tide_heart": dict(frame=0x1E4A6A, stone=0x2E6A8A, books=[0x2A7FD6, 0x6AD6FF, 0x1A3A8A, 0xFFFFFF], glow=0x4AC8FF),
    "abyss_dormant": dict(frame=0x2E2E36, stone=0x3A3A44, books=[0x4A4A56, 0x5A5A66, 0x36363F, 0x6A6A76], glow=None),
    "abyss": dict(frame=0x2E2E36, stone=0x3A3A44, books=[0x1E6A6A, 0x2A9A8A, 0x0E3A44, 0x5AD6C0], glow=None),
    "abyss_echo": dict(frame=0x1A2A36, stone=0x22343F, books=[0x0E7A8A, 0x2AC8D6, 0x0A3A4A, 0xC8F6FF], glow=0x2AD6E0),
    "void": dict(frame=0x5A5A3A, stone=0xD6D69A, books=[0x8A3AB0, 0xB06AD6, 0x4A1E6A, 0xE0C8F0], glow=None),
    "void_draconic": dict(frame=0x2A1636, stone=0x3A2248, books=[0xB03AE0, 0xE06AFF, 0x5A1E8A, 0xFFC8FF], glow=0xE07AFF),
    "sight": dict(frame=0x6A5022, stone=0xC8A23A, books=[0xF2C63A, 0xFFFFFF, 0x9A7A1E, 0x7AD6F0], glow=None),
    "sight_ii": dict(frame=0x6A5022, stone=0xC8A23A, books=[0xF2C63A, 0xFFFFFF, 0x9A7A1E, 0x7AD6F0], glow=0xFFF2A0),
    "stellarite": dict(frame=0x6A4A12, stone=0xF0B33A, books=[0xF0B33A, 0xFFE08A, 0xB07A1E, 0xFFFFFF], glow=0xFFD27A),
    "infernium": dict(frame=0x5A1E0E, stone=0xE0562A, books=[0xE0562A, 0xFF9A5A, 0x8A2A0E, 0xFFD27A], glow=0xFF7A3A),
    "voidium": dict(frame=0x3A1250, stone=0xC23CE6, books=[0xC23CE6, 0xE69AFF, 0x6A1E8A, 0xFFFFFF], glow=0xE07AFF),
}


def shelf_side(family, seed):
    f = FAMILIES[family]
    frame, books, glow = rgb(f["frame"]), [rgb(b) for b in f["books"]], f["glow"]
    rnd = random.Random(seed)
    px = [[CLEAR] * 16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            edge = y in (0, 15) or x in (0, 15)
            mid = y in (7, 8)
            if edge or mid:
                c = shade(frame, 1.25 if y in (0, 8) else 0.8 if y in (15, 7) else 1.0)
                if x in (0, 15) and not mid:
                    c = shade(frame, 0.9)
                px[y][x] = (*c, 255)
    for top, bottom in ((1, 6), (9, 14)):
        x = 1
        while x < 15:
            w = min(rnd.choice([1, 2, 2, 3]), 15 - x)
            c = rnd.choice(books)
            h = rnd.choice([0, 0, 1, 1, 2])
            for bx in range(x, x + w):
                for by in range(top, bottom + 1):
                    if by < top + h:
                        px[by][bx] = (*shade(frame, 0.45), 255)
                        continue
                    k = 1.15 if bx == x else 0.75 if bx == x + w - 1 and w > 1 else 1.0
                    cc = shade(c, k)
                    if by == top + h + 1 and w > 1:
                        cc = mix(cc, (255, 255, 255), 0.3)
                    px[by][bx] = (*cc, 255)
            x += w
    if glow:
        g = rgb(glow)
        for _ in range(5):
            gx, gy = rnd.randrange(1, 15), rnd.choice([rnd.randrange(2, 7), rnd.randrange(10, 15)])
            px[gy][gx] = (*mix(g, (255, 255, 255), 0.4), 255)
    return px


def shelf_end(family, seed):
    f = FAMILIES[family]
    stone, frame = rgb(f["stone"]), rgb(f["frame"])
    rnd = random.Random(seed + 99)
    px = [[CLEAR] * 16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            # Bricks of the family's stone, offset every other row.
            row = y // 4
            off = 4 if row % 2 else 0
            mortar = y % 4 == 3 or (x + off) % 8 == 7
            c = shade(stone, 0.6) if mortar else shade(stone, rnd.uniform(0.88, 1.08))
            if x in (0, 15) or y in (0, 15):
                c = shade(frame, 1.0)
            px[y][x] = (*c, 255)
    return px


def table_textures():
    """The table's top (a dark slab with a rim of the Vast metals and a star), side and bottom."""
    dark, rune = rgb(0x1A1026), rgb(0x8A3AB0)
    gold, fire, void = rgb(0xF0B33A), rgb(0xE0562A), rgb(0xC23CE6)
    top = [[(*dark, 255)] * 16 for _ in range(16)]
    for i in range(16):
        for (x, y) in ((i, 0), (i, 15), (0, i), (15, i)):
            top[y][x] = (*shade(gold, 0.8), 255)
        for (x, y) in ((i, 1), (1, i)):
            if 1 <= i <= 14:
                top[y][x] = (*fire, 255)
        for (x, y) in ((i, 14), (14, i)):
            if 1 <= i <= 14:
                top[y][x] = (*void, 255)
    c = 7.5
    for y in range(3, 13):
        for x in range(3, 13):
            dx, dy = x - c, y - c
            r = math.hypot(dx, dy)
            a = math.atan2(dy, dx)
            spike = 1.6 + 3.2 * abs(math.cos(2 * a)) ** 6
            if r <= spike:
                top[y][x] = (*mix((255, 255, 255), gold, min(1.0, r / 3.5)), 255)
            elif (x + y) % 3 == 0 and r < 5.5:
                top[y][x] = (*shade(rune, 0.7), 255)
    side = [[CLEAR] * 16 for _ in range(16)]
    rnd = random.Random(5)
    for y in range(16):
        for x in range(16):
            c = shade(dark, rnd.uniform(0.85, 1.25))
            if y in (4, 5):
                c = gold if y == 4 else shade(gold, 0.7)
            if 7 <= y <= 13 and (x * 3 + y) % 7 == 0:
                c = shade(rune, rnd.uniform(0.6, 1.0))
            side[y][x] = (*c, 255)
    bottom = [[(*shade(dark, rnd.uniform(0.7, 1.0)), 255) for _ in range(16)] for _ in range(16)]
    book = [[CLEAR] * 16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            if y < 8:
                c = (0xF2, 0xEC, 0xD6) if x not in (0, 15) else (0x8A, 0x3A, 0xB0)
                if y in (2, 4, 6) and 2 <= x <= 13 and x != 7 and x != 8:
                    c = (0x9A, 0x8A, 0x7A)
            else:
                c = (0x6A, 0x1E, 0x8A) if (x + y) % 5 else (0xF0, 0xB3, 0x3A)
            book[y][x] = (*c, 255)
    return top, side, bottom, book


def tome_texture(name, cover, trim):
    """A closed book seen from the front: spine on the left, page edges on the right and below."""
    cover, trim = rgb(cover), rgb(trim)
    page, page_dark = (0xEE, 0xE6, 0xCC), (0xB8, 0xAC, 0x8E)
    img = [[CLEAR] * 16 for _ in range(16)]

    def put(x, y, c):
        if 0 <= x < 16 and 0 <= y < 16:
            img[y][x] = (*c, 255)

    for y in range(2, 14):  # pages behind the cover, showing right and below
        for x in range(5, 14):
            put(x, y + 1, page if (y + x) % 2 else page_dark)
    rnd = random.Random(len(name))
    for y in range(2, 14):
        for x in range(3, 13):
            c = shade(cover, rnd.uniform(0.9, 1.1))
            if x in (3, 4):  # the spine
                c = shade(cover, 0.6 if x == 3 else 0.75)
            if y in (2, 13) or x == 12:
                c = shade(cover, 0.7)
            put(x, y, c)
    if name == "tattered_tome":
        for x, y in ((12, 4), (12, 5), (11, 5), (12, 9), (12, 10), (11, 10), (12, 11), (8, 13), (9, 13)):
            img[y][x] = CLEAR  # torn bites
        for x, y in ((6, 4), (7, 5), (8, 6), (9, 7), (7, 9), (6, 10)):
            put(x, y, shade(trim, 0.9))  # scuffs and a stitch
        for y in range(4, 12, 2):
            put(4, y, trim)
    elif name == "bound_tome":
        for y in range(2, 14):
            put(8, y, trim)  # a gold strap
        for x, y in ((5, 3), (6, 3), (5, 4), (11, 3), (10, 3), (11, 4), (5, 12), (6, 12), (5, 11), (11, 12), (10, 12), (11, 11)):
            put(x, y, trim)  # corners
        put(8, 7, (255, 255, 255)); put(8, 8, shade(trim, 0.7))
        put(13, 7, trim); put(13, 8, shade(trim, 0.7))  # the clasp
    else:  # siphon: a spiral drawing inward
        cx, cy = 8.0, 7.5
        for i in range(40):
            a = i * 0.5
            r = 3.9 - i * 0.12
            if r < 0.6:
                break
            put(round(cx + r * math.cos(a)), round(cy + r * math.sin(a)), mix(shade(trim, 0.7), (255, 255, 255), i / 32))
        put(8, 8, (255, 255, 255))
        for y in range(3, 13, 3):
            put(4, y, trim)
    return img


def gui_texture():
    """256x256: the screen (176x194) at the top left; below it the offer buttons (normal, hover, off) and the bars'
    groove and fills."""
    W = 256
    px = [[CLEAR] * W for _ in range(W)]
    bg, light, darkc, black = (0xC6, 0xC6, 0xC6), (0xFF, 0xFF, 0xFF), (0x55, 0x55, 0x55), (0x00, 0x00, 0x00)

    def rect(x0, y0, w, h, c):
        for y in range(y0, y0 + h):
            for x in range(x0, x0 + w):
                px[y][x] = (*c, 255)

    def bevel(x0, y0, w, h, fill, hi, lo):
        rect(x0, y0, w, h, fill)
        rect(x0, y0, w, 1, hi)
        rect(x0, y0, 1, h, hi)
        rect(x0, y0 + h - 1, w, 1, lo)
        rect(x0 + w - 1, y0, 1, h, lo)

    def slot(x0, y0):
        rect(x0, y0, 18, 18, (0x8B, 0x8B, 0x8B))
        rect(x0, y0, 17, 1, (0x37, 0x37, 0x37))
        rect(x0, y0, 1, 17, (0x37, 0x37, 0x37))
        rect(x0 + 1, y0 + 17, 17, 1, light)
        rect(x0 + 17, y0 + 1, 1, 17, light)

    # The panel: a window's outline and bevel.
    w, h = 176, 194
    rect(1, 1, w - 2, h - 2, bg)
    rect(2, 0, w - 4, 1, black)
    rect(2, h - 1, w - 4, 1, black)
    rect(0, 2, 1, h - 4, black)
    rect(w - 1, 2, 1, h - 4, black)
    for (x, y) in ((1, 1), (w - 2, 1), (1, h - 2), (w - 2, h - 2)):
        px[y][x] = (*black, 255)
    rect(2, 1, w - 4, 2, light)
    rect(1, 2, 2, h - 5, light)
    rect(3, h - 3, w - 5, 2, darkc)
    rect(w - 3, 3, 2, h - 5, darkc)
    # Item and lapis slots, the offers' well, the stats' well.
    slot(14, 46)
    slot(34, 46)
    bevel(59, 13, 110, 59, (0x3A, 0x2A, 0x4A), (0x22, 0x18, 0x2C), (0x6A, 0x5A, 0x7A))
    bevel(7, 74, 162, 34, (0xB0, 0xA8, 0xBC), (0x8A, 0x82, 0x96), (0xE6, 0xE0, 0xF0))
    # The player's inventory (3 rows and the hotbar), at y 112.
    for r in range(3):
        for c in range(9):
            slot(7 + c * 18, 111 + r * 18)
    for c in range(9):
        slot(7 + c * 18, 169)
    # Offer buttons: normal (y 195), hover (y 214), off (y 233), 108x19 each.
    for i, (fill, hi, lo) in enumerate((((0x6A, 0x5A, 0x8A), (0x9A, 0x8A, 0xBA), (0x3A, 0x2A, 0x5A)),
                                        ((0x8A, 0x6A, 0xC0), (0xC0, 0xA8, 0xF0), (0x4A, 0x2A, 0x7A)),
                                        ((0x4A, 0x44, 0x52), (0x5A, 0x54, 0x62), (0x34, 0x30, 0x3A)))):
        bevel(0, 195 + i * 19, 108, 19, fill, hi, lo)
        # The level badge on the left.
        bevel(2, 197 + i * 19, 15, 15, shade(fill, 0.7), lo, hi)
    # Bars: groove (y 195), then power, chaos, rarity fills (y 201, 207, 213), 110x6 each, at x 110.
    rect(110, 195, 110, 6, (0x2A, 0x22, 0x34))
    for i, (a, b) in enumerate(((0xF0B33A, 0xFFE59A), (0xE0562A, 0xFFA07A), (0xC23CE6, 0xF0A8FF))):
        for x in range(110):
            c = mix(rgb(a), rgb(b), x / 109)
            for y in range(6):
                px[201 + i * 6 + y][110 + x] = (*shade(c, 1.15 if y == 0 else 0.8 if y == 5 else 1.0), 255)
    return px


def archive_textures():
    """The archive's side (two shelves of books behind a gold-trimmed frame, a band of runes between), top (a lapis
    sigil in a ring) and bottom."""
    frame, frame_dark = rgb(0x2E1C40), rgb(0x1A1026)
    gold, lapis, glow = rgb(0xF0B33A), rgb(0x2A5ADC), rgb(0x5AF0E6)
    books = [rgb(c) for c in (0x3A4ABA, 0x6A2A9A, 0x1E7A8A, 0x8A3A6A, 0x2A6ACA, 0x4A1E7A)]
    rnd = random.Random(11)
    side = [[CLEAR] * 16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            c = shade(frame, rnd.uniform(0.85, 1.1))
            if x in (0, 15) or y in (0, 15):
                c = frame_dark
            if (x, y) in ((0, 0), (15, 0), (0, 15), (15, 15), (1, 1), (14, 1), (1, 14), (14, 14)):
                c = gold
            side[y][x] = (*c, 255)
    for top_row in (2, 10):  # two shelves of books, four pixels tall
        x = 1
        while x < 15:
            w = rnd.choice((1, 2, 2))
            book = rnd.choice(books)
            h = rnd.choice((3, 4, 4))
            for bx in range(x, min(15, x + w)):
                for by in range(top_row + 4 - h, top_row + 4):
                    c = shade(book, 1.15 if bx == x else 0.9)
                    if by == top_row + 4 - h + 1 and w == 2:
                        c = shade(gold, 0.85)  # a gilt band on the wider spines
                    side[by][bx] = (*c, 255)
            x += w
        for bx in range(1, 15):
            side[top_row + 4][bx] = (*shade(frame_dark, 1.2), 255)  # the shelf board
    for x in range(1, 15):  # the rune band
        side[7][x] = (*shade(glow, 0.55 + 0.45 * ((x * 5) % 3 == 0)), 255)
        side[8][x] = (*shade(frame_dark, 1.1), 255)
    top = [[(*shade(frame, rnd.uniform(0.85, 1.05)), 255) for _ in range(16)] for _ in range(16)]
    c = 7.5
    for y in range(16):
        for x in range(16):
            r = math.hypot(x - c, y - c)
            if x in (0, 15) or y in (0, 15):
                top[y][x] = (*gold, 255)
            elif 5.6 <= r <= 6.6:
                top[y][x] = (*shade(gold, 0.85), 255)
            elif r <= 4.6:
                a = math.atan2(y - c, x - c)
                star = r <= 1.2 + 3.0 * abs(math.cos(2.5 * a)) ** 4
                top[y][x] = (*(mix(glow, (255, 255, 255), 0.3) if star else shade(lapis, 0.7 + 0.1 * (r % 2))), 255)
    bottom = [[(*shade(frame_dark, rnd.uniform(0.8, 1.15)), 255) for _ in range(16)] for _ in range(16)]
    return side, top, bottom


def archive_gui_texture():
    """256x256: the archive's screen (176x194) at the top left; below it the list's rows (normal, hover, faint),
    128x15 each, and the scrollbar's thumb (8x15) at x 240."""
    W = 256
    px = [[CLEAR] * W for _ in range(W)]
    bg, light, darkc, black = (0xC6, 0xC6, 0xC6), (0xFF, 0xFF, 0xFF), (0x55, 0x55, 0x55), (0x00, 0x00, 0x00)

    def rect(x0, y0, w, h, c):
        for y in range(y0, y0 + h):
            for x in range(x0, x0 + w):
                px[y][x] = (*c, 255)

    def bevel(x0, y0, w, h, fill, hi, lo):
        rect(x0, y0, w, h, fill)
        rect(x0, y0, w, 1, hi)
        rect(x0, y0, 1, h, hi)
        rect(x0, y0 + h - 1, w, 1, lo)
        rect(x0 + w - 1, y0, 1, h, lo)

    def slot(x0, y0):
        rect(x0, y0, 18, 18, (0x8B, 0x8B, 0x8B))
        rect(x0, y0, 17, 1, (0x37, 0x37, 0x37))
        rect(x0, y0, 1, 17, (0x37, 0x37, 0x37))
        rect(x0 + 1, y0 + 17, 17, 1, light)
        rect(x0 + 17, y0 + 1, 1, 17, light)

    w, h = 176, 194
    rect(1, 1, w - 2, h - 2, bg)
    rect(2, 0, w - 4, 1, black)
    rect(2, h - 1, w - 4, 1, black)
    rect(0, 2, 1, h - 4, black)
    rect(w - 1, 2, 1, h - 4, black)
    for (x, y) in ((1, 1), (w - 2, 1), (1, h - 2), (w - 2, h - 2)):
        px[y][x] = (*black, 255)
    rect(2, 1, w - 4, 2, light)
    rect(1, 2, 2, h - 5, light)
    rect(3, h - 3, w - 5, 2, darkc)
    rect(w - 3, 3, 2, h - 5, darkc)
    # The book going in (top) and the book coming out (bottom), an arrow from each toward the list and back.
    slot(7, 17)
    slot(7, 79)
    arrow = (0x6A, 0x5A, 0x8A)
    for i in range(4):  # in: pointing right, toward the list
        rect(25 + i, 23 + i, 1, 7 - 2 * i, arrow)
    for i in range(4):  # out: pointing left, from the list
        rect(28 - i, 85 + i, 1, 7 - 2 * i, arrow)
    # A stack of tiny books between the slots: what lies inside.
    for k, col in enumerate(((0x3A, 0x4A, 0xBA), (0x6A, 0x2A, 0x9A), (0x1E, 0x7A, 0x8A))):
        bevel(9, 44 + k * 8, 14, 7, col, shade(col, 1.4), shade(col, 0.6))
        rect(11, 46 + k * 8, 10, 1, (0xF0, 0xB3, 0x3A))
    # The list's well and the scrollbar's track.
    bevel(31, 13, 130, 94, (0x3A, 0x2A, 0x4A), (0x22, 0x18, 0x2C), (0x6A, 0x5A, 0x7A))
    bevel(162, 13, 10, 94, (0x2A, 0x22, 0x34), (0x1A, 0x14, 0x22), (0x5A, 0x4E, 0x66))
    for r in range(3):
        for c in range(9):
            slot(7 + c * 18, 111 + r * 18)
    for c in range(9):
        slot(7 + c * 18, 169)
    for i, (fill, hi, lo) in enumerate((((0x5A, 0x4A, 0x7A), (0x8A, 0x7A, 0xAA), (0x3A, 0x2A, 0x5A)),
                                        ((0x8A, 0x6A, 0xC0), (0xC0, 0xA8, 0xF0), (0x4A, 0x2A, 0x7A)),
                                        ((0x44, 0x3C, 0x50), (0x54, 0x4C, 0x60), (0x34, 0x30, 0x3A)))):
        bevel(0, 195 + i * 15, 128, 15, fill, hi, lo)
    bevel(240, 195, 8, 15, (0xC0, 0xA8, 0xF0), (0xF0, 0xE6, 0xFF), (0x6A, 0x4A, 0x9A))
    return px


# ---- the mod ----------------------------------------------------------------------------------------------------------


def build(out):
    files, tags, items = {}, {}, []
    lang_en = {"container.vastenchanting.table": "Vast Enchanting Table",
               "vastenchanting.power": "Power", "vastenchanting.chaos": "Chaos", "vastenchanting.rarity": "Rarity",
               "vastenchanting.clues": "Clues: %s",
               "vastenchanting.power.desc": "How strong the offers get: the top offer costs this many levels.",
               "vastenchanting.chaos.desc": "How much each offer's strength varies, up or down.",
               "vastenchanting.rarity.desc": "Favours rare enchantments; at 25 and 75 an offer gains another.",
               "vastenchanting.clues.desc": "How many of each offer's enchantments are shown before you choose.",
               "vastenchanting.more": "...and %s more",
               "vastenchanting.unknown": "Unknown enchantments",
               "vastenchanting.infuse": "Infuse into %s",
               "vastenchanting.levels": "%s levels, %s lapis",
               "vastenchanting.need_levels": "Needs %s levels",
               "vastenchanting.shelf.stats": "Power +%s (up to %s) · Chaos %s · Rarity %s",
               "vastenchanting.shelf.clues": "Clues %s"}
    lang_pt = {"container.vastenchanting.table": "Mesa de Encantamento Vasta",
               "vastenchanting.power": "Poder", "vastenchanting.chaos": "Caos", "vastenchanting.rarity": "Raridade",
               "vastenchanting.clues": "Pistas: %s",
               "vastenchanting.power.desc": "A força das ofertas: a oferta de cima custa esse tanto de níveis.",
               "vastenchanting.chaos.desc": "O quanto a força de cada oferta varia, pra cima ou pra baixo.",
               "vastenchanting.rarity.desc": "Favorece encantamentos raros; em 25 e 75 cada oferta ganha mais um.",
               "vastenchanting.clues.desc": "Quantos encantamentos de cada oferta aparecem antes de escolher.",
               "vastenchanting.more": "...e mais %s",
               "vastenchanting.unknown": "Encantamentos desconhecidos",
               "vastenchanting.infuse": "Infundir em %s",
               "vastenchanting.levels": "%s níveis, %s lápis",
               "vastenchanting.need_levels": "Precisa de %s níveis",
               "vastenchanting.shelf.stats": "Poder +%s (até %s) · Caos %s · Raridade %s",
               "vastenchanting.shelf.clues": "Pistas %s"}

    def loot_self(name):
        files[f"data/{NS}/loot_table/blocks/{name}.json"] = {
            "type": "minecraft:block", "random_sequence": f"{NS}:blocks/{name}",
            "pools": [{"bonus_rolls": 0.0, "rolls": 1.0, "entries": [{"type": "minecraft:item", "name": f"{NS}:{name}"}],
                       "conditions": [{"condition": "minecraft:survives_explosion"}]}]}

    # The table.
    table = "vast_enchanting_table"
    files[f"data/{NS}/horizon/block/{table}.json"] = {
        "template": "minecraft:beacon", "item": True,
        "properties": {"hardness": 5.0, "explosion_resistance": 1200.0, "light": 7, "requires_tool": True,
                       "sounds": "minecraft:obsidian"}}
    files[f"data/{NS}/horizon/menu/table.json"] = {"slots": 2, "data_slots": 49}
    top, side, bottom, book = table_textures()
    for n, p in (("table_top", top), ("table_side", side), ("table_bottom", bottom), ("table_book", book)):
        files[f"assets/{NS}/textures/block/{n}.png"] = png(p)
    tex = {"top": f"{NS}:block/table_top", "side": f"{NS}:block/table_side", "bottom": f"{NS}:block/table_bottom",
           "book": f"{NS}:block/table_book", "particle": f"{NS}:block/table_side"}
    files[f"assets/{NS}/models/block/{table}.json"] = {
        "parent": "minecraft:block/block", "textures": tex,
        "elements": [
            {"from": [0, 0, 0], "to": [16, 12, 16], "faces": {
                "down": {"uv": [0, 0, 16, 16], "texture": "#bottom", "cullface": "down"},
                "up": {"uv": [0, 0, 16, 16], "texture": "#top"},
                "north": {"uv": [0, 4, 16, 16], "texture": "#side", "cullface": "north"},
                "south": {"uv": [0, 4, 16, 16], "texture": "#side", "cullface": "south"},
                "west": {"uv": [0, 4, 16, 16], "texture": "#side", "cullface": "west"},
                "east": {"uv": [0, 4, 16, 16], "texture": "#side", "cullface": "east"}}},
            # An open book resting above the top, its two halves tipped up.
            {"from": [3, 14, 5], "to": [8, 14.5, 11], "rotation": {"angle": 22.5, "axis": "z", "origin": [8, 14, 8]},
             "faces": {"up": {"uv": [0, 0, 8, 8], "texture": "#book"}, "down": {"uv": [0, 8, 8, 16], "texture": "#book"},
                       "north": {"uv": [0, 8, 8, 9], "texture": "#book"}, "south": {"uv": [0, 8, 8, 9], "texture": "#book"},
                       "west": {"uv": [0, 8, 1, 14], "texture": "#book"}, "east": {"uv": [0, 8, 1, 14], "texture": "#book"}}},
            {"from": [8, 14, 5], "to": [13, 14.5, 11], "rotation": {"angle": -22.5, "axis": "z", "origin": [8, 14, 8]},
             "faces": {"up": {"uv": [8, 0, 16, 8], "texture": "#book"}, "down": {"uv": [8, 8, 16, 16], "texture": "#book"},
                       "north": {"uv": [8, 8, 16, 9], "texture": "#book"}, "south": {"uv": [8, 8, 16, 9], "texture": "#book"},
                       "west": {"uv": [15, 8, 16, 14], "texture": "#book"}, "east": {"uv": [15, 8, 16, 14], "texture": "#book"}}},
        ]}
    files[f"assets/{NS}/blockstates/{table}.json"] = {"variants": {"": {"model": f"{NS}:block/{table}"}}}
    files[f"assets/{NS}/items/{table}.json"] = {"model": {"type": "minecraft:model", "model": f"{NS}:block/{table}"}}
    loot_self(table)
    files[f"data/{NS}/recipe/{table}.json"] = {
        "type": "minecraft:crafting_shaped", "category": "misc",
        "key": {"B": "minecraft:book", "D": "minecraft:diamond", "E": "minecraft:enchanting_table", "O": "minecraft:obsidian"},
        "pattern": [" B ", "DED", "OOO"], "result": {"id": f"{NS}:{table}"}}
    lang_en[f"block.{NS}.{table}"], lang_pt[f"block.{NS}.{table}"] = "Vast Enchanting Table", "Mesa de Encantamento Vasta"
    for t in ("minecraft:mineable/pickaxe", "minecraft:needs_diamond_tool"):
        tags.setdefault(("block", t), []).append(f"{NS}:{table}")
    items.append(f"{NS}:{table}")

    # The shelves.
    for i, (name, en, pt, family, _stats) in enumerate(SHELVES):
        files[f"data/{NS}/horizon/block/{name}.json"] = {
            "template": "minecraft:bookshelf", "item": True,
            "properties": {"hardness": 2.0 if "abyss" not in family else 4.0, "explosion_resistance": 6.0,
                           "sounds": "minecraft:bookshelf",
                           **({"light": 7} if FAMILIES[family]["glow"] else {})}}
        files[f"assets/{NS}/textures/block/{name}.png"] = png(shelf_side(family, i * 31 + 7))
        files[f"assets/{NS}/textures/block/{name}_end.png"] = png(shelf_end(family, i))
        files[f"assets/{NS}/models/block/{name}.json"] = {
            "parent": "minecraft:block/cube_column",
            "textures": {"end": f"{NS}:block/{name}_end", "side": f"{NS}:block/{name}"}}
        files[f"assets/{NS}/blockstates/{name}.json"] = {"variants": {"": {"model": f"{NS}:block/{name}"}}}
        files[f"assets/{NS}/items/{name}.json"] = {"model": {"type": "minecraft:model", "model": f"{NS}:block/{name}"}}
        loot_self(name)
        lang_en[f"block.{NS}.{name}"], lang_pt[f"block.{NS}.{name}"] = en, pt
        tags.setdefault(("block", "minecraft:mineable/axe" if "abyss" not in family else "minecraft:mineable/pickaxe"),
                        []).append(f"{NS}:{name}")
        tags.setdefault(("item", f"{NS}:shelves"), []).append(f"{NS}:{name}")
        items.append(f"{NS}:{name}")

    def shaped(name, pattern, key, conditions=None):
        r = {"type": "minecraft:crafting_shaped", "category": "misc", "group": "vastenchanting_shelves",
             "key": key, "pattern": pattern, "result": {"id": f"{NS}:{name}"}}
        if conditions:
            r["neoforge:conditions"] = conditions
        files[f"data/{NS}/recipe/{name}.json"] = r

    shaped("ember_shelf", ["NNN", "BOM", "NNN"], {"N": "minecraft:nether_bricks", "B": "minecraft:blaze_rod",
                                                    "O": "minecraft:bookshelf", "M": "minecraft:magma_cream"})
    shaped("tide_shelf", ["PPP", "COS", "PPP"], {"P": "minecraft:prismarine_bricks", "C": "minecraft:prismarine_crystals",
                                                   "O": "minecraft:bookshelf", "S": "minecraft:nautilus_shell"})
    shaped("dormant_abyss_shelf", ["TTT", "BSB", "TTT"], {"T": "minecraft:deepslate_tiles", "B": "minecraft:book",
                                                            "S": "minecraft:sculk"})
    shaped("void_shelf", ["EEE", "COP", "EEE"], {"E": "minecraft:end_stone_bricks", "C": "minecraft:popped_chorus_fruit",
                                                   "O": "minecraft:bookshelf", "P": "minecraft:ender_pearl"})
    shaped("blazing_ember_shelf", ["BFB", "FIF", "BFB"], {"B": "minecraft:blaze_powder", "F": "minecraft:fire_charge",
                                                            "I": f"{NS}:infused_ember_shelf"})
    shaped("heart_tide_shelf", ["SHS", "SIS", "SSS"], {"H": "minecraft:heart_of_the_sea", "S": "minecraft:prismarine_shard",
                                                         "I": f"{NS}:infused_tide_shelf"})
    shaped("echoing_abyss_shelf", ["CEC", "CAC", "CCC"], {"E": "minecraft:echo_shard", "A": f"{NS}:abyss_shelf",
                                                            "C": "minecraft:sculk_vein"})
    shaped("draconic_void_shelf", ["PDP", "PVP", "PPP"], {"D": "minecraft:dragon_head", "V": f"{NS}:void_shelf",
                                                            "P": "minecraft:ender_pearl"})
    shaped("sight_shelf", ["GAG", "SOY", "GAG"], {"G": "minecraft:gold_block", "A": "minecraft:amethyst_shard",
                                                    "S": "minecraft:spyglass", "O": "minecraft:bookshelf",
                                                    "Y": "minecraft:ender_eye"})
    for name, metal in VAST_DEPTHS.items():
        shaped(name, ["IBI", "BVB", "IBI"], {"I": f"vastdepths:{metal}_ingot", "B": "minecraft:bookshelf",
                                             "V": f"{NS}:void_shelf"},
               [{"type": "neoforge:mod_loaded", "modid": "vastdepths"}])

    # The tomes.
    for name, en, pt, cover, trim in TOMES:
        files[f"assets/{NS}/textures/item/{name}.png"] = png(tome_texture(name, cover, trim))
        files[f"assets/{NS}/models/item/{name}.json"] = {"parent": "minecraft:item/generated",
                                                          "textures": {"layer0": f"{NS}:item/{name}"}}
        files[f"assets/{NS}/items/{name}.json"] = {"model": {"type": "minecraft:model", "model": f"{NS}:item/{name}"}}
        files[f"data/{NS}/horizon/item/{name}.json"] = {"template": "minecraft:paper"}
        lang_en[f"item.{NS}.{name}"], lang_pt[f"item.{NS}.{name}"] = en, pt
        lang_en[f"item.{NS}.{name}.desc"], lang_pt[f"item.{NS}.{name}.desc"] = TOME_TIPS[name]
        tags.setdefault(("item", f"{NS}:tomes"), []).append(f"{NS}:{name}")
        items.append(f"{NS}:{name}")
    files[f"data/{NS}/recipe/tattered_tome.json"] = {
        "type": "minecraft:crafting_shaped", "category": "misc",
        "key": {"S": "minecraft:string", "B": "minecraft:book", "I": "minecraft:iron_ingot"},
        "pattern": [" S ", "BIB", " S "], "result": {"id": f"{NS}:tattered_tome", "count": 2}}

    # The archive.
    side, top, bottom = archive_textures()
    for name, en, pt, cap in ARCHIVES:
        files[f"data/{NS}/horizon/block/{name}.json"] = {
            "template": "minecraft:beacon", "item": True,
            # No tool needed: a hand breaking it must still drop it, with what it holds.
            "properties": {"hardness": 3.0, "explosion_resistance": 1200.0, "light": 5, "requires_tool": False,
                           "sounds": "minecraft:chiseled_bookshelf"}}
        for n, p in (("side", side), ("top", top), ("bottom", bottom)):
            files[f"assets/{NS}/textures/block/{name}_{n}.png"] = png(p)
        files[f"assets/{NS}/models/block/{name}.json"] = {
            "parent": "minecraft:block/cube_bottom_top",
            "textures": {"side": f"{NS}:block/{name}_side", "top": f"{NS}:block/{name}_top",
                         "bottom": f"{NS}:block/{name}_bottom"}}
        files[f"assets/{NS}/blockstates/{name}.json"] = {"variants": {"": {"model": f"{NS}:block/{name}"}}}
        files[f"assets/{NS}/items/{name}.json"] = {"model": {"type": "minecraft:model", "model": f"{NS}:block/{name}"}}
        loot_self(name)
        lang_en[f"block.{NS}.{name}"], lang_pt[f"block.{NS}.{name}"] = en, pt
        lang_en[f"block.{NS}.{name}.desc"] = (f"Put enchanted books in: they're kept as points. Take enchantments back "
                                              f"out onto books, up to level {cap}. Keeps them when broken.")
        lang_pt[f"block.{NS}.{name}.desc"] = (f"Coloque livros encantados: eles viram pontos. Tire os encantamentos de "
                                              f"volta em livros, até o nível {cap}. Guarda tudo ao ser quebrado.")
        tags.setdefault(("block", "minecraft:mineable/axe"), []).append(f"{NS}:{name}")
        items.append(f"{NS}:{name}")
    files[f"data/{NS}/horizon/menu/archive.json"] = {"slots": 2, "data_slots": 192}
    files[f"data/{NS}/recipe/arcane_archive.json"] = {
        "type": "minecraft:crafting_shaped", "category": "misc",
        "key": {"I": f"#{NS}:infused_shelves", "C": "minecraft:chiseled_bookshelf", "Y": "minecraft:ender_eye",
                "T": f"{NS}:{table}", "L": "minecraft:lapis_block"},
        "pattern": ["ICI", "YTY", "LLL"], "result": {"id": f"{NS}:arcane_archive"}}
    tags[("item", f"{NS}:infused_shelves")] = [f"{NS}:infused_ember_shelf", f"{NS}:infused_tide_shelf"]
    lang_en.update({
        "vastenchanting.archive.empty": "Nothing stored yet: put an enchanted book in the top slot.",
        "vastenchanting.archive.points": "%s points stored",
        "vastenchanting.archive.max": "Up to level %s (this archive keeps up to %s)",
        "vastenchanting.archive.cost": "Level %s costs %s points from a plain book",
        "vastenchanting.archive.click": "Left click: one level more on the bottom book",
        "vastenchanting.archive.click_max": "Right click: as high as the points allow",
        "vastenchanting.archive.holds": "Holds %s enchantments"})
    lang_pt.update({
        "vastenchanting.archive.empty": "Nada guardado ainda: coloque um livro encantado no espaço de cima.",
        "vastenchanting.archive.points": "%s pontos guardados",
        "vastenchanting.archive.max": "Até o nível %s (este arquivo guarda até %s)",
        "vastenchanting.archive.cost": "Nível %s custa %s pontos num livro comum",
        "vastenchanting.archive.click": "Clique esquerdo: um nível a mais no livro de baixo",
        "vastenchanting.archive.click_max": "Clique direito: o mais alto que os pontos permitem",
        "vastenchanting.archive.holds": "Guarda %s encantamentos"})
    files[f"assets/{NS}/textures/gui/archive.png"] = png(archive_gui_texture())

    files[f"assets/{NS}/textures/gui/table.png"] = png(gui_texture())
    for (registry, tname), values in tags.items():
        tns, path = tname.split(":")
        files[f"data/{tns}/tags/{registry}/{path}.json"] = {"values": sorted(set(values))}
    files[f"data/{NS}/horizon/creative_tab/main.json"] = {"title": "Vast Enchanting", "icon": f"{NS}:{table}", "items": items}
    files[f"assets/{NS}/lang/en_us.json"] = dict(sorted(lang_en.items()))
    files[f"assets/{NS}/lang/pt_br.json"] = dict(sorted(lang_pt.items()))

    out = Path(out) / NS
    if out.exists():
        shutil.rmtree(out)
    for path, value in files.items():
        target = out / path
        target.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(value, bytes):
            target.write_bytes(value)
        else:
            target.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")
    (out / "horizon-mod.toml").write_text(
        f'id = "{NS}"\nname = "Vast Enchanting (content)"\nversion = "{VERSION}"\n'
        'description = "The Vast Enchanting Table and its shelves: blocks, art, recipes and names. Needs the native mod."\n'
        'authors = ["voltzz900"]\napi = "0.2"\nside = "both"\ncapabilities = ["registries"]\n')
    write_rust()
    print(f"{NS}: {len(items)} blocks and items, {len(files) + 1} files")


def write_rust():
    """The shelves' numbers and the infusions, for the native mod: between the markers in its client.rs."""
    lines = ["/// A block the table reads: (block, max power it may raise to, power, chaos, rarity, clues).",
             "pub const STATS: &[(&str, f32, f32, f32, f32, i32)] = &["]
    for name, (mx, e, q, a, c) in [(f"{NS}:{n}", s) for n, _, _, _, s in SHELVES] + VANILLA_STATS:
        lines.append(f'    ("{name}", {float(mx)}, {float(e)}, {float(q)}, {float(a)}, {c}),')
    lines += ["];", "/// (input, result, minimum power, chaos, rarity, maximum power, chaos, rarity (-1 for none), how many).",
              "pub const INFUSIONS: &[(&str, &str, [f32; 3], [f32; 3], u32)] = &["]
    for inp, res, mn, mx, n in INFUSIONS:
        lines.append(f'    ("{inp}", "{res}", [{", ".join(str(float(v)) for v in mn)}], '
                     f'[{", ".join(str(float(v)) for v in mx)}], {n}),')
    lines.append("];")
    path = ROOT / "mixins/vastenchanting_native/client.rs"
    text = path.read_text()
    start, end = "// ---- generated: shelves (tools/build_content.py writes this part) ----\n", "// ---- end generated ----"
    a, b = text.index(start) + len(start), text.index(end)
    path.write_text(text[:a] + "\n".join(lines) + "\n" + text[b:])


if __name__ == "__main__":
    build(sys.argv[1] if len(sys.argv) > 1 else ROOT / "build")
