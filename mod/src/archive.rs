//! The archives: enchanted books put in are kept as points per enchantment (a level-L book is worth 2^(L-1), so two
//! of a level make one of the next), with the highest level ever put in; a click writes an enchantment back onto the
//! book below, one level more (left) or as high as the points allow (right). What one holds goes with its item when
//! it's broken, and comes back when that item is placed.
//!
//! Saved data (kept from 0.1, so archives in old worlds load as they were): `points` and `levels`, compounds keyed by
//! enchantment id; slot 0 takes books in, slot 1 holds the book written onto.

use crate::data::ENCHANTMENTS;
use crate::enchant;
use crate::nbt::{self, BLOCK_ENTITY_DATA, STORED};
use crate::{Keys, keys};
use mod_api::{BlockEntityTickEvent, BlockEntityTickReply, Compound, ItemStack, MenuButtonEvent, Tag};

pub const ARCHIVE: &str = "vastenchanting:arcane_archive";
pub const VOID_ARCHIVE: &str = "vastenchanting:void_archive";
pub const MENU: &str = "vastenchanting:archive";
/// Enchantments the screen can show: three data slots each (points' low and high 15 bits, highest level).
pub const CAP: usize = 64;
pub const DATA_SLOTS: usize = CAP * 3;
pub const BOOK_IN: usize = 0;
pub const BOOK_OUT: usize = 1;

/// The highest level any archive keeps (the void archive's); past it a level's points stop doubling.
pub const MAX_LEVEL: u32 = 31;

/// The highest level an archive keeps.
pub fn cap_of(block: &str) -> Option<u32> {
    match block {
        ARCHIVE => Some(16),
        VOID_ARCHIVE => Some(31),
        _ => None,
    }
}

// ---- the arithmetic ------------------------------------------------------------------------------------------------

pub fn points_for(level: u32) -> i64 {
    if level == 0 { 0 } else { 1i64 << (level.min(MAX_LEVEL) - 1) }
}

/// The highest level the points pay for from a plain book, no higher than `level` (nor `MAX_LEVEL`).
pub fn affordable(points: i64, level: u32) -> u32 {
    (1..=level.min(MAX_LEVEL)).rev().find(|l| points_for(*l) <= points).unwrap_or(0)
}

/// `count` books of `level` put into an archive keeping up to `cap` that held `points` and a highest `top`: the new
/// points and highest level.
pub fn deposit(points: i64, top: u32, level: u32, count: u32, cap: u32) -> (i64, u32) {
    ((points + points_for(level) * i64::from(count.max(1))).min(points_for(cap)), top.max(level).min(cap))
}

/// Writing an enchantment at `current` on the book one level higher (or, with `max`, as high as the points allow):
/// the level it reaches and the points it takes, or None when it can't. Never past `MAX_LEVEL`, so the search is short
/// whatever the saved data says.
pub fn extract(current: u32, points: i64, level: u32, max: bool) -> Option<(u32, i64)> {
    let level = level.min(MAX_LEVEL);
    let pay = |target: u32| points_for(target) - points_for(current);
    let next = current.checked_add(1)?;
    let target = if max { (next..=level).rev().find(|t| pay(*t) <= points)? } else { next };
    (target > current && target <= level && pay(target) <= points).then(|| (target, pay(target)))
}

/// The data slots for what an archive holds: (points, highest level) by index in `ENCHANTMENTS`.
pub fn pack(held: impl Fn(&str) -> (i64, u32)) -> Vec<i32> {
    let mut data = vec![0; DATA_SLOTS];
    for (i, ench) in ENCHANTMENTS.iter().take(CAP).enumerate() {
        let (points, level) = held(ench.id);
        let points = points.clamp(0, (1 << 30) - 1);
        data[i * 3] = (points & 0x7FFF) as i32;
        data[i * 3 + 1] = (points >> 15) as i32;
        data[i * 3 + 2] = level as i32;
    }
    data
}

/// The enchantments the data slots say an archive holds, as (index in `ENCHANTMENTS`, points, highest level).
pub fn rows(data: &[i32]) -> Vec<(usize, i64, u32)> {
    if data.len() < DATA_SLOTS {
        return Vec::new();
    }
    (0..CAP.min(ENCHANTMENTS.len()))
        .filter_map(|i| {
            let points = i64::from(data[i * 3] & 0x7FFF) | i64::from(data[i * 3 + 1] & 0x7FFF) << 15;
            (points > 0).then_some((i, points, (data[i * 3 + 2].max(0) as u32).min(MAX_LEVEL)))
        })
        .collect()
}

// ---- saved data ----------------------------------------------------------------------------------------------------

/// An enchantment's stored points and highest level, the level no higher than the archive's `cap` (saved data from
/// elsewhere may say anything).
pub fn stored(nbt: &Compound, id: &str, cap: u32) -> (i64, u32) {
    let points = nbt::compound(nbt, "points").map_or(0, |c| nbt::get_i(c, id, 0).max(0) as i64);
    let level = nbt::compound(nbt, "levels").map_or(0, |c| nbt::get_i(c, id, 0).max(0) as u32);
    (points, level.min(cap).min(MAX_LEVEL))
}

fn set_stored(nbt: &mut Compound, id: &str, points: i64, level: u32) {
    if points <= 0 {
        nbt::remove(nbt::compound_mut(nbt, "points"), id);
        nbt::remove(nbt::compound_mut(nbt, "levels"), id);
        return;
    }
    nbt::set_i(nbt::compound_mut(nbt, "points"), id, points.min(i64::from(i32::MAX)) as i32);
    nbt::set_i(nbt::compound_mut(nbt, "levels"), id, level as i32);
}

fn data_of(nbt: &Compound, cap: u32) -> Vec<i32> {
    pack(|id| stored(nbt, id, cap))
}

// ---- the block entity ----------------------------------------------------------------------------------------------

/// Each tick: take in the book in the top slot, and show what the archive holds in the menu's data slots.
pub fn tick(event: &mut BlockEntityTickEvent<'_>, cap: u32) {
    let keys = keys();
    let book = event.entity.items.get(BOOK_IN).cloned().flatten();
    let mut changed = false;
    if let Some(book) = book.filter(|b| Some(b.item()) == keys.enchanted_book) {
        let list = nbt::levels(&book, STORED);
        if !list.is_empty() {
            let nbt = &mut event.entity.nbt;
            for (id, level) in &list {
                if enchant::index_of(id).is_none() {
                    continue;
                }
                let (points, top) = stored(nbt, id, cap);
                let (points, top) = deposit(points, top, *level, book.count(), cap);
                set_stored(nbt, id, points, top);
            }
            event.entity.items[BOOK_IN] = None;
            changed = true;
        }
    }
    let data = data_of(&event.entity.nbt, cap);
    if event.entity.data != data {
        event.entity.data = data;
        changed = true;
    }
    if !changed {
        *event.reply = BlockEntityTickReply::Unchanged;
    }
}

/// A row clicked: `id` is the enchantment's index times two, plus one for "as high as the points allow"; `cap` is
/// the highest level the archive keeps.
pub fn menu_button(event: &mut MenuButtonEvent<'_>, cap: u32) {
    let keys = keys();
    let id = event.context.id;
    if id < 0 {
        return;
    }
    let (index, max) = (id as usize / 2, id % 2 == 1);
    let Some(ench) = ENCHANTMENTS.get(index) else { return };
    let Some(book) = event.entity.items.get(BOOK_OUT).cloned().flatten() else { return };
    let plain = Some(book.item()) == keys.book;
    if !plain && Some(book.item()) != keys.enchanted_book {
        return;
    }
    let on_book = nbt::levels(&book, STORED);
    if on_book.iter().any(|(other, _)| other != ench.id && !enchant::compatible(ench.id, other)) {
        return;
    }
    let current = on_book.iter().find(|(other, _)| other == ench.id).map_or(0, |(_, l)| *l);
    let (points, level) = stored(&event.entity.nbt, ench.id, cap);
    let Some((target, pay)) = extract(current, points, level, max) else { return };
    let Some(mut out) = (if plain { to_enchanted_book(&keys, &book) } else { Some(book) }) else { return };
    nbt::enchant(&mut out, STORED, ench.id, target);
    event.entity.items[BOOK_OUT] = Some(out);
    set_stored(&mut event.entity.nbt, ench.id, points - pay, level);
    event.entity.data = data_of(&event.entity.nbt, cap);
}

/// A plain book as an enchanted book, keeping its own components (a name).
pub fn to_enchanted_book(keys: &Keys, book: &ItemStack) -> Option<ItemStack> {
    let mut out = ItemStack::new(keys.enchanted_book?, 1)?;
    if let Some(patch) = book.components() {
        let _ = out.set_components(patch.as_nbt().clone());
    }
    Some(out)
}

// ---- its item ------------------------------------------------------------------------------------------------------

/// What the archive's saved data `nbt` puts on its item: `minecraft:block_entity_data` with its points and levels,
/// under the block entity type `id`. None when it holds nothing.
pub fn item_data(nbt: &Compound, id: &str) -> Option<Tag> {
    let mut data = Compound::new();
    for key in ["points", "levels"] {
        if let Some(c) = nbt::compound(nbt, key).filter(|c| !c.is_empty()) {
            data.insert(nbt::java(key), Tag::Compound(c.clone()));
        }
    }
    if data.is_empty() {
        return None;
    }
    data.insert(nbt::java("id"), Tag::String(nbt::java(id)));
    Some(Tag::Compound(data))
}

/// The archive item's held enchantments, from its `minecraft:block_entity_data`.
pub fn held_count(stack: &ItemStack) -> usize {
    match nbt::component(stack, BLOCK_ENTITY_DATA) {
        Some(Tag::Compound(c)) => nbt::compound(c, "points").map_or(0, |p| p.len()),
        _ => 0,
    }
}

/// `block_drops`: a broken archive's item carries what it held (`nbt`, its saved data as the break read it).
pub fn on_drops(block: &str, nbt: Option<Compound>, drops: &mut [mod_api::ItemDrop], key: Option<mod_api::ItemKey>) {
    let (Some(nbt), Some(key)) = (nbt, key) else { return };
    let Some(data) = item_data(&nbt, block) else { return };
    if let Some(drop) = drops.iter_mut().find(|d| d.stack.item() == key) {
        nbt::set_component(&mut drop.stack, BLOCK_ENTITY_DATA, data);
    }
}

/// Whether an item may go in an archive's slot: enchanted books on top, one book (plain or enchanted) below.
pub fn slot_rule(slot: usize, item: mod_api::ItemKey, keys: &Keys) -> (bool, Option<u32>) {
    match slot {
        BOOK_IN => (Some(item) == keys.enchanted_book, None),
        BOOK_OUT => (Some(item) == keys.book || Some(item) == keys.enchanted_book, Some(1)),
        _ => (true, None),
    }
}

/// Infusing an archive into the next: the result keeps what it held, under its own type id.
pub fn carry_into(from: &ItemStack, to: &mut ItemStack, to_id: &str) {
    if let Some(Tag::Compound(mut held)) = nbt::component(from, BLOCK_ENTITY_DATA).cloned() {
        held.insert(nbt::java("id"), Tag::String(nbt::java(to_id)));
        nbt::set_component(to, BLOCK_ENTITY_DATA, Tag::Compound(held));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_double_by_level() {
        assert_eq!(points_for(0), 0);
        assert_eq!(points_for(1), 1);
        assert_eq!(points_for(5), 16);
        assert_eq!(affordable(16, 7), 5);
        assert_eq!(affordable(15, 7), 4);
        assert_eq!(affordable(100, 3), 3);
    }

    #[test]
    fn deposit_caps() {
        // Two level-3 books: 8 points, level 3.
        assert_eq!(deposit(0, 0, 3, 2, 16), (8, 3));
        // A level-20 book in an archive keeping 16: capped.
        assert_eq!(deposit(0, 0, 20, 1, 16), (points_for(16), 16));
    }

    #[test]
    fn extract_one_or_max() {
        // Holding 16 points up to level 7, a plain book: one level, or level 5 at most.
        assert_eq!(extract(0, 16, 7, false), Some((1, 1)));
        assert_eq!(extract(0, 16, 7, true), Some((5, 16)));
        // A book at 4 (8 points paid already) going to 5 costs 8 more.
        assert_eq!(extract(4, 8, 7, false), Some((5, 8)));
        assert_eq!(extract(4, 7, 7, false), None);
        // Never past the highest level put in.
        assert_eq!(extract(7, 1000, 7, false), None);
    }

    #[test]
    fn malformed_levels_are_bounded() {
        // A huge saved level with too few points: the max search stops at MAX_LEVEL, at once.
        assert_eq!(extract(0, 1, u32::MAX, true), Some((1, 1)));
        assert_eq!(extract(0, 0, u32::MAX, true), None);
        assert_eq!(extract(0, 0, u32::MAX, false), None);
        assert_eq!(extract(0, i64::MAX, u32::MAX, true), Some((MAX_LEVEL, points_for(MAX_LEVEL))));
        // A book already at a huge level: nothing to write, and no overflow.
        assert_eq!(extract(u32::MAX, i64::MAX, u32::MAX, false), None);
        assert_eq!(extract(u32::MAX, i64::MAX, u32::MAX, true), None);
        assert_eq!(affordable(0, u32::MAX), 0);
        assert_eq!(affordable(3, u32::MAX), 2);
        assert_eq!(affordable(i64::MAX, u32::MAX), MAX_LEVEL);
    }

    #[test]
    fn saved_levels_clamp_to_the_cap() {
        let mut nbt = Compound::new();
        nbt::set_i(nbt::compound_mut(&mut nbt, "points"), "minecraft:smite", 3);
        nbt::set_i(nbt::compound_mut(&mut nbt, "levels"), "minecraft:smite", i32::MAX);
        nbt::set_i(nbt::compound_mut(&mut nbt, "points"), "minecraft:power", -40);
        nbt::set_i(nbt::compound_mut(&mut nbt, "levels"), "minecraft:power", -5);
        assert_eq!(stored(&nbt, "minecraft:smite", 16), (3, 16));
        assert_eq!(stored(&nbt, "minecraft:smite", 31), (3, 31));
        assert_eq!(stored(&nbt, "minecraft:power", 16), (0, 0));
        assert_eq!(stored(&nbt, "minecraft:power", 31), (0, 0));
        // Too few points for anything past level 2, whatever the level says.
        let (points, level) = stored(&nbt, "minecraft:smite", 16);
        assert_eq!(extract(0, points, level, true), Some((2, 2)));
        // The shown level is clamped too.
        let data = pack(|id| stored(&nbt, id, 16));
        let rows = rows(&data);
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].1, rows[0].2), (3, 16));
    }

    #[test]
    fn rows_clamp_shown_levels() {
        let mut data = vec![0; DATA_SLOTS];
        data[0] = 5;
        data[2] = i32::MAX;
        data[3] = 5;
        data[5] = -9;
        let rows = rows(&data);
        assert_eq!(rows.iter().map(|r| r.2).collect::<Vec<_>>(), vec![MAX_LEVEL, 0]);
    }

    #[test]
    fn pack_round_trips() {
        let data = pack(|id| if id == "minecraft:sharpness" { (100_000, 9) } else { (0, 0) });
        let rows = rows(&data);
        assert_eq!(rows.len(), 1);
        assert_eq!(ENCHANTMENTS[rows[0].0].id, "minecraft:sharpness");
        assert_eq!((rows[0].1, rows[0].2), (100_000, 9));
    }

    #[test]
    fn saved_data_round_trips() {
        let mut nbt = Compound::new();
        set_stored(&mut nbt, "minecraft:smite", 64, 7);
        assert_eq!(stored(&nbt, "minecraft:smite", 16), (64, 7));
        let tag = item_data(&nbt, ARCHIVE).expect("held");
        let Tag::Compound(c) = tag else { panic!("compound") };
        assert_eq!(stored(&c, "minecraft:smite", 16), (64, 7));
        set_stored(&mut nbt, "minecraft:smite", 0, 7);
        assert_eq!(stored(&nbt, "minecraft:smite", 16), (0, 0));
        assert!(item_data(&nbt, ARCHIVE).is_none());
    }
}
