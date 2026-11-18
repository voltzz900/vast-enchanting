//! The Vast Enchanting Table on the server: its stats and offers into the menu's data slots, and an offer taken.
//!
//! Saved data (kept from 0.1): `seed`, the offers' seed; slot 0 holds the item, slot 1 the lapis.

use crate::archive;
use crate::data::{ENCHANTMENTS, INFUSIONS};
use crate::enchant::{self, BOOK, DATA_SLOTS, Input, Items, Stats};
use crate::nbt::{self, ENCHANTMENTS as ENCHANTED, STORED};
use crate::{keys, parse_item};
use mod_api::{
    BlockEntityTickEvent, BlockEntityTickReply, Host, Identifier, ItemStack, LevelRef, MenuButtonEvent, ServerHost,
    Tag,
};
use std::collections::HashMap;
use std::sync::Mutex;

pub const TABLE: &str = "vastenchanting:vast_enchanting_table";
pub const MENU: &str = "vastenchanting:table";
pub const ITEM: usize = 0;
pub const LAPIS: usize = 1;
/// Slots in a veto shelf (a hopper's menu).
pub const VETO_SLOTS: usize = 5;

/// What the selection asks of items, answered from the game's item defaults and merged tags.
pub struct HostItems<'a>(pub &'a dyn Host);

impl Items for HostItems<'_> {
    fn enchantability(&self, item: &str) -> i32 {
        let Some(key) = parse_item(self.0, item) else { return 0 };
        match self.0.item_default_component(key, "minecraft:enchantable") {
            Some(Tag::Compound(c)) => nbt::get_i(&c, "value", 0),
            Some(Tag::Int(v)) => v,
            _ => 0,
        }
    }

    fn in_set(&self, item: &str, set: &[&str]) -> bool {
        set.iter().any(|entry| match entry.strip_prefix('#') {
            Some(tag) => (|| {
                let key = parse_item(self.0, item)?;
                let tags = self.0.tags();
                tags.is_in_tag(key, tags.key(&Identifier::parse(tag).ok()?)?)
            })()
            .unwrap_or(false),
            None => *entry == item,
        })
    }
}

/// The enchantments a stack has, for the offers (an enchanted book's stored ones too).
fn input_enchantments(stack: &ItemStack) -> Vec<String> {
    nbt::all_levels(stack).into_iter().map(|(id, _)| id).collect()
}

/// The stats around the table at `pos` in `level`.
pub fn stats_at(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Stats {
    enchant::gather(
        pos,
        |at| level.block(host, at).map(|k| k.id(host)),
        |at| {
            let Some(entity) = host.block_entity(level, at) else { return 0 };
            let mut ids = Vec::new();
            for slot in 0..VETO_SLOTS as u32 {
                if let Ok(Some(stack)) = entity.item_slot(host, slot) {
                    ids.extend(input_enchantments(&stack));
                }
            }
            enchant::bits_of(ids.iter().map(String::as_str))
        },
    )
}

/// The offers for what lies on the table.
fn offers_for(host: &dyn Host, seed: i32, stats: Stats, stack: Option<&ItemStack>) -> enchant::Offers {
    let items = HostItems(host);
    let Some(stack) = stack else { return enchant::offers(&items, seed, stats, None) };
    let id = stack.item().id(host);
    let input = Input { item: &id, count: stack.count(), enchantments: input_enchantments(stack) };
    enchant::offers(&items, seed, stats, Some(&input))
}

/// Each table's tick counter, kept out of its saved data so a tick changes nothing saved.
static TICKS: Mutex<Option<HashMap<(String, [i32; 3]), u32>>> = Mutex::new(None);

fn count_tick(dimension: &str, pos: [i32; 3]) -> u32 {
    let Ok(mut ticks) = TICKS.lock() else { return 1 };
    let map = ticks.get_or_insert_with(HashMap::new);
    let t = map.entry((dimension.to_owned(), pos)).or_insert(0);
    *t = t.wrapping_add(1);
    *t
}

/// Every few ticks: the stats around the table and the offers for what lies on it, into the menu's data slots.
pub fn tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let dimension = host.dimension();
    let ready = event.entity.data.len() == DATA_SLOTS;
    if count_tick(dimension, pos) % 4 != 1 && ready {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    }
    let Some(level) = host.level(dimension) else {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    };
    let mut changed = false;
    let mut seed = nbt::get_i(&event.entity.nbt, "seed", 0);
    if seed == 0 {
        seed = enchant::first_seed(pos);
        nbt::set_i(&mut event.entity.nbt, "seed", seed);
        changed = true;
    }
    let host: &dyn Host = host;
    let stats = stats_at(host, level, pos);
    let stack = event.entity.items.get(ITEM).cloned().flatten();
    let offers = offers_for(host, seed, stats, stack.as_ref());
    let data = enchant::write_data(stats, &offers);
    if event.entity.data != data {
        event.entity.data = data;
        changed = true;
    }
    if !changed {
        *event.reply = BlockEntityTickReply::Unchanged;
    }
}

/// An offer clicked (`clickMenuButton`): check the levels and the lapis, then enchant or infuse, charge, and draw a
/// new seed.
pub fn menu_button(host: &mut dyn ServerHost, event: &mut MenuButtonEvent<'_>) {
    let keys = keys();
    let id = event.context.id;
    if !(0..3).contains(&id) {
        return;
    }
    let slot = id as usize;
    let pos = event.context.pos;
    let Some(player) = host.player(event.context.player) else { return };
    let Some(level) = player.level(host) else { return };
    let creative = player.is_creative(host).unwrap_or(false);
    let levels = player.experience(host).map_or(0, |e| e.level);
    let Some(stack) = event.entity.items.get(ITEM).cloned().flatten() else { return };
    let seed = match nbt::get_i(&event.entity.nbt, "seed", 0) {
        0 => enchant::first_seed(pos),
        s => s,
    };
    let offers = {
        let h: &dyn Host = host;
        offers_for(h, seed, stats_at(h, level, pos), Some(&stack))
    };
    let cost = offers.costs[slot];
    let need = slot as i32 + 1;
    let lapis = event.entity.items.get(LAPIS).cloned().flatten();
    let lapis_count = lapis.as_ref().filter(|l| Some(l.item()) == keys.lapis).map_or(0, |l| l.count() as i32);
    if cost <= 0 || (!creative && (lapis_count < need || levels < need || levels < cost)) {
        return;
    }
    let result = if slot == 2
        && let Some(i) = offers.infusion
    {
        let (_, to, _, _, count) = INFUSIONS[i];
        let Some(mut out) = parse_item(host, to).and_then(|k| ItemStack::new(k, count)) else { return };
        // An archive infused into the next keeps what it holds.
        archive::carry_into(&stack, &mut out, to);
        out
    } else {
        let list = &offers.lists[slot];
        if list.is_empty() {
            return;
        }
        let book = stack.item().id(host) == BOOK;
        let Some(mut out) = (if book { archive::to_enchanted_book(&keys, &stack) } else { Some(stack.clone()) }) else {
            return;
        };
        let component = if book { STORED } else { ENCHANTED };
        for (index, level) in list {
            nbt::enchant(&mut out, component, ENCHANTMENTS[*index].id, *level);
        }
        out
    };
    event.entity.items[ITEM] = Some(result);
    if !creative {
        event.entity.items[LAPIS] = lapis.and_then(|l| l.shrink(need as u32));
        player.give_levels(host, -need);
    }
    nbt::set_i(&mut event.entity.nbt, "seed", enchant::next_seed(seed, levels));
    event.entity.data.clear();
}
