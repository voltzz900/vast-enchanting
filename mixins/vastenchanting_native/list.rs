// The mod's row in Options > Mods.
fn vastenchanting_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "vastenchanting_native".into(),
        title: "Vast Enchanting 0.1.0".into(),
        description: "The Vast Enchanting Table: shelves give it power, chaos, rarity and clues.".into(),
        keys: Vec::new(),
    });
}
