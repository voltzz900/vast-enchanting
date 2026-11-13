fn vastenchanting_slot_rules(menu: &str, slot: usize, stack: &ItemStack, may_place: &mut bool, max_stack: &mut u32) {
    if menu == "vastenchanting:table" {
        match slot {
            0 => *max_stack = 1,
            1 => *may_place = &*stack.item == "minecraft:lapis_lazuli",
            _ => {}
        }
    } else if menu == "vastenchanting:archive" {
        // Top: enchanted books going in. Bottom: one book (plain or enchanted) to write onto.
        match slot {
            0 => *may_place = &*stack.item == "minecraft:enchanted_book",
            1 => {
                *may_place = matches!(&*stack.item, "minecraft:book" | "minecraft:enchanted_book");
                *max_stack = 1;
            }
            _ => {}
        }
    }
}
