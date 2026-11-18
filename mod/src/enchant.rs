//! The table's arithmetic, with no Host in it: the stats around a table, the enchantments an offer gives and the
//! three offers for an item. It follows Apothic Enchanting's MIT code (Shadows_of_Fire): `ApothEnchantmentHelper`,
//! `EnchantmentTableStats`, `Arcana` and `PowerFunction`.

use crate::data::{ENCHANTMENTS, Ench, INFUSIONS, STATS};

// ---- a seeded random, so the offers shown are the offers given ---------------------------------------------------

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: i64) -> Self {
        Rng((seed as u64) ^ 0x9E37_79B9_7F4A_7C15)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn int(&mut self, bound: i32) -> i32 {
        if bound <= 1 {
            return 0;
        }
        (self.next() % bound as u64) as i32
    }
    pub fn float(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.float() * (hi - lo)
    }
    fn gaussian(&mut self) -> f32 {
        let u1 = self.float().max(1e-7);
        let u2 = self.float();
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
    }
}

// ---- the enchantments --------------------------------------------------------------------------------------------

/// An enchantment's index in `ENCHANTMENTS` (sorted by id), as data slots, buttons and the veto bits carry it.
pub fn index_of(id: &str) -> Option<usize> {
    ENCHANTMENTS.binary_search_by(|e| e.id.cmp(id)).ok()
}

pub fn ench(id: &str) -> Option<&'static Ench> {
    index_of(id).map(|i| &ENCHANTMENTS[i])
}

/// `Enchantment.areCompatible`: different, and neither in the other's exclusive set.
pub fn compatible(a: &str, b: &str) -> bool {
    let excludes = |x: &str, y: &str| ench(x).is_some_and(|e| e.exclusive.contains(&y));
    a != b && !excludes(a, b) && !excludes(b, a)
}

pub fn is_curse(id: &str) -> bool {
    ench(id).is_some_and(|e| e.curse)
}

fn min_cost(def: &Ench, level: u32) -> i32 {
    def.min_cost.0 + def.min_cost.1 * (level as i32 - 1)
}

/// `DefaultMinPowerFunction`: vanilla's minimum cost up to the vanilla maximum, then climbing past it.
pub fn min_power(def: &Ench, level: u32) -> i32 {
    let max = def.max_level;
    if level > max && level > 1 {
        let mut diff = min_cost(def, max) - min_cost(def, max.saturating_sub(1).max(1));
        if max == 1 {
            diff = 0;
        }
        if diff == 0 {
            diff = 15;
        }
        return min_cost(def, level) + diff * (f64::from(level - max)).powf(1.6) as i32;
    }
    min_cost(def, level)
}

/// `computeDefaultMaxLevel`: how high the table may take an enchantment (one-level ones stay at one).
pub fn max_level(def: &Ench) -> u32 {
    let mut level = def.max_level;
    if level <= 1 {
        return 1;
    }
    let mut p = min_power(def, level);
    if p >= 200 {
        return level;
    }
    let mut last = p;
    while p < 200 && level < 127 {
        level += 1;
        p = min_power(def, level);
        if p == last {
            return level;
        }
        if p > 200 {
            return level - 1;
        }
        last = p;
    }
    level
}

/// `Arcana`: the weight of each rarity (common, uncommon, rare, very rare) at a rarity stat.
fn rarity_weights(rarity: f32) -> [i32; 4] {
    const TIERS: [(f32, [i32; 4]); 11] = [
        (0.0, [10, 5, 2, 1]), (10.0, [8, 5, 3, 1]), (20.0, [7, 5, 4, 2]), (30.0, [5, 5, 4, 2]),
        (40.0, [5, 5, 4, 3]), (50.0, [5, 5, 5, 5]), (60.0, [3, 4, 5, 5]), (70.0, [2, 4, 5, 5]),
        (80.0, [2, 4, 5, 7]), (90.0, [1, 3, 5, 8]), (99.0, [1, 2, 5, 10]),
    ];
    TIERS.iter().rev().find(|t| rarity >= t.0).map_or(TIERS[0].1, |t| t.1)
}

fn weight_class(weight: u32) -> usize {
    match weight {
        w if w > 10 => 0,
        w if w > 5 => 1,
        w if w > 2 => 2,
        _ => 3,
    }
}

pub const BOOK: &str = "minecraft:book";
pub const ENCHANTED_BOOK: &str = "minecraft:enchanted_book";

/// What the selection asks of items: answered by the Host in the game, by a fake in the tests.
pub trait Items {
    /// The item's `minecraft:enchantable` value (0 for none).
    fn enchantability(&self, item: &str) -> i32;
    /// Whether the item is one of `set` (item ids, or `#tag`s).
    fn in_set(&self, item: &str, set: &[&str]) -> bool;
}

/// `isPrimaryItem`: the items a table offers it for (its supported items when it names none).
fn is_primary(items: &dyn Items, def: &Ench, item: &str) -> bool {
    items.in_set(item, if def.primary.is_empty() { def.supported } else { def.primary })
}

// ---- the stats around a table ------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    pub power: f32,
    pub chaos: f32,
    pub rarity: f32,
    pub clues: i32,
    /// A relic shelf is read: treasure enchantments may be offered.
    pub treasure: bool,
    /// Enchantments veto shelves keep out, by index in `ENCHANTMENTS`.
    pub blocked: u64,
}

pub const RELIC_SHELF: &str = "vastenchanting:relic_shelf";
/// Not "_shelf": Horizon gives a block with that ending the vanilla shelf's entity, not ours.
pub const VETO_SHELF: &str = "vastenchanting:veto_bookcase";

/// What a shelf gives a table: (max power, power, chaos, rarity, clues).
pub fn shelf_stats(block: &str) -> Option<(f32, f32, f32, f32, i32)> {
    STATS.iter().find(|s| s.0 == block).map(|s| (s.1, s.2, s.3, s.4, s.5))
}

/// Air a table reads a shelf through (`#minecraft:enchantment_power_transmitter`).
fn transmits(block: &str) -> bool {
    matches!(block, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:light")
}

/// The enchantment bits of a set of enchantment ids, by index in `ENCHANTMENTS`.
pub fn bits_of<'a>(ids: impl IntoIterator<Item = &'a str>) -> u64 {
    ids.into_iter().filter_map(index_of).filter(|i| *i < 64).fold(0, |b, i| b | 1 << i)
}

/// `EnchantmentTableStats.gatherStats`: each spot two blocks out (the ring a vanilla table reads), at the table's
/// height and one above, when the block between is air. Power climbs a ladder: each shelf adds up to its own limit.
/// `block_at` names the block at a position; `vetoed` gives a veto shelf's kept-out enchantments.
pub fn gather(pos: [i32; 3], block_at: impl Fn([i32; 3]) -> Option<String>, vetoed: impl Fn([i32; 3]) -> u64) -> Stats {
    let mut ladder: Vec<(f32, f32)> = Vec::new();
    let (mut chaos, mut rarity, mut clues) = (15.0f32, 0.0f32, 1i32);
    let (mut treasure, mut blocked) = (false, 0u64);
    for dy in 0..=1 {
        for dx in -2i32..=2 {
            for dz in -2i32..=2 {
                if dx.abs() != 2 && dz.abs() != 2 {
                    continue;
                }
                let between = block_at([pos[0] + dx / 2, pos[1] + dy, pos[2] + dz / 2]);
                if !between.as_deref().is_some_and(transmits) {
                    continue;
                }
                let at = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
                let Some(block) = block_at(at) else { continue };
                let Some((max, power, q, a, c)) = shelf_stats(&block) else { continue };
                match block.as_str() {
                    RELIC_SHELF => treasure = true,
                    VETO_SHELF => blocked |= vetoed(at),
                    _ => {}
                }
                match ladder.iter_mut().find(|(m, _)| *m == max) {
                    Some(step) => step.1 += power,
                    None => ladder.push((max, power)),
                }
                chaos += q;
                rarity += a;
                clues += c;
            }
        }
    }
    ladder.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut power = 0.0f32;
    for (max, add) in ladder {
        power = if max > 0.0 { max.min(power + add) } else { power + add };
    }
    Stats {
        power: power.clamp(0.0, 100.0),
        chaos: chaos.clamp(0.0, 100.0),
        rarity: rarity.clamp(0.0, 100.0),
        clues: clues.max(0),
        treasure,
        blocked,
    }
}

// ---- choosing enchantments ---------------------------------------------------------------------------------------

/// `selectEnchantment`: the enchantments an offer of `level` gives, as (index in `ENCHANTMENTS`, level).
pub fn select(items: &dyn Items, rng: &mut Rng, item: &str, level: i32, stats: Stats) -> Vec<(usize, u32)> {
    let mut chosen: Vec<(usize, u32)> = Vec::new();
    let book = item == BOOK;
    let enchantability = items.enchantability(item).max(i32::from(book));
    let mut level = level;
    if enchantability > 0 {
        let factor = 1.0 + stats.chaos * (rng.gaussian() / 3.0).clamp(-1.0, 1.0) / 100.0;
        let power = ((level as f32 * factor).round() as i32).clamp(1, 200);
        let weights = rarity_weights(stats.rarity);
        let mut possible: Vec<(usize, u32, i32)> = Vec::new();
        for (i, def) in ENCHANTMENTS.iter().enumerate() {
            let offered = def.table || (stats.treasure && def.treasure);
            if !offered || (i < 64 && stats.blocked & (1 << i) != 0) {
                continue;
            }
            if !book && !is_primary(items, def, item) {
                continue;
            }
            for l in (1..=max_level(def)).rev() {
                if power >= min_power(def, l) {
                    possible.push((i, l, weights[weight_class(def.weight)]));
                    break;
                }
            }
        }
        let pick = |rng: &mut Rng, chosen: &mut Vec<(usize, u32)>, possible: &mut Vec<(usize, u32, i32)>| {
            let total: i32 = possible.iter().map(|p| p.2).sum();
            let mut roll = rng.int(total.max(1));
            let mut index = 0;
            for (i, p) in possible.iter().enumerate() {
                roll -= p.2;
                if roll < 0 {
                    index = i;
                    break;
                }
            }
            let (id, l, _) = possible[index];
            let name = ENCHANTMENTS[id].id;
            possible.retain(|p| p.0 != id && compatible(name, ENCHANTMENTS[p.0].id));
            chosen.push((id, l));
        };
        // One enchantment always, a second at rarity 25 and a third at 75.
        for threshold in [0.0, 25.0, 75.0] {
            if stats.rarity >= threshold && !possible.is_empty() {
                pick(rng, &mut chosen, &mut possible);
            }
        }
        // Then more, each less likely than the last.
        let bound = 50.max((level as f32 * 1.15) as i32);
        while rng.int(bound) <= level && !possible.is_empty() {
            pick(rng, &mut chosen, &mut possible);
            level /= 2;
        }
    }
    // Enchantability: a chance for each enchantment to be one level higher.
    let mut chance = (enchantability as f32 / 100.0).min(0.80);
    if enchantability > 80 {
        chance += (3.5 * ((enchantability - 80) as f32).ln() / 100.0).clamp(0.0, 0.20);
    }
    for c in &mut chosen {
        if rng.float() < chance {
            c.1 += 1;
        }
    }
    chosen
}

// ---- the offers ---------------------------------------------------------------------------------------------------

/// How many of an offer's enchantments the screen can show.
pub const MAX_CLUES: usize = 6;

/// The item on the table, as the offers read it.
pub struct Input<'a> {
    pub item: &'a str,
    pub count: u32,
    /// The enchantments it has (stored ones for an enchanted book).
    pub enchantments: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Offers {
    pub costs: [i32; 3],
    /// The infusion the top offer is (an index into `INFUSIONS`).
    pub infusion: Option<usize>,
    /// Each offer's enchantments, in full, and those the clues show.
    pub lists: [Vec<(usize, u32)>; 3],
    pub shown: [Vec<(usize, u32)>; 3],
}

pub fn infusion_for(item: &str, stats: Stats) -> Option<usize> {
    INFUSIONS.iter().position(|(input, _, min, max, _)| {
        let v = [stats.power, stats.chaos, stats.rarity];
        *input == item && (0..3).all(|i| v[i] >= min[i] && (max[i] < 0.0 || v[i] <= max[i]))
    })
}

/// `slotsChanged`: the three offers for `input` at a table with `stats`, from the table's `seed`.
pub fn offers(items: &dyn Items, seed: i32, stats: Stats, input: Option<&Input<'_>>) -> Offers {
    let mut out = Offers::default();
    let Some(input) = input else { return out };
    let item = input.item;
    let infusion = INFUSIONS.iter().any(|i| i.0 == item);
    let enchantable = items.enchantability(item) > 0 || item == BOOK;
    let bare = input.enchantments.iter().all(|id| is_curse(id));
    if input.count != 1 || !(infusion || (enchantable && bare)) {
        return out;
    }
    let power = stats.power.max(1.5);
    let mut rng = Rng::new(i64::from(seed));
    for slot in 0..3 {
        let level = power.round() as i32;
        let mut cost = if slot == 2 {
            level
        } else {
            let lo = 0.6 - 0.4 * (1 - slot as i32) as f32;
            let hi = 0.8 - 0.4 * (1 - slot as i32) as f32;
            ((level as f32 * rng.range(lo, hi)).round() as i32).max(1)
        };
        if cost < slot as i32 + 1 {
            cost += 1;
        }
        out.costs[slot] = cost;
    }
    out.infusion = infusion_for(item, stats);
    if let Some(i) = out.infusion {
        let need = (INFUSIONS[i].2[0].ceil() as i32).max(3);
        out.costs[2] = out.costs[2].min(need);
    }
    for slot in 0..3 {
        if slot == 2 && out.infusion.is_some() {
            continue;
        }
        if !enchantable {
            out.costs[slot] = 0;
            continue;
        }
        let mut list_rng = Rng::new(i64::from(seed) + slot as i64);
        let list = select(items, &mut list_rng, item, out.costs[slot], stats);
        if list.is_empty() {
            out.costs[slot] = 0;
            continue;
        }
        let mut hidden = list.clone();
        let mut shown = Vec::new();
        let mut clues = stats.clues;
        while clues > 0 && !hidden.is_empty() && shown.len() < MAX_CLUES {
            shown.push(hidden.remove(list_rng.int(hidden.len() as i32) as usize));
            clues -= 1;
        }
        out.shown[slot] = shown;
        out.lists[slot] = list;
    }
    out
}

// ---- the table's data slots (16 bits each on the network) ---------------------------------------------------------

pub const D_COST: usize = 0; // 3: each offer's level, 0 for none
pub const D_POWER: usize = 3; // stats, times 10
pub const D_CHAOS: usize = 4;
pub const D_RARITY: usize = 5;
pub const D_CLUES: usize = 6;
pub const D_INFUSE: usize = 7; // 1 + the infusion the top offer is, or 0
pub const D_TOTAL: usize = 8; // 3: how many enchantments each offer gives
pub const D_CLUE: usize = 11; // 3 x 6 x (1 + enchantment index, level)
pub const D_TREASURE: usize = 47; // 1 when a relic shelf is read
pub const D_BLOCKED: usize = 48; // how many enchantments veto shelves keep out
pub const DATA_SLOTS: usize = 49;

pub fn write_data(stats: Stats, offers: &Offers) -> Vec<i32> {
    let mut data = vec![0; DATA_SLOTS];
    for slot in 0..3 {
        data[D_COST + slot] = offers.costs[slot];
        data[D_TOTAL + slot] = offers.lists[slot].len() as i32;
        for (k, (index, level)) in offers.shown[slot].iter().enumerate() {
            data[D_CLUE + slot * MAX_CLUES * 2 + k * 2] = *index as i32 + 1;
            data[D_CLUE + slot * MAX_CLUES * 2 + k * 2 + 1] = *level as i32;
        }
    }
    data[D_POWER] = (stats.power * 10.0).round() as i32;
    data[D_CHAOS] = (stats.chaos * 10.0).round() as i32;
    data[D_RARITY] = (stats.rarity * 10.0).round() as i32;
    data[D_CLUES] = stats.clues;
    data[D_INFUSE] = offers.infusion.map_or(0, |i| i as i32 + 1);
    data[D_TREASURE] = i32::from(stats.treasure);
    data[D_BLOCKED] = stats.blocked.count_ones() as i32;
    data
}

/// A clue in the data slots: (index in `ENCHANTMENTS`, level).
pub fn clue(data: &[i32], slot: usize, k: usize) -> Option<(usize, u32)> {
    let index = *data.get(D_CLUE + slot * MAX_CLUES * 2 + k * 2)?;
    let level = *data.get(D_CLUE + slot * MAX_CLUES * 2 + k * 2 + 1)?;
    (index > 0 && (index as usize) <= ENCHANTMENTS.len()).then(|| (index as usize - 1, level.max(1) as u32))
}

/// A fresh seed for a table at `pos` that has none.
pub fn first_seed(pos: [i32; 3]) -> i32 {
    (pos[0].wrapping_mul(73_856_093) ^ pos[1].wrapping_mul(19_349_663) ^ pos[2].wrapping_mul(83_492_791)) | 1
}

/// The seed after an offer is taken.
pub fn next_seed(seed: i32, levels: i32) -> i32 {
    let mut rng = Rng::new(i64::from(seed) ^ i64::from(levels) << 20 ^ 0x5EED);
    (rng.next() as i32) | 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Swords (enchantability 10) and books; every item tag holds the sword.
    struct Fake;
    impl Items for Fake {
        fn enchantability(&self, item: &str) -> i32 {
            match item {
                "minecraft:diamond_sword" => 10,
                BOOK => 1,
                _ => 0,
            }
        }
        fn in_set(&self, item: &str, set: &[&str]) -> bool {
            item == "minecraft:diamond_sword"
                && set.iter().any(|s| s.contains("weapon") || s.contains("sharp") || s.contains("durability"))
        }
    }

    fn ring(pos: [i32; 3], shelf: &'static str) -> impl Fn([i32; 3]) -> Option<String> {
        move |at: [i32; 3]| {
            let (dx, dz) = (at[0] - pos[0], at[2] - pos[2]);
            let dy = at[1] - pos[1];
            if (0..=1).contains(&dy) && (dx.abs() == 2 || dz.abs() == 2) {
                Some(shelf.to_owned())
            } else {
                Some("minecraft:air".to_owned())
            }
        }
    }

    #[test]
    fn enchantments_sorted_and_found() {
        assert!(ENCHANTMENTS.windows(2).all(|w| w[0].id < w[1].id));
        assert_eq!(ench("minecraft:sharpness").map(|e| e.max_level), Some(5));
        assert!(!compatible("minecraft:sharpness", "minecraft:smite"));
        assert!(compatible("minecraft:sharpness", "minecraft:unbreaking"));
        assert!(!compatible("minecraft:sharpness", "minecraft:sharpness"));
        assert!(is_curse("minecraft:binding_curse"));
    }

    #[test]
    fn power_climbs_past_vanilla() {
        let sharp = ench("minecraft:sharpness").expect("sharpness");
        assert_eq!(min_power(sharp, 1), 1);
        assert_eq!(min_power(sharp, 5), 45);
        assert!(min_power(sharp, 6) > min_power(sharp, 5));
        assert!(max_level(sharp) > 5);
        assert_eq!(max_level(ench("minecraft:mending").expect("mending")), 1);
    }

    #[test]
    fn bookshelves_reach_thirty() {
        let s = gather([0, 64, 0], ring([0, 64, 0], "minecraft:bookshelf"), |_| 0);
        // 32 spots of 2 power each, capped at 30.
        assert_eq!(s.power, 30.0);
        assert_eq!(s.chaos, 15.0);
        assert_eq!(s.clues, 1);
        assert!(!s.treasure);
    }

    #[test]
    fn void_shelves_and_blocked_air() {
        let s = gather([0, 64, 0], ring([0, 64, 0], "vastenchanting:void_shelf"), |_| 0);
        assert_eq!(s.power, 90.0);
        assert_eq!(s.chaos, (15.0f32 + 32.0 * 5.0).min(100.0));
        // No air between: nothing read.
        let walled = gather([0, 64, 0], |_| Some("minecraft:stone".to_owned()), |_| 0);
        assert_eq!(walled.power, 0.0);
    }

    #[test]
    fn relic_and_veto() {
        let s = gather([0, 64, 0], ring([0, 64, 0], RELIC_SHELF), |_| 0);
        assert!(s.treasure);
        let sharp = bits_of(["minecraft:sharpness"]);
        let v = gather([0, 64, 0], ring([0, 64, 0], VETO_SHELF), |_| sharp);
        assert_eq!(v.blocked, sharp);
        let stats = Stats { power: 50.0, chaos: 15.0, rarity: 50.0, clues: 3, blocked: sharp, ..Stats::default() };
        let sharpness = index_of("minecraft:sharpness").expect("sharpness");
        for seed in 1..200 {
            let mut rng = Rng::new(seed);
            assert!(select(&Fake, &mut rng, "minecraft:diamond_sword", 50, stats).iter().all(|(i, _)| *i != sharpness));
        }
    }

    #[test]
    fn offers_are_stable_and_compatible() {
        let stats = Stats { power: 80.0, chaos: 30.0, rarity: 80.0, clues: 2, ..Stats::default() };
        let input = Input { item: "minecraft:diamond_sword", count: 1, enchantments: Vec::new() };
        let a = offers(&Fake, 12345, stats, Some(&input));
        let b = offers(&Fake, 12345, stats, Some(&input));
        assert_eq!(a, b);
        assert_eq!(a.costs[2], 80);
        assert!(a.costs[0] >= 1 && a.costs[0] <= a.costs[1] && a.costs[1] <= a.costs[2]);
        for list in &a.lists {
            assert!(!list.is_empty());
            for (i, x) in list.iter().enumerate() {
                for y in &list[i + 1..] {
                    assert!(compatible(ENCHANTMENTS[x.0].id, ENCHANTMENTS[y.0].id));
                }
            }
        }
        assert!(a.shown.iter().all(|s| s.len() <= 2));
        // An enchanted item gets nothing; a stack of two neither.
        let enchanted = Input { enchantments: vec!["minecraft:sharpness".into()], ..input };
        assert_eq!(offers(&Fake, 1, stats, Some(&enchanted)).costs, [0, 0, 0]);
        let two = Input { item: "minecraft:diamond_sword", count: 2, enchantments: Vec::new() };
        assert_eq!(offers(&Fake, 1, stats, Some(&two)).costs, [0, 0, 0]);
    }

    #[test]
    fn data_slots_round_trip_clues() {
        let stats = Stats { power: 60.0, chaos: 20.0, rarity: 40.0, clues: 6, ..Stats::default() };
        let input = Input { item: BOOK, count: 1, enchantments: Vec::new() };
        let o = offers(&Fake, 7, stats, Some(&input));
        let data = write_data(stats, &o);
        assert_eq!(data.len(), DATA_SLOTS);
        assert_eq!(data[D_POWER], 600);
        for slot in 0..3 {
            for (k, shown) in o.shown[slot].iter().enumerate() {
                assert_eq!(clue(&data, slot, k), Some(*shown));
            }
        }
    }

    #[test]
    fn infusions_need_their_range() {
        let low = Stats { power: 20.0, chaos: 10.0, ..Stats::default() };
        assert!(infusion_for("minecraft:carrot", low).is_some());
        let high = Stats { power: 21.0, chaos: 10.0, ..Stats::default() };
        assert!(infusion_for("minecraft:carrot", high).is_none());
        let input = Input { item: "vastenchanting:ember_shelf", count: 1, enchantments: Vec::new() };
        let stats = Stats { power: 50.0, chaos: 30.0, ..Stats::default() };
        let o = offers(&Fake, 3, stats, Some(&input));
        assert_eq!(o.infusion, Some(0));
        assert_eq!(o.costs[2], 45);
        assert_eq!(o.costs[0], 0);
    }
}
