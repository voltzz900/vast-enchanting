fn vastenchanting_anvil_taken(game: &mut crate::game::Game<'_>, player: u128, pos: [i32; 3], craft: &horizon_client::menu::AnvilCraft) {
    crate::game::vastenchanting_client::anvil_taken(game, player, pos, craft);
}
