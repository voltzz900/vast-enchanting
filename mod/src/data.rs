//! The table's numbers: written by tools/build_content.py (shelves and infusions from its lists,
//! enchantments from vanilla's data).

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
    ("vastenchanting:relic_shelf", 0.0, 0.0, 0.0, 5.0, 0),
    ("vastenchanting:veto_bookcase", 30.0, 1.0, 0.0, 0.0, 0),
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
    ("vastenchanting:arcane_archive", "vastenchanting:void_archive", [95.0, 40.0, 90.0], [-1.0, 50.0, -1.0], 1),
];

/// A vanilla enchantment, as the table's selection reads it.
pub struct Ench {
    pub id: &'static str,
    pub max_level: u32,
    pub weight: u32,
    /// Minimum cost: base and per level above the first.
    pub min_cost: (i32, i32),
    /// The items it may go on, and those a table offers it for (empty: the same): ids or `#tag`s.
    pub supported: &'static [&'static str],
    pub primary: &'static [&'static str],
    /// Enchantments it can't share an item with.
    pub exclusive: &'static [&'static str],
    /// In `#minecraft:in_enchanting_table`, `#minecraft:treasure`, `#minecraft:curse`.
    pub table: bool,
    pub treasure: bool,
    pub curse: bool,
}

pub const ENCHANTMENTS: &[Ench] = &[
    Ench { id: "minecraft:aqua_affinity", max_level: 1, weight: 2, min_cost: (1, 0), supported: &["#minecraft:enchantable/head_armor"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:bane_of_arthropods", max_level: 5, weight: 5, min_cost: (5, 8), supported: &["#minecraft:enchantable/weapon"], primary: &["#minecraft:enchantable/melee_weapon"], exclusive: &["minecraft:breach", "minecraft:density", "minecraft:impaling", "minecraft:sharpness", "minecraft:smite"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:binding_curse", max_level: 1, weight: 1, min_cost: (25, 0), supported: &["#minecraft:enchantable/equippable"], primary: &[], exclusive: &[], table: false, treasure: true, curse: true },
    Ench { id: "minecraft:blast_protection", max_level: 4, weight: 2, min_cost: (5, 8), supported: &["#minecraft:enchantable/armor"], primary: &[], exclusive: &["minecraft:fire_protection", "minecraft:projectile_protection", "minecraft:protection"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:breach", max_level: 4, weight: 2, min_cost: (15, 9), supported: &["#minecraft:enchantable/mace"], primary: &[], exclusive: &["minecraft:bane_of_arthropods", "minecraft:density", "minecraft:impaling", "minecraft:sharpness", "minecraft:smite"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:channeling", max_level: 1, weight: 1, min_cost: (25, 0), supported: &["#minecraft:enchantable/trident"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:density", max_level: 5, weight: 5, min_cost: (5, 8), supported: &["#minecraft:enchantable/mace"], primary: &[], exclusive: &["minecraft:bane_of_arthropods", "minecraft:breach", "minecraft:impaling", "minecraft:sharpness", "minecraft:smite"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:depth_strider", max_level: 3, weight: 2, min_cost: (10, 10), supported: &["#minecraft:enchantable/foot_armor"], primary: &[], exclusive: &["minecraft:frost_walker"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:efficiency", max_level: 5, weight: 10, min_cost: (1, 10), supported: &["#minecraft:enchantable/mining"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:feather_falling", max_level: 4, weight: 5, min_cost: (5, 6), supported: &["#minecraft:enchantable/foot_armor"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:fire_aspect", max_level: 2, weight: 2, min_cost: (10, 20), supported: &["#minecraft:enchantable/fire_aspect"], primary: &["#minecraft:enchantable/melee_weapon"], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:fire_protection", max_level: 4, weight: 5, min_cost: (10, 8), supported: &["#minecraft:enchantable/armor"], primary: &[], exclusive: &["minecraft:blast_protection", "minecraft:projectile_protection", "minecraft:protection"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:flame", max_level: 1, weight: 2, min_cost: (20, 0), supported: &["#minecraft:enchantable/bow"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:fortune", max_level: 3, weight: 2, min_cost: (15, 9), supported: &["#minecraft:enchantable/mining_loot"], primary: &[], exclusive: &["minecraft:silk_touch"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:frost_walker", max_level: 2, weight: 2, min_cost: (10, 10), supported: &["#minecraft:enchantable/foot_armor"], primary: &[], exclusive: &["minecraft:depth_strider"], table: false, treasure: true, curse: false },
    Ench { id: "minecraft:impaling", max_level: 5, weight: 2, min_cost: (1, 8), supported: &["#minecraft:enchantable/trident"], primary: &[], exclusive: &["minecraft:bane_of_arthropods", "minecraft:breach", "minecraft:density", "minecraft:sharpness", "minecraft:smite"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:infinity", max_level: 1, weight: 1, min_cost: (20, 0), supported: &["#minecraft:enchantable/bow"], primary: &[], exclusive: &["minecraft:mending"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:knockback", max_level: 2, weight: 5, min_cost: (5, 20), supported: &["#minecraft:enchantable/melee_weapon"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:looting", max_level: 3, weight: 2, min_cost: (15, 9), supported: &["#minecraft:enchantable/melee_weapon"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:loyalty", max_level: 3, weight: 5, min_cost: (12, 7), supported: &["#minecraft:enchantable/trident"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:luck_of_the_sea", max_level: 3, weight: 2, min_cost: (15, 9), supported: &["#minecraft:enchantable/fishing"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:lunge", max_level: 3, weight: 5, min_cost: (5, 8), supported: &["#minecraft:enchantable/lunge"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:lure", max_level: 3, weight: 2, min_cost: (15, 9), supported: &["#minecraft:enchantable/fishing"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:mending", max_level: 1, weight: 2, min_cost: (25, 25), supported: &["#minecraft:enchantable/durability"], primary: &[], exclusive: &[], table: false, treasure: true, curse: false },
    Ench { id: "minecraft:multishot", max_level: 1, weight: 2, min_cost: (20, 0), supported: &["#minecraft:enchantable/crossbow"], primary: &[], exclusive: &["minecraft:piercing"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:piercing", max_level: 4, weight: 10, min_cost: (1, 10), supported: &["#minecraft:enchantable/crossbow"], primary: &[], exclusive: &["minecraft:multishot"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:power", max_level: 5, weight: 10, min_cost: (1, 10), supported: &["#minecraft:enchantable/bow"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:projectile_protection", max_level: 4, weight: 5, min_cost: (3, 6), supported: &["#minecraft:enchantable/armor"], primary: &[], exclusive: &["minecraft:blast_protection", "minecraft:fire_protection", "minecraft:protection"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:protection", max_level: 4, weight: 10, min_cost: (1, 11), supported: &["#minecraft:enchantable/armor"], primary: &[], exclusive: &["minecraft:blast_protection", "minecraft:fire_protection", "minecraft:projectile_protection"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:punch", max_level: 2, weight: 2, min_cost: (12, 20), supported: &["#minecraft:enchantable/bow"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:quick_charge", max_level: 3, weight: 5, min_cost: (12, 20), supported: &["#minecraft:enchantable/crossbow"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:respiration", max_level: 3, weight: 2, min_cost: (10, 10), supported: &["#minecraft:enchantable/head_armor"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:riptide", max_level: 3, weight: 2, min_cost: (17, 7), supported: &["#minecraft:enchantable/trident"], primary: &[], exclusive: &["minecraft:channeling", "minecraft:loyalty"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:sharpness", max_level: 5, weight: 10, min_cost: (1, 11), supported: &["#minecraft:enchantable/sharp_weapon"], primary: &["#minecraft:enchantable/melee_weapon"], exclusive: &["minecraft:bane_of_arthropods", "minecraft:breach", "minecraft:density", "minecraft:impaling", "minecraft:smite"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:silk_touch", max_level: 1, weight: 1, min_cost: (15, 0), supported: &["#minecraft:enchantable/mining_loot"], primary: &[], exclusive: &["minecraft:fortune"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:smite", max_level: 5, weight: 5, min_cost: (5, 8), supported: &["#minecraft:enchantable/weapon"], primary: &["#minecraft:enchantable/melee_weapon"], exclusive: &["minecraft:bane_of_arthropods", "minecraft:breach", "minecraft:density", "minecraft:impaling", "minecraft:sharpness"], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:soul_speed", max_level: 3, weight: 1, min_cost: (10, 10), supported: &["#minecraft:enchantable/foot_armor"], primary: &[], exclusive: &[], table: false, treasure: true, curse: false },
    Ench { id: "minecraft:sweeping_edge", max_level: 3, weight: 2, min_cost: (5, 9), supported: &["#minecraft:enchantable/sweeping"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:swift_sneak", max_level: 3, weight: 1, min_cost: (25, 25), supported: &["#minecraft:enchantable/leg_armor"], primary: &[], exclusive: &[], table: false, treasure: true, curse: false },
    Ench { id: "minecraft:thorns", max_level: 3, weight: 1, min_cost: (10, 20), supported: &["#minecraft:enchantable/armor"], primary: &["#minecraft:enchantable/chest_armor"], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:unbreaking", max_level: 3, weight: 5, min_cost: (5, 8), supported: &["#minecraft:enchantable/durability"], primary: &[], exclusive: &[], table: true, treasure: false, curse: false },
    Ench { id: "minecraft:vanishing_curse", max_level: 1, weight: 1, min_cost: (25, 0), supported: &["#minecraft:enchantable/vanishing"], primary: &[], exclusive: &[], table: false, treasure: true, curse: true },
    Ench { id: "minecraft:wind_burst", max_level: 3, weight: 2, min_cost: (15, 9), supported: &["#minecraft:enchantable/mace"], primary: &[], exclusive: &[], table: false, treasure: true, curse: false },
];
