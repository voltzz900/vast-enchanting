//! Small helpers over saved data and item components.

use mod_api::{Compound, ItemStack, Tag};

/// A compound key or NBT string (the game's Java string), from `s`.
pub fn java<K: for<'a> From<&'a str>>(s: &str) -> K {
    K::from(s)
}

pub fn get<'a>(nbt: &'a Compound, key: &str) -> Option<&'a Tag> {
    nbt.iter().find(|(k, _)| k.0.iter().copied().eq(key.encode_utf16())).map(|(_, v)| v)
}

pub fn get_i(nbt: &Compound, key: &str, default: i32) -> i32 {
    match get(nbt, key) {
        Some(Tag::Int(v)) => *v,
        Some(Tag::Short(v)) => i32::from(*v),
        Some(Tag::Byte(v)) => i32::from(*v),
        // Item defaults are read from JSON: a number may come as a Long or a Double.
        Some(Tag::Long(v)) => (*v).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        Some(Tag::Double(v)) => f64::from_bits(*v) as i32,
        _ => default,
    }
}

pub fn set_i(nbt: &mut Compound, key: &str, value: i32) {
    nbt.insert(java(key), Tag::Int(value));
}

pub fn remove(nbt: &mut Compound, key: &str) {
    nbt.retain(|k, _| !k.0.iter().copied().eq(key.encode_utf16()));
}

pub fn compound<'a>(nbt: &'a Compound, key: &str) -> Option<&'a Compound> {
    match get(nbt, key) {
        Some(Tag::Compound(c)) => Some(c),
        _ => None,
    }
}

/// The compound under `key`, made (or made over) when it isn't one.
pub fn compound_mut<'a>(nbt: &'a mut Compound, key: &str) -> &'a mut Compound {
    if compound(nbt, key).is_none() {
        nbt.insert(java(key), Tag::Compound(Compound::new()));
    }
    let slot = nbt.iter_mut().find(|(k, _)| k.0.iter().copied().eq(key.encode_utf16())).map(|(_, v)| v);
    match slot {
        Some(Tag::Compound(c)) => c,
        _ => unreachable!("just made a compound"),
    }
}

/// The stack's own value for component `id` (not the item's default).
pub fn component<'a>(stack: &'a ItemStack, id: &str) -> Option<&'a Tag> {
    get(stack.components()?.as_nbt(), id)
}

pub const ENCHANTMENTS: &str = "minecraft:enchantments";
pub const STORED: &str = "minecraft:stored_enchantments";
pub const BLOCK_ENTITY_DATA: &str = "minecraft:block_entity_data";

/// The enchantments in component `id` (`minecraft:enchantments` or `minecraft:stored_enchantments`), by id.
pub fn levels(stack: &ItemStack, id: &str) -> Vec<(String, u32)> {
    let Some(Tag::Compound(map)) = component(stack, id) else { return Vec::new() };
    // Older saves wrap the map in {levels: {...}}.
    let map = compound(map, "levels").unwrap_or(map);
    map.iter()
        .filter_map(|(k, v)| {
            let level = match v {
                Tag::Int(l) => *l,
                Tag::Short(l) => i32::from(*l),
                Tag::Byte(l) => i32::from(*l),
                _ => return None,
            };
            let id = k.to_string_lossy();
            let id = if id.contains(':') { id } else { format!("minecraft:{id}") };
            (level > 0).then_some((id, level as u32))
        })
        .collect()
}

/// Every enchantment on the stack: its own and, for a book, those stored in it.
pub fn all_levels(stack: &ItemStack) -> Vec<(String, u32)> {
    let mut all = levels(stack, ENCHANTMENTS);
    all.extend(levels(stack, STORED));
    all
}

/// Sets component `id` to exactly `list`.
pub fn set_levels(stack: &mut ItemStack, id: &str, list: &[(String, u32)]) {
    let mut map = Compound::new();
    for (e, l) in list {
        map.insert(java(e), Tag::Int(*l as i32));
    }
    let mut patch = Compound::new();
    patch.insert(java(id), Tag::Compound(map));
    if let Err(e) = stack.apply_components(&patch) {
        eprintln!("Vast Enchanting: couldn't set {id}: {e}");
    }
}

/// Puts `id` at `level` into component `component` (replacing a level it had).
pub fn enchant(stack: &mut ItemStack, component: &str, id: &str, level: u32) {
    let mut list = levels(stack, component);
    list.retain(|(e, _)| e != id);
    list.push((id.to_owned(), level));
    set_levels(stack, component, &list);
}

/// The stack's own component patch, with `id` taken out (back to the item's default).
pub fn without_component(stack: &mut ItemStack, id: &str) {
    let mut patch = stack.components().map(|c| c.as_nbt().clone()).unwrap_or_default();
    remove(&mut patch, id);
    if let Err(e) = stack.set_components(patch) {
        eprintln!("Vast Enchanting: couldn't clear {id}: {e}");
    }
}

/// Sets component `id` to `value`.
pub fn set_component(stack: &mut ItemStack, id: &str, value: Tag) {
    let mut patch = Compound::new();
    patch.insert(java(id), value);
    if let Err(e) = stack.apply_components(&patch) {
        eprintln!("Vast Enchanting: couldn't set {id}: {e}");
    }
}
