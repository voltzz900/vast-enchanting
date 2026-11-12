fn vastenchanting_be_tick(game: &mut crate::game::Game<'_>, pos: [i32; 3], entity: &mut horizon_client::containers::ModBlockEntity) {
    crate::game::vastenchanting_client::block_entity_tick(game, pos, entity);
}
