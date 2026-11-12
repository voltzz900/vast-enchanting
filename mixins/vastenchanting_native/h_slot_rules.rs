fn vastenchanting_slot_rules(menu: &str, slot: usize, stack: &ItemStack, may_place: &mut bool, max_stack: &mut u32) {
    if menu == "vastenchanting:table" {
        match slot {
            0 => *max_stack = 1,
            1 => *may_place = &*stack.item == "minecraft:lapis_lazuli",
            _ => {}
        }
    }
}
