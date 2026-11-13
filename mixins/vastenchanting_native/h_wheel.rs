fn vastenchanting_wheel(screen: &crate::game::native_hooks::NativeScreenView<'_>, scroll: [f64; 2], canceled: &mut bool) {
    crate::game::vastenchanting_client::archive_wheel(screen.menu, scroll, canceled);
}
