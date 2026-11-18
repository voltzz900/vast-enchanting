//! Tomes at the anvil: an enchanted item on the left and a tome on the right give an enchanted book with the
//! enchantments the tome takes; a siphon tome's item comes back, plain and worn.

use crate::enchant::Rng;
use crate::nbt::{self, ENCHANTMENTS, STORED};
use crate::keys;
use mod_api::{AnvilCraftPostEvent, AnvilUpdateEvent, Host, HostSide, ItemStack, ServerHost};

pub const TATTERED: &str = "vastenchanting:tattered_tome";
pub const BOUND: &str = "vastenchanting:bound_tome";
pub const SIPHON: &str = "vastenchanting:siphon_tome";
pub const TOMES: [&str; 3] = [TATTERED, BOUND, SIPHON];

/// The enchantments a tome takes from an item (`item`'s id, its `damage` and enchantments `all`): all of them, or
/// for a tattered tome about half (at least one), picked from the item itself so the window and the server pick
/// alike.
pub fn tome_takes(tome: &str, item: &str, damage: u32, all: Vec<(String, u32)>) -> Vec<(String, u32)> {
    if tome != TATTERED || all.len() <= 1 {
        return all;
    }
    let mut seed: i64 = 0x7A77_E12E;
    for b in item.bytes().chain(all.iter().flat_map(|(id, l)| id.bytes().chain(l.to_le_bytes()))) {
        seed = seed.wrapping_mul(31).wrapping_add(i64::from(b));
    }
    let mut rng = Rng::new(seed ^ i64::from(damage));
    let mut order: Vec<usize> = (0..all.len()).collect();
    for i in (1..order.len()).rev() {
        order.swap(i, rng.int(i as i32 + 1) as usize);
    }
    let keep = all.len().div_ceil(2);
    let mut chosen: Vec<usize> = order[..keep].to_vec();
    chosen.sort_unstable();
    chosen.into_iter().map(|i| all[i].clone()).collect()
}

/// Levels a tome charges per enchantment.
fn per_enchantment(tome: &str) -> i32 {
    match tome {
        TATTERED => 2,
        BOUND => 3,
        _ => 5,
    }
}

/// `anvil_update`: a tome on the right takes the left item's enchantments into a book, for a few levels each and
/// one tome.
pub fn anvil_update(host: HostSide<'_>, event: &mut AnvilUpdateEvent<'_>) {
    let keys = keys();
    let host: &dyn Host = match host {
        HostSide::Server(h) => h,
        HostSide::Client(h) => h,
    };
    let update = &mut *event.update;
    let Some(right) = update.right.as_ref() else { return };
    let Some(tome) = keys.tome(right.item()) else { return };
    let left = &update.left;
    let book = Some(left.item()) == keys.book || Some(left.item()) == keys.enchanted_book;
    let taken = if book {
        Vec::new()
    } else {
        tome_takes(tome, &left.item().id(host), left.damage(), nbt::levels(left, ENCHANTMENTS))
    };
    let output = if taken.is_empty() {
        None
    } else {
        keys.enchanted_book.and_then(|k| ItemStack::new(k, 1)).map(|mut out| {
            nbt::set_levels(&mut out, STORED, &taken);
            out
        })
    };
    match output {
        Some(out) => {
            update.output = Some(out);
            update.xp_cost = (taken.len() as i32 * per_enchantment(tome)).clamp(1, 39);
            update.material_cost = 1;
        }
        None => {
            update.output = None;
            update.xp_cost = 0;
        }
    }
}

/// `anvil_craft_post`: a siphon tome's item comes back to the player without its enchantments and a quarter of its
/// durability worn off (never broken); what doesn't fit drops on the anvil.
pub fn anvil_craft_post(host: &mut dyn ServerHost, event: &mut AnvilCraftPostEvent<'_>) {
    let keys = keys();
    let craft = event.craft;
    let (Some(left), Some(right)) = (craft.left.as_ref(), craft.right.as_ref()) else { return };
    if Some(right.item()) != keys.siphon || nbt::levels(left, ENCHANTMENTS).is_empty() {
        return;
    }
    let Some(mut back) = ItemStack::new(left.item(), 1) else { return };
    if let Some(patch) = left.components() {
        let _ = back.set_components(patch.as_nbt().clone());
    }
    nbt::without_component(&mut back, ENCHANTMENTS);
    let max = host.max_damage(left.item()).unwrap_or(0);
    if max > 0 {
        back.set_damage((left.damage() + max / 4).min(max - 1));
    }
    let Some(player) = host.player(event.player) else { return };
    if player.give_stack(host, back.clone()).is_ok() {
        return;
    }
    let level = player.level(host).or_else(|| host.level(host.dimension()));
    let pos = event.pos;
    let dropped = level.map(|l| {
        l.spawn_item(host, f64::from(pos[0]) + 0.5, f64::from(pos[1]) + 1.1, f64::from(pos[2]) + 0.5, back, 10)
    });
    if !matches!(dropped, Some(Ok(_))) {
        eprintln!("Vast Enchanting: a siphoned item couldn't be given back or dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(n: u32) -> Vec<(String, u32)> {
        (1..=n).map(|i| (format!("minecraft:e{i}"), i)).collect()
    }

    #[test]
    fn bound_takes_all_tattered_half() {
        assert_eq!(tome_takes(BOUND, "minecraft:diamond_sword", 0, list(5)).len(), 5);
        let half = tome_takes(TATTERED, "minecraft:diamond_sword", 0, list(5));
        assert_eq!(half.len(), 3);
        // The same item picks the same half.
        assert_eq!(half, tome_takes(TATTERED, "minecraft:diamond_sword", 0, list(5)));
        assert_eq!(tome_takes(TATTERED, "minecraft:diamond_sword", 0, list(1)).len(), 1);
    }
}
