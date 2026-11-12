fn vastenchanting_be(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::vastenchanting_client::block_entity(block, entity);
}
