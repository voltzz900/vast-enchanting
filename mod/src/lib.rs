//! Vast Enchanting: the Vast Enchanting Table and the shelves around it.
//!
//! The table reads four stats from the blocks around it (the spots a vanilla table reads its bookshelves from):
//! power (how strong the offers get, up to level 100), chaos (how much each offer's strength varies), rarity (rare
//! enchantments favoured, and an extra enchantment at 25 and 75) and clues (how many of each offer's enchantments
//! are shown). It offers enchantments past vanilla's maximum levels, and infuses some items into others. Relic shelves
//! let it offer treasure enchantments, veto shelves keep their books' enchantments out; tomes take an item's
//! enchantments into a book at an anvil; archives keep enchanted books as points. The selection follows Apothic
//! Enchanting's MIT code (Shadows_of_Fire); the names, art and words are this mod's own.

mod archive;
mod data;
mod enchant;
mod nbt;
mod screen;
mod table;
mod tomes;
mod tooltip;

use mod_api::{
    self, AnvilCraftPostEvent, AnvilUpdateEvent, BlockDropsEvent, BlockEntityPart, BlockEntityTickEvent,
    BlockEntityTicksEvent, BlockPart, ClientHost, Flow, Hooks, Host, HostSide, Identifier, ItemKey, ItemPart,
    LevelHost, MenuButtonEvent, MenuContainer, MenuPart, MenuSlotRulesEvent, ModContainer, Registrar,
    RegistrationContext, Screen, ScreenClickEvent, ScreenDrawEvent, ScreenEvent, ScreenMouseScrolledPreEvent,
    ScreenTooltipEvent, ServerHost, TooltipEvent,
};
use std::sync::OnceLock;

/// The items the code compares stacks with, issued at registration.
#[derive(Default, Clone, Copy)]
pub struct Keys {
    pub book: Option<ItemKey>,
    pub enchanted_book: Option<ItemKey>,
    pub lapis: Option<ItemKey>,
    pub tattered: Option<ItemKey>,
    pub bound: Option<ItemKey>,
    pub siphon: Option<ItemKey>,
    pub archive: Option<ItemKey>,
    pub void_archive: Option<ItemKey>,
}

impl Keys {
    /// The tome `item` is, by id.
    pub fn tome(&self, item: ItemKey) -> Option<&'static str> {
        let item = Some(item);
        [(self.tattered, tomes::TATTERED), (self.bound, tomes::BOUND), (self.siphon, tomes::SIPHON)]
            .into_iter()
            .find(|(k, _)| *k == item)
            .map(|(_, id)| id)
    }
}

static KEYS: OnceLock<Keys> = OnceLock::new();

pub fn keys() -> Keys {
    KEYS.get().copied().unwrap_or_default()
}

/// The item with id `id`.
pub fn parse_item(host: &dyn Host, id: &str) -> Option<ItemKey> {
    host.item_key(&Identifier::parse(id).ok()?)
}

/// The table's block, block entity and screen.
struct TablePart;

#[mod_api::horizon_part]
impl BlockPart for TablePart {}

#[mod_api::horizon_part]
impl BlockEntityPart for TablePart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        table::tick(host, event);
    }

    fn menu_button(&self, host: &mut dyn ServerHost, event: &mut MenuButtonEvent<'_>) {
        table::menu_button(host, event);
    }

    fn screen(&self, _host: &dyn Host, event: &mut ScreenEvent<'_>) -> Option<Screen> {
        (event.block == table::TABLE).then(screen::table_screen)
    }

    fn screen_draw(&self, host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        if event.block == table::TABLE {
            screen::table_draw(host, event);
        }
    }

    fn screen_click(&self, _host: &dyn ClientHost, event: &mut ScreenClickEvent<'_>) -> Option<i32> {
        if event.block == table::TABLE { screen::table_click(event) } else { None }
    }

    fn screen_tooltip(&self, host: &dyn ClientHost, event: &mut ScreenTooltipEvent<'_>) {
        if event.block == table::TABLE {
            screen::table_tooltip(host, event);
        }
    }
}

/// An archive's block, block entity and screen; `cap` is the highest level it keeps.
struct ArchivePart {
    cap: u32,
}

#[mod_api::horizon_part]
impl BlockPart for ArchivePart {}

#[mod_api::horizon_part]
impl BlockEntityPart for ArchivePart {
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn block_entity_tick(&self, _host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        archive::tick(event, self.cap);
    }

    fn menu_button(&self, _host: &mut dyn ServerHost, event: &mut MenuButtonEvent<'_>) {
        archive::menu_button(event);
    }

    fn screen(&self, _host: &dyn Host, event: &mut ScreenEvent<'_>) -> Option<Screen> {
        archive::cap_of(event.block).map(|_| screen::archive_screen())
    }

    fn screen_draw(&self, host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        if archive::cap_of(event.block).is_some() {
            screen::archive_draw(host, event);
        }
    }

    fn screen_click(&self, _host: &dyn ClientHost, event: &mut ScreenClickEvent<'_>) -> Option<i32> {
        archive::cap_of(event.block).and_then(|_| screen::archive_click(event))
    }

    fn screen_tooltip(&self, host: &dyn ClientHost, event: &mut ScreenTooltipEvent<'_>) {
        if archive::cap_of(event.block).is_some() {
            screen::archive_tooltip(host, event);
        }
    }
}

/// The veto shelf's block entity: it only holds books, which the table reads.
struct VetoPart;

#[mod_api::horizon_part]
impl BlockPart for VetoPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for VetoPart {}

/// What the table's and the archives' slots take.
struct SlotRules;

#[mod_api::horizon_part]
impl MenuPart for SlotRules {
    fn menu_slot_rules(&self, _host: &dyn Host, event: &mut MenuSlotRulesEvent<'_>) {
        if event.container != MenuContainer::Main {
            return;
        }
        let keys = keys();
        let item = event.stack.item();
        if event.menu == table::MENU {
            // One item to enchant; only lapis beside it.
            match event.slot {
                table::ITEM => event.max_stack = 1,
                table::LAPIS => event.may_place = Some(item) == keys.lapis,
                _ => {}
            }
        } else if event.menu == archive::MENU {
            let (may, max) = archive::slot_rule(event.slot, item, &keys);
            event.may_place = event.may_place && may;
            if let Some(max) = max {
                event.max_stack = event.max_stack.min(max);
            }
        }
    }
}

/// An item's tooltip; `id` is the item's.
struct Tooltip {
    id: String,
}

#[mod_api::horizon_part]
impl ItemPart for Tooltip {
    fn tooltip(&self, _host: &dyn ClientHost, event: &mut TooltipEvent<'_>) {
        tooltip::lines(&self.id, event.stack, event.lines);
    }
}

/// The mod's blocks and items with tooltips: shelves of its own, tomes and archives.
fn tooltip_items() -> Vec<String> {
    let shelves = data::STATS.iter().map(|s| s.0).filter(|id| id.starts_with("vastenchanting:"));
    shelves.chain(tomes::TOMES).chain([archive::ARCHIVE, archive::VOID_ARCHIVE]).map(str::to_owned).collect()
}

/// The container of a block with a menu: `slots` slots, `inputs` reached by hoppers from every side.
fn container(slots: usize, inputs: Vec<usize>, menu: &str) -> ModContainer {
    ModContainer { slots, faces: std::array::from_fn(|_| inputs.clone()), inputs, outputs: Vec::new(), menu: Some(menu.to_owned()) }
}

#[derive(Default)]
struct VastEnchanting;

#[mod_api::horizon_mod]
impl Hooks for VastEnchanting {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        let _ = KEYS.set(Keys {
            book: Some(r.item_key(enchant::BOOK)),
            enchanted_book: Some(r.item_key(enchant::ENCHANTED_BOOK)),
            lapis: Some(r.item_key("minecraft:lapis_lazuli")),
            tattered: Some(r.item_key(tomes::TATTERED)),
            bound: Some(r.item_key(tomes::BOUND)),
            siphon: Some(r.item_key(tomes::SIPHON)),
            archive: Some(r.item_key(archive::ARCHIVE)),
            void_archive: Some(r.item_key(archive::VOID_ARCHIVE)),
        });
        // One block entity type per block, its id the block's (as 0.1 saved them, so old worlds load).
        let name = |id: &'static str| id.strip_prefix("vastenchanting:").unwrap_or(id);
        let table = name(table::TABLE);
        let kind = r.block_entity_type(table).blocks(&[table]).build();
        r.block_entity_part(&kind, TablePart).expect("Vast Enchanting: the table's block entity");
        let block = r.block_key(table::TABLE);
        r.block_container(&block, container(2, Vec::new(), table::MENU));
        for id in [archive::ARCHIVE, archive::VOID_ARCHIVE] {
            let n = name(id);
            let kind = r.block_entity_type(n).blocks(&[n]).build();
            let cap = archive::cap_of(id).unwrap_or(16);
            r.block_entity_part(&kind, ArchivePart { cap }).expect("Vast Enchanting: an archive's block entity");
            // Hoppers may feed books in from any side; nothing comes out by them.
            let block = r.block_key(id);
            r.block_container(&block, container(2, vec![archive::BOOK_IN], archive::MENU));
        }
        // The veto shelf holds its books in a hopper's menu.
        let veto = name(enchant::VETO_SHELF);
        let kind = r.block_entity_type(veto).blocks(&[veto]).build();
        r.block_entity_part(&kind, VetoPart).expect("Vast Enchanting: the veto shelf's block entity");
        let block = r.block_key(enchant::VETO_SHELF);
        r.block_container(&block, container(table::VETO_SLOTS, Vec::new(), "minecraft:hopper"));
        r.menu_part(table::MENU, SlotRules);
        r.menu_part(archive::MENU, SlotRules);
        for id in tooltip_items() {
            let item = r.item_key(&id);
            r.item_part(item, Tooltip { id });
        }
    }

    fn anvil_update(&self, host: HostSide<'_>, event: &mut AnvilUpdateEvent<'_>) -> Flow {
        tomes::anvil_update(host, event);
        Flow::Continue
    }

    fn anvil_craft_post(&self, host: &mut dyn ServerHost, event: &mut AnvilCraftPostEvent<'_>) {
        tomes::anvil_craft_post(host, event);
    }

    fn block_drops(&self, level: &dyn LevelHost, event: &mut BlockDropsEvent<'_>) -> Flow {
        let Some(block) = level.state_block(event.state) else { return Flow::Continue };
        let key = match block {
            archive::ARCHIVE => keys().archive,
            archive::VOID_ARCHIVE => keys().void_archive,
            _ => return Flow::Continue,
        };
        let nbt = event.block_entity.and_then(|e| level.block_entity_data(e));
        archive::on_drops(block, nbt, event.drops, key);
        Flow::Continue
    }

    fn screen_mouse_scrolled_pre(&self, _host: &dyn ClientHost, event: &mut ScreenMouseScrolledPreEvent<'_>) -> Flow {
        screen::wheel(event)
    }
}
