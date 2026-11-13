// Vast Enchanting: the Vast Enchanting Table and the shelves around it.
//
// The table reads four stats from the blocks around it (the spots a vanilla table reads its bookshelves from):
// power (how strong the offers get, up to level 100), chaos (how much each offer's strength varies), rarity
// (rare enchantments favoured, and an extra enchantment at 25 and 75) and clues (how many of each offer's
// enchantments are shown). It offers enchantments past vanilla's maximum levels, and infuses some items into others.
// The selection follows Apothic Enchanting's MIT code (Shadows_of_Fire): `ApothEnchantmentHelper`,
// `EnchantmentTableStats`, `Arcana` and `PowerFunction`.

use super::Game;
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::cell::RefCell;
use std::sync::Arc;

pub const TABLE: &str = "vastenchanting:vast_enchanting_table";
pub const MENU: &str = "vastenchanting:table";

// ---- generated: shelves (tools/build_content.py writes this part) ----
/// A block the table reads: (block, max power it may raise to, power, chaos, rarity, clues).
pub const STATS: &[(&str, f32, f32, f32, f32, i32)] = &[
    ("vastenchanting:ember_shelf", 45.0, 3.0, 3.0, 0.0, 0),
    ("vastenchanting:infused_ember_shelf", 60.0, 5.0, 5.0, 0.0, 0),
    ("vastenchanting:blazing_ember_shelf", 65.0, 10.0, 10.0, 0.0, -1),
    ("vastenchanting:tide_shelf", 45.0, 3.0, 0.0, 3.0, 0),
    ("vastenchanting:infused_tide_shelf", 60.0, 5.0, 0.0, 5.0, 0),
    ("vastenchanting:heart_tide_shelf", 60.0, 15.0, 0.0, 20.0, 0),
    ("vastenchanting:dormant_abyss_shelf", 30.0, 2.0, 0.0, 0.0, 0),
    ("vastenchanting:abyss_shelf", 70.0, 5.0, 5.0, 5.0, 0),
    ("vastenchanting:echoing_abyss_shelf", 75.0, 5.0, 0.0, 15.0, 0),
    ("vastenchanting:void_shelf", 90.0, 5.0, 5.0, 5.0, 0),
    ("vastenchanting:draconic_void_shelf", 100.0, 20.0, 0.0, 0.0, 0),
    ("vastenchanting:sight_shelf", 0.0, 0.0, 0.0, 0.0, 1),
    ("vastenchanting:sight_shelf_ii", 0.0, 0.0, 0.0, 0.0, 2),
    ("vastenchanting:stellarite_shelf", 100.0, 12.0, 5.0, 5.0, 0),
    ("vastenchanting:infernium_shelf", 100.0, 8.0, 15.0, 0.0, 0),
    ("vastenchanting:voidium_shelf", 100.0, 8.0, 0.0, 15.0, 0),
    ("minecraft:bookshelf", 30.0, 2.0, 0.0, 0.0, 0),
    ("minecraft:chiseled_bookshelf", 30.0, 2.0, 0.0, 0.0, 0),
];
/// (input, result, minimum power, chaos, rarity, maximum power, chaos, rarity (-1 for none), how many).
pub const INFUSIONS: &[(&str, &str, [f32; 3], [f32; 3], u32)] = &[
    ("vastenchanting:ember_shelf", "vastenchanting:infused_ember_shelf", [45.0, 30.0, 0.0], [-1.0, -1.0, -1.0], 1),
    ("vastenchanting:tide_shelf", "vastenchanting:infused_tide_shelf", [45.0, 15.0, 10.0], [-1.0, -1.0, -1.0], 1),
    ("vastenchanting:dormant_abyss_shelf", "vastenchanting:abyss_shelf", [60.0, 40.0, 40.0], [-1.0, -1.0, -1.0], 1),
    ("vastenchanting:sight_shelf", "vastenchanting:sight_shelf_ii", [60.0, 20.0, 30.0], [-1.0, -1.0, -1.0], 1),
    ("minecraft:carrot", "minecraft:golden_carrot", [20.0, 10.0, 0.0], [20.0, 30.0, -1.0], 1),
    ("minecraft:amethyst_block", "minecraft:budding_amethyst", [60.0, 30.0, 50.0], [-1.0, 50.0, -1.0], 1),
    ("vastenchanting:tattered_tome", "vastenchanting:bound_tome", [40.0, 20.0, 30.0], [-1.0, 50.0, -1.0], 2),
    ("vastenchanting:bound_tome", "vastenchanting:siphon_tome", [55.0, 25.0, 45.0], [-1.0, 70.0, -1.0], 2),
];
// ---- end generated ----

// The menu's data slots (16 bits each on the network).
const D_COST: usize = 0; // 3: each offer's level, 0 for none
const D_POWER: usize = 3; // stats, times 10
const D_CHAOS: usize = 4;
const D_RARITY: usize = 5;
const D_CLUES: usize = 6;
const D_INFUSE: usize = 7; // 1 + the infusion the top offer is, or 0
const D_TOTAL: usize = 8; // 3: how many enchantments each offer gives
const D_CLUE: usize = 11; // 3 x 6 x (1 + enchantment index, level)
const MAX_CLUES: usize = 6;
pub const DATA_SLOTS: usize = 49;

const ITEM: usize = 0;
const LAPIS: usize = 1;

fn java(s: &str) -> JavaString {
    JavaString::from(s)
}

fn get_i(nbt: &Compound, key: &str, default: i32) -> i32 {
    match nbt.get(&java(key)) {
        Some(Tag::Int(v)) => *v,
        _ => default,
    }
}

fn set_i(nbt: &mut Compound, key: &str, value: i32) {
    nbt.insert(java(key), Tag::Int(value));
}

fn lang(key: &str, english: &str) -> String {
    horizon_client::lang::get_or(key, english).to_string()
}

// ---- a seeded random, so the offers shown are the offers given ---------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: i64) -> Self {
        Rng((seed as u64) ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn int(&mut self, bound: i32) -> i32 {
        if bound <= 1 {
            return 0;
        }
        (self.next() % bound as u64) as i32
    }
    fn float(&mut self) -> f32 {
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

// ---- the stats around a table ------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    pub power: f32,
    pub chaos: f32,
    pub rarity: f32,
    pub clues: i32,
}

fn block_stats(block: &str) -> Option<(f32, f32, f32, f32, i32)> {
    STATS.iter().find(|s| s.0 == block).map(|s| (s.1, s.2, s.3, s.4, s.5))
}

/// Air a table reads a shelf through (`#minecraft:enchantment_power_transmitter`).
fn transmits(block: &str) -> bool {
    matches!(block, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:light")
}

/// `EnchantmentTableStats.gatherStats`: each spot two blocks out (the ring a vanilla table reads), at the table's
/// height and one above, when the block between is air. Power climbs a ladder: each shelf adds up to its own limit.
pub fn gather(game: &Game<'_>, pos: [i32; 3]) -> Stats {
    let mut ladder: Vec<(f32, f32)> = Vec::new();
    let (mut chaos, mut rarity, mut clues) = (15.0f32, 0.0f32, 1i32);
    for dy in 0..=1 {
        for dx in -2i32..=2 {
            for dz in -2i32..=2 {
                if dx.abs() != 2 && dz.abs() != 2 {
                    continue;
                }
                let between = game.native_block([pos[0] + dx / 2, pos[1] + dy, pos[2] + dz / 2]);
                if !transmits(&between) {
                    continue;
                }
                let block = game.native_block([pos[0] + dx, pos[1] + dy, pos[2] + dz]);
                let Some((max, power, q, a, c)) = block_stats(&block) else { continue };
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
    }
}

// ---- choosing enchantments ---------------------------------------------------------------------------------------

type Enchantments = horizon_client::enchant::Enchantments;
type Def = horizon_client::enchant::Enchantment;

/// `DefaultMinPowerFunction`: vanilla's minimum cost up to the vanilla maximum, then climbing past it.
fn min_power(def: &Def, level: u32) -> i32 {
    let max = def.max_level;
    if level > max && level > 1 {
        let mut diff = def.min_cost.calculate(max) - def.min_cost.calculate(max.saturating_sub(1).max(1));
        if max == 1 {
            diff = 0;
        }
        if diff == 0 {
            diff = 15;
        }
        return def.min_cost.calculate(level) + diff * (f64::from(level - max)).powf(1.6) as i32;
    }
    def.min_cost.calculate(level)
}

/// `computeDefaultMaxLevel`: how high the table may take an enchantment (one-level ones stay at one).
fn max_level(def: &Def) -> u32 {
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

fn is_book(item: &str) -> bool {
    item == "minecraft:book"
}

/// `selectEnchantment`: the enchantments an offer of `level` gives.
fn select(e: &Enchantments, items: &horizon_client::items::Items, rng: &mut Rng, item: &str, level: i32, stats: Stats) -> Vec<(Arc<str>, u32)> {
    let mut chosen: Vec<(Arc<str>, u32)> = Vec::new();
    let enchantability = e.enchantable(item).max(if is_book(item) { 1 } else { 0 }) as i32;
    let mut level = level;
    if enchantability > 0 {
        let factor = 1.0 + stats.chaos * (rng.gaussian() / 3.0).clamp(-1.0, 1.0) / 100.0;
        let power = ((level as f32 * factor).round() as i32).clamp(1, 200);
        let weights = rarity_weights(stats.rarity);
        let mut possible: Vec<(Arc<str>, u32, i32)> = Vec::new();
        for id in e.tag("minecraft:in_enchanting_table") {
            let Some(def) = e.get(id) else { continue };
            if !is_book(item) && !e.is_primary(items, def, item) {
                continue;
            }
            for l in (1..=max_level(def)).rev() {
                if power >= min_power(def, l) {
                    possible.push((id.clone(), l, weights[weight_class(def.weight)]));
                    break;
                }
            }
        }
        let pick = |rng: &mut Rng, chosen: &mut Vec<(Arc<str>, u32)>, possible: &mut Vec<(Arc<str>, u32, i32)>| {
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
            let (id, l, _) = possible[index].clone();
            possible.retain(|p| p.0 != id && e.compatible(&id, &p.0));
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

#[derive(Clone, Debug, Default)]
pub struct Offers {
    pub costs: [i32; 3],
    /// The infusion the top offer is (an index into `INFUSIONS`).
    pub infusion: Option<usize>,
    /// Each offer's enchantments, in full, and how many of them the clues show.
    pub lists: [Vec<(Arc<str>, u32)>; 3],
    pub shown: [Vec<(Arc<str>, u32)>; 3],
}

fn infusion_for(item: &str, stats: Stats) -> Option<usize> {
    INFUSIONS.iter().position(|(input, _, min, max, _)| {
        let v = [stats.power, stats.chaos, stats.rarity];
        *input == item && (0..3).all(|i| v[i] >= min[i] && (max[i] < 0.0 || v[i] <= max[i]))
    })
}

fn enchantable_enough(e: &Enchantments, stack: &ItemStack) -> bool {
    let list = if stack.item.as_ref() == "minecraft:enchanted_book" { stack.stored_enchantments() } else { stack.enchantments() };
    list.iter().all(|(id, _)| e.is_curse(id))
}

/// `slotsChanged`: the three offers for `stack` at a table with `stats`, from the table's `seed`.
pub fn offers(e: &Enchantments, items: &horizon_client::items::Items, seed: i32, stats: Stats, stack: Option<&ItemStack>) -> Offers {
    let mut out = Offers::default();
    let Some(stack) = stack else { return out };
    let item: &str = &stack.item;
    let infusion = INFUSIONS.iter().any(|i| i.0 == item);
    let enchantable = e.enchantable(item) > 0 || is_book(item);
    if stack.count != 1 || !(infusion || (enchantable && enchantable_enough(e, stack))) {
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
        let list = select(e, items, &mut list_rng, item, out.costs[slot], stats);
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

fn enchantment_index(e: &Enchantments, id: &str) -> i32 {
    e.tag("minecraft:in_enchanting_table").iter().position(|x| &**x == id).map_or(0, |i| i as i32 + 1)
}

fn write_data(e: &Enchantments, stats: Stats, offers: &Offers, data: &mut Vec<i32>) {
    data.clear();
    data.resize(DATA_SLOTS, 0);
    for slot in 0..3 {
        data[D_COST + slot] = offers.costs[slot];
        data[D_TOTAL + slot] = offers.lists[slot].len() as i32;
        for (k, (id, level)) in offers.shown[slot].iter().enumerate() {
            data[D_CLUE + slot * MAX_CLUES * 2 + k * 2] = enchantment_index(e, id);
            data[D_CLUE + slot * MAX_CLUES * 2 + k * 2 + 1] = *level as i32;
        }
    }
    data[D_POWER] = (stats.power * 10.0).round() as i32;
    data[D_CHAOS] = (stats.chaos * 10.0).round() as i32;
    data[D_RARITY] = (stats.rarity * 10.0).round() as i32;
    data[D_CLUES] = stats.clues;
    data[D_INFUSE] = offers.infusion.map_or(0, |i| i as i32 + 1);
}

// ---- the block entity ---------------------------------------------------------------------------------------------

pub fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && block == TABLE {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub fn block_entity_ticks(block: &str, ticks: &mut bool) {
    if block == TABLE {
        *ticks = true;
    }
}

pub fn block_entity_container(block: &str, out: &mut Option<ModContainer>) {
    if block == TABLE {
        *out = Some(ModContainer {
            slots: 2,
            faces: std::array::from_fn(|_| Vec::new()),
            inputs: Vec::new(),
            outputs: Vec::new(),
            menu: Some(MENU.to_owned()),
        });
    }
}

fn seed_of(entity: &mut ModBlockEntity, pos: [i32; 3]) -> i32 {
    let seed = get_i(&entity.nbt, "seed", 0);
    if seed != 0 {
        return seed;
    }
    let fresh = (pos[0].wrapping_mul(73_856_093) ^ pos[1].wrapping_mul(19_349_663) ^ pos[2].wrapping_mul(83_492_791)) | 1;
    set_i(&mut entity.nbt, "seed", fresh);
    fresh
}

/// Every few ticks: the stats around the table and the offers for what lies on it, into the menu's data slots.
pub(in crate::game) fn block_entity_tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if entity.id != TABLE {
        return;
    }
    // A counter of our own, so a tick changes nothing saved.
    let t = TICKS.with_borrow_mut(|m| {
        let slot = match m.iter_mut().find(|(p, _)| *p == pos) {
            Some(s) => s,
            None => {
                m.push((pos, 0));
                m.last_mut().expect("just pushed")
            }
        };
        slot.1 = slot.1.wrapping_add(1);
        slot.1
    });
    if t % 4 != 1 && entity.data.len() == DATA_SLOTS {
        return;
    }
    let seed = seed_of(entity, pos);
    let stats = gather(game, pos);
    let e = game.enchantments();
    let offers = offers(e, game.items(), seed, stats, entity.items.get(ITEM).and_then(Option::as_ref));
    let mut data = Vec::new();
    write_data(e, stats, &offers, &mut data);
    if entity.data != data {
        entity.data = data;
    }
}

thread_local! {
    /// Each table's tick counter (kept out of its saved data, so a tick doesn't count as a change).
    static TICKS: RefCell<Vec<([i32; 3], i32)>> = const { RefCell::new(Vec::new()) };
    /// The screen's last known cursor, from its tooltip hook (the draw hook isn't told it).
    static MOUSE: RefCell<[f32; 2]> = const { RefCell::new([-1.0, -1.0]) };
}


/// An offer clicked (`clickMenuButton`): check the levels and the lapis, then enchant or infuse, charge, and
/// draw a new seed.
pub(in crate::game) fn menu_button(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity, id: i32) {
    if entity.id != TABLE || !(0..3).contains(&id) {
        return;
    }
    let slot = id as usize;
    let Some(player) = game.native_acting_player() else { return };
    let creative = player.game_mode == 1;
    let seed = seed_of(entity, pos);
    let stats = gather(game, pos);
    let Some(stack) = entity.items.get(ITEM).cloned().flatten() else { return };
    let e = game.enchantments();
    let offers = offers(e, game.items(), seed, stats, Some(&stack));
    let cost = offers.costs[slot];
    let need = slot as i32 + 1;
    let lapis = entity.items.get(LAPIS).cloned().flatten();
    let lapis_count = lapis.as_ref().map_or(0, |l| l.count as i32);
    let levels = game.experience.level;
    if cost <= 0 || (!creative && (lapis_count < need || levels < need || levels < cost)) {
        return;
    }
    let result = if slot == 2 && let Some(i) = offers.infusion {
        ItemStack::new(INFUSIONS[i].1, INFUSIONS[i].4)
    } else {
        let list = &offers.lists[slot];
        if list.is_empty() {
            return;
        }
        let mut out = if is_book(&stack.item) { stack.transmute("minecraft:enchanted_book", 1) } else { stack.clone() };
        for (id, level) in list {
            out.enchant(id, *level);
        }
        out
    };
    entity.items[ITEM] = Some(result);
    if !creative {
        entity.items[LAPIS] = lapis.and_then(|mut l| {
            l.count = l.count.saturating_sub(need as u32);
            (l.count > 0).then_some(l)
        });
        game.native_give_levels(player.uuid, -need);
    }
    let mut rng = Rng::new(i64::from(seed) ^ i64::from(levels) << 20 ^ 0x5EED);
    set_i(&mut entity.nbt, "seed", (rng.next() as i32) | 1);
    entity.data.clear();
}

// ---- the screen ---------------------------------------------------------------------------------------------------

const GUI: &str = "vastenchanting:gui/table";
const OFFER: [f32; 4] = [60.0, 14.0, 108.0, 19.0];
const BARS_X: f32 = 52.0;
const BAR_W: f32 = 92.0;

pub fn screen(block: &str, out: &mut Option<super::native_hooks::NativeScreen>) {
    if block == TABLE {
        *out = Some(super::native_hooks::NativeScreen {
            width: 176.0,
            height: 194.0,
            slots: vec![Some([15.0, 47.0]), Some([35.0, 47.0])],
            inventory_y: 112.0,
            inventory_x: 8.0,
            own_labels: true,
            own_background: true,
        });
    }
}

fn offer_rect(i: usize) -> [f32; 4] {
    [OFFER[0], OFFER[1] + 19.0 * i as f32, OFFER[2], OFFER[3]]
}

fn inside(r: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= r[0] && at[0] < r[0] + r[2] && at[1] >= r[1] && at[1] < r[1] + r[3]
}

fn clue(e: &Enchantments, data: &[i32], slot: usize, k: usize) -> Option<(String, bool)> {
    let index = data[D_CLUE + slot * MAX_CLUES * 2 + k * 2];
    let level = data[D_CLUE + slot * MAX_CLUES * 2 + k * 2 + 1];
    if index <= 0 {
        return None;
    }
    let id = e.tag("minecraft:in_enchanting_table").get(index as usize - 1)?;
    Some(e.full_name(id, level.max(1) as u32))
}

fn fit(game: &Game<'_>, text: &str, width: f32) -> String {
    if game.native_text_width(text) <= width {
        return text.to_owned();
    }
    let mut s: String = text.to_owned();
    while !s.is_empty() && game.native_text_width(&format!("{s}…")) > width {
        s.pop();
    }
    format!("{s}…")
}

fn infusion_name(game: &Game<'_>, i: usize) -> String {
    let id = INFUSIONS[i].1;
    let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
    let block_key = format!("block.{ns}.{path}");
    let item_key = format!("item.{ns}.{path}");
    let name = lang(&block_key, "");
    if !name.is_empty() && name != block_key {
        return name;
    }
    let _ = game;
    lang(&item_key, path)
}

pub(in crate::game) fn screen_draw(game: &Game<'_>, block: &str, data: &[i32], batch: &mut HudBatch, origin: [f32; 2]) {
    if block != TABLE {
        return;
    }
    let [ox, oy] = origin;
    game.native_sprite(batch, GUI, [0.0, 0.0, 176.0, 194.0], [ox, oy, 176.0, 194.0]);
    game.native_text(batch, &lang("container.vastenchanting.table", "Vast Enchanting Table"), [ox + 8.0, oy + 5.0], 0x404040, false);
    if data.len() < DATA_SLOTS {
        return;
    }
    let mouse = MOUSE.with_borrow(|m| *m);
    let e = game.enchantments();
    let levels = game.experience.level;
    for i in 0..3 {
        let r = offer_rect(i);
        let cost = data[D_COST + i];
        let off = cost <= 0;
        let v = if off { 233.0 } else if inside(r, mouse) { 214.0 } else { 195.0 };
        game.native_sprite(batch, GUI, [0.0, v, 108.0, 19.0], [ox + r[0], oy + r[1], 108.0, 19.0]);
        if off {
            continue;
        }
        // The badge: lapis it takes (1 to 3); the level it needs on the right.
        game.native_text(batch, &(i + 1).to_string(), [ox + r[0] + 7.0, oy + r[1] + 6.0], 0x9AD6FF, true);
        let level = cost.to_string();
        let colour = if levels >= cost { 0x80FF20 } else { 0xFF6A6A };
        let lw = game.native_text_width(&level);
        game.native_text(batch, &level, [ox + r[0] + r[2] - lw - 4.0, oy + r[1] + 6.0], colour, true);
        // An infusion offer shows its result's name in pink; the tooltip says "Infuse into".
        let (text, tint) = if i == 2 && data[D_INFUSE] > 0 {
            (infusion_name(game, data[D_INFUSE] as usize - 1), 0xFF9CFF)
        } else {
            (clue(e, data, i, 0).map_or_else(|| lang("vastenchanting.unknown", "Unknown enchantments"), |c| c.0), 0xE6DCF0)
        };
        let room = r[2] - 18.0 - lw - 5.0;
        game.native_text(batch, &fit(game, &text, room), [ox + r[0] + 17.0, oy + r[1] + 6.0], tint, true);
    }
    // The stats.
    let rows = [
        ("vastenchanting.power", "Power", D_POWER, 0.0),
        ("vastenchanting.chaos", "Chaos", D_CHAOS, 1.0),
        ("vastenchanting.rarity", "Rarity", D_RARITY, 2.0),
    ];
    for (k, (key, english, d, n)) in rows.iter().enumerate() {
        let y = oy + 77.0 + k as f32 * 10.0;
        game.native_text(batch, &lang(key, english), [ox + 10.0, y], 0x3A2A4A, false);
        let value = data[*d] as f32 / 10.0;
        game.native_sprite(batch, GUI, [110.0, 195.0, BAR_W, 6.0], [ox + BARS_X, y + 1.0, BAR_W, 6.0]);
        let w = (value / 100.0).clamp(0.0, 1.0) * BAR_W;
        game.native_sprite(batch, GUI, [110.0, 201.0 + n * 6.0, w, 6.0], [ox + BARS_X, y + 1.0, w, 6.0]);
        let shown = format!("{:.0}", value);
        let sw = game.native_text_width(&shown);
        game.native_text(batch, &shown, [ox + 167.0 - sw, y], 0x3A2A4A, false);
    }
    let clues = lang("vastenchanting.clues", "Clues: %s").replacen("%s", &data[D_CLUES].to_string(), 1);
    let cw = game.native_text_width(&clues);
    game.native_text(batch, &clues, [ox + 168.0 - cw, oy + 5.0], 0x404040, false);
}

pub fn screen_click(block: &str, data: &[i32], at: [f32; 2], right: bool, button: &mut Option<i32>) {
    if block != TABLE || right || data.len() < DATA_SLOTS {
        return;
    }
    for i in 0..3 {
        if data[D_COST + i] > 0 && inside(offer_rect(i), at) {
            *button = Some(i as i32);
        }
    }
}

pub(in crate::game) fn screen_tooltip(game: &Game<'_>, block: &str, data: &[i32], at: [f32; 2], lines: &mut Vec<String>) {
    if block != TABLE {
        return;
    }
    MOUSE.with_borrow_mut(|m| *m = at);
    if data.len() < DATA_SLOTS {
        return;
    }
    let e = game.enchantments();
    for i in 0..3 {
        let cost = data[D_COST + i];
        if cost <= 0 || !inside(offer_rect(i), at) {
            continue;
        }
        if i == 2 && data[D_INFUSE] > 0 {
            lines.push(format!("§d{}", lang("vastenchanting.infuse", "Infuse into %s").replacen("%s", &infusion_name(game, data[D_INFUSE] as usize - 1), 1)));
        } else {
            let mut shown = 0;
            for k in 0..MAX_CLUES {
                if let Some((name, curse)) = clue(e, data, i, k) {
                    lines.push(format!("{}{name}", if curse { "§c" } else { "§7" }));
                    shown += 1;
                }
            }
            if shown == 0 {
                lines.push(format!("§7{}", lang("vastenchanting.unknown", "Unknown enchantments")));
            }
            let more = data[D_TOTAL + i] - shown;
            if more > 0 && shown > 0 {
                lines.push(format!("§8{}", lang("vastenchanting.more", "...and %s more").replacen("%s", &more.to_string(), 1)));
            }
        }
        let levels = game.experience.level;
        let colour = if levels >= cost { "§a" } else { "§c" };
        lines.push(format!("{colour}{}", lang("vastenchanting.levels", "%s levels, %s lapis").replacen("%s", &cost.to_string(), 1).replacen("%s", &(i + 1).to_string(), 1)));
        return;
    }
    let rows = [("vastenchanting.power", "Power", "How strong the offers get: the top offer costs this many levels."),
                ("vastenchanting.chaos", "Chaos", "How much each offer's strength varies, up or down."),
                ("vastenchanting.rarity", "Rarity", "Favours rare enchantments; at 25 and 75 an offer gains another.")];
    for (k, (key, english, desc)) in rows.iter().enumerate() {
        let r = [8.0, 76.0 + k as f32 * 10.0, 160.0, 10.0];
        if inside(r, at) {
            lines.push(format!("§f{}", lang(key, english)));
            lines.push(format!("§7{}", lang(&format!("{key}.desc"), desc)));
        }
    }
}

/// Shelves' tooltips: what they give a table.
pub fn tooltip(stack: &ItemStack, lines: &mut Vec<(horizon_client::text::Text, u32)>) {
    use horizon_client::text::Text;
    if TOMES.contains(&stack.item.as_ref()) {
        let key = format!("item.{}.desc", stack.item.replacen(':', ".", 1));
        lines.push((Text::legacy(&format!("§7{}", lang(&key, ""))), u32::MAX));
        return;
    }
    let Some((max, power, chaos, rarity, clues)) = block_stats(&stack.item) else { return };
    if !stack.item.starts_with("vastenchanting:") {
        return;
    }
    if power != 0.0 || chaos != 0.0 || rarity != 0.0 {
        let line = lang("vastenchanting.shelf.stats", "Power +%s (up to %s) · Chaos %s · Rarity %s")
            .replacen("%s", &format!("{power}"), 1)
            .replacen("%s", &format!("{max}"), 1)
            .replacen("%s", &format!("{chaos:+}"), 1)
            .replacen("%s", &format!("{rarity:+}"), 1);
        lines.push((Text::legacy(&format!("§7{line}")), u32::MAX));
    }
    if clues != 0 {
        let line = lang("vastenchanting.shelf.clues", "Clues %s").replacen("%s", &format!("{clues:+}"), 1);
        lines.push((Text::legacy(&format!("§7{line}")), u32::MAX));
    }
}

// ---- tomes at the anvil -------------------------------------------------------------------------------------------

const TATTERED: &str = "vastenchanting:tattered_tome";
const BOUND: &str = "vastenchanting:bound_tome";
const SIPHON: &str = "vastenchanting:siphon_tome";
const TOMES: [&str; 3] = [TATTERED, BOUND, SIPHON];

/// The enchantments a tome takes from `left`: all of them, or for a tattered tome about half (at least one), picked
/// from the item itself so the window and the server pick alike.
fn tome_takes(tome: &str, left: &ItemStack) -> Vec<(Arc<str>, u32)> {
    let all = left.enchantments().to_vec();
    if tome != TATTERED || all.len() <= 1 {
        return all;
    }
    let mut seed: i64 = 0x7A77_E12E;
    for b in left.item.bytes().chain(all.iter().flat_map(|(id, l)| id.bytes().chain(l.to_le_bytes()))) {
        seed = seed.wrapping_mul(31).wrapping_add(i64::from(b));
    }
    let mut rng = Rng::new(seed ^ i64::from(left.damage));
    let mut order: Vec<usize> = (0..all.len()).collect();
    for i in (1..order.len()).rev() {
        order.swap(i, rng.int(i as i32 + 1) as usize);
    }
    let keep = all.len().div_ceil(2);
    let mut chosen: Vec<usize> = order[..keep].to_vec();
    chosen.sort_unstable();
    chosen.into_iter().map(|i| all[i].clone()).collect()
}

/// `native_anvil_update`: an enchanted item on the left and a tome on the right give an enchanted book with the
/// enchantments the tome takes, for a few levels a enchantment and one tome.
pub fn anvil(left: &ItemStack, right: Option<&ItemStack>, output: &mut Option<ItemStack>, xp_cost: &mut i32, material_cost: &mut i32) {
    let Some(right) = right else { return };
    let Some(tome) = TOMES.iter().copied().find(|t| *t == right.item.as_ref()) else { return };
    let taken = if is_book(&left.item) || left.item.as_ref() == "minecraft:enchanted_book" { Vec::new() } else { tome_takes(tome, left) };
    if taken.is_empty() {
        *output = None;
        *xp_cost = 0;
        return;
    }
    let per = match tome {
        TATTERED => 2,
        BOUND => 3,
        _ => 5,
    };
    let mut book = ItemStack::new("minecraft:enchanted_book", 1);
    book.set_crafting_enchantments(taken.clone());
    *output = Some(book);
    *xp_cost = (taken.len() as i32 * per).clamp(1, 39);
    *material_cost = 1;
}

/// `anvil_craft_post`: a siphon tome's item comes back to the player without its enchantments and a quarter of its
/// durability worn off (never broken); what doesn't fit drops on the anvil.
pub(in crate::game) fn anvil_taken(game: &mut Game<'_>, player: u128, pos: [i32; 3], craft: &horizon_client::menu::AnvilCraft) {
    let (Some(left), Some(right)) = (craft.left.as_ref(), craft.right.as_ref()) else { return };
    if right.item.as_ref() != SIPHON || left.enchantments().is_empty() {
        return;
    }
    let mut back = left.clone();
    back.count = 1;
    back.data_mut().enchantments.clear();
    back.tidy();
    let max = game.items().get(&back.item).map_or(0, |i| i.max_damage);
    if max > 0 {
        back.damage = (back.damage + max / 4).min(max - 1);
    }
    if let Some(rest) = game.native_give_stack(player, back) {
        let dimension = game.dim.id().to_owned();
        let item = horizon_items::item_nbt::item_to_nbt(&rest);
        let nbt: Compound = [("Item", item), ("PickupDelay", Tag::Short(10))].into_iter().map(|(k, v)| (k.into(), v)).collect();
        let at = [f64::from(pos[0]) + 0.5, f64::from(pos[1]) + 1.1, f64::from(pos[2]) + 0.5];
        if game.native_summon(&dimension, "minecraft:item", at, nbt).is_none() {
            log::warn!("Vast Enchanting: a siphoned item couldn't be dropped: {}", rest.item);
        }
    }
}
