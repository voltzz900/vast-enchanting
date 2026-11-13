# Vast Enchanting

A native mod and a content mod for [Horizon](https://github.com/horizon-rust/horizon): an enchanting table of its
own that reads the shelves around it, offers enchantments past the game's maximum levels and infuses items; tomes
that take enchantments off items at the anvil; and archives that store enchanted books. The table's rules follow the
MIT code of [Apothic Enchanting](https://github.com/Shadows-of-Fire/Apothic-Enchanting) (by Shadows_of_Fire); the
blocks, items, art, recipes and words are this mod's own.

## The Vast Enchanting Table

Crafted from an enchanting table, a book, two diamonds and three obsidian. It has a screen of its own: the item and
the lapis on the left, three offers on the right, and the table's stats below.

It reads the blocks in the same ring a vanilla table reads its bookshelves from (two blocks out, at its height and
one above, with air between), and gathers four stats:

- **Power** (up to 100): how strong the offers get. The top offer costs this many levels. Each shelf adds power up
  to its own limit, so the highest shelves are needed to climb past each step.
- **Chaos**: how much each offer's strength varies, up or down. A table starts at 15.
- **Rarity**: favours rare enchantments; at 25 and at 75 every offer gains one more enchantment.
- **Clues**: how many of each offer's enchantments are shown before you choose.

Offers go past vanilla's maximum levels (a Sharpness VII, a Smite VIII), and the game uses those levels as they are.
Like vanilla, an offer costs 1 to 3 lapis and as many levels; the level shown is what you need to have.

### Shelves

| Shelf | Power (up to) | Chaos | Rarity | Clues |
| --- | --- | --- | --- | --- |
| Bookshelf, chiseled bookshelf | +2 (30) | | | |
| Ember Shelf | +3 (45) | +3 | | |
| Infused Ember Shelf | +5 (60) | +5 | | |
| Blazing Ember Shelf | +10 (65) | +10 | | −1 |
| Tide Shelf | +3 (45) | | +3 | |
| Infused Tide Shelf | +5 (60) | | +5 | |
| Heart of the Tide Shelf | +15 (60) | | +20 | |
| Dormant Abyss Shelf | +2 (30) | | | |
| Abyss Shelf | +5 (70) | +5 | +5 | |
| Echoing Abyss Shelf | +5 (75) | | +15 | |
| Void Shelf | +5 (90) | +5 | +5 | |
| Draconic Void Shelf | +20 (100) | | | |
| Shelf of Sight / Sight II | | | | +1 / +2 |
| Relic Shelf | | | +5 | lets the table offer **treasure** enchantments |
| Veto Shelf | +1 (30) | | | holds five enchanted books: their enchantments are **not offered** |
| Stellarite Shelf | +12 (100) | +5 | +5 | |
| Infernium Shelf | +8 (100) | +15 | | |
| Voidium Shelf | +8 (100) | | +15 | |

The last three need [Vast Depths](https://github.com/voltzz900/vast-ores) for their metals; without it they have no
recipe. A table that reads a relic shelf shows a gold star above its slots, and one that reads veto shelves a red
count of the enchantments kept out; hover them for a note.

### Infusions

With the right stats, the top offer of an item that can be infused is its infusion: the item becomes the result.

| Item | Becomes | Power | Chaos | Rarity |
| --- | --- | --- | --- | --- |
| Ember Shelf | Infused Ember Shelf | 45+ | 30+ | |
| Tide Shelf | Infused Tide Shelf | 45+ | 15+ | 10+ |
| Dormant Abyss Shelf | Abyss Shelf | 60+ | 40+ | 40+ |
| Shelf of Sight | Shelf of Sight II | 60+ | 20+ | 30+ |
| Carrot | Golden Carrot | exactly 20 | 10 to 30 | |
| Block of Amethyst | Budding Amethyst | 60+ | 30 to 50 | 50+ |
| Tattered Tome | 2 Bound Tomes | 40+ | 20 to 50 | 30+ |
| Bound Tome | 2 Siphon Tomes | 55+ | 25 to 70 | 45+ |
| Arcane Archive | Void Archive (keeps what it holds) | 95+ | 40 to 50 | 90+ |

## Tomes

Put an enchanted item on the left of an anvil and a tome on the right: the result is an enchanted book with the
item's enchantments, for a few levels per enchantment and one tome.

- **Tattered Tome** (crafted, two at a time, from string, books and an iron ingot): about half the enchantments,
  picked from the item itself. The item is lost. 2 levels each.
- **Bound Tome** (infused): all of them. The item is lost. 3 levels each.
- **Siphon Tome** (infused): all of them, and the item comes back without its enchantments and a quarter of its
  durability worn. 5 levels each. In single player it comes back on top of the anvil.

## Archives

An archive keeps enchanted books as points: a book of level N is worth 2^(N−1) points of each of its enchantments,
so two books of a level make one of the next. Put enchanted books in the top slot (hoppers can too); put a book or an
enchanted book in the bottom slot and click an enchantment in the list: **left click** writes it one level higher
on that book, **right click** as high as its points allow. An enchantment goes no higher than the highest level ever
put in, and never onto a book that holds one it's incompatible with. The list scrolls with the wheel or its bar.

- **Arcane Archive** (crafted from infused shelves, a chiseled bookshelf, ender eyes, a Vast Enchanting Table and
  lapis blocks): keeps up to level 16.
- **Void Archive** (an arcane archive infused at the table): keeps up to level 31.

A broken archive drops itself with everything it holds (its tooltip says how many enchantments), and placed again it
has them all.

## Not yet

- Apothic Enchanting's own enchantments: Horizon loads only the game's enchantments.
- The typed tomes that enchant like one kind of item, the scrap tome's random half and the prismatic web.
- The table's own particles and sounds.

## Install

- **The native mod:** `vastenchanting_native-<version>.hnmod`. Install it, then build natives in the launcher.
- **The content mod:** `vastenchanting-<version>.hmod`: blocks, items, art, recipes and words in English and
  Brazilian Portuguese. Install both.

## Building

- `tools/build_content.py` builds `build/vastenchanting` (art drawn by code, models, recipes, loot, lang) and writes
  the shelves' numbers and the infusions into `mixins/vastenchanting_native/client.rs`.
- `tools/package.py` packs both files into `dist/`.

It is pinned to Horizon `fdc87ae` (Nightly 2026-11-08). Besides the native hook points, it hooks the anvil's update
(`native_anvil_update`) for the tomes and a broken block's drops (`native_block_drops`) for the archives.

## License

MIT, see [LICENSE](LICENSE). Apothic Enchanting is by Shadows_of_Fire (Stormraven Studios, LLC): its code is MIT
([NOTICE-apothic-enchanting.txt](NOTICE-apothic-enchanting.txt)); its assets and lang are all rights reserved and
none are used. Not affiliated with or endorsed by Apothic Enchanting's authors.
