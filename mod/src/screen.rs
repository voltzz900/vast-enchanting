//! The table's and the archives' screens, drawn from their menus' data slots.

use crate::archive::{self, affordable, points_for};
use crate::data::{ENCHANTMENTS, INFUSIONS};
use crate::enchant::{self, D_BLOCKED, D_CHAOS, D_CLUES, D_COST, D_INFUSE, D_POWER, D_RARITY, D_TOTAL, D_TREASURE, DATA_SLOTS, MAX_CLUES};
use mod_api::{
    ClientHost, DrawContext, Flow, Host, Screen, ScreenClickEvent, ScreenDrawEvent, ScreenMouseScrolledPreEvent,
    ScreenTooltipEvent,
};
use std::sync::atomic::{AtomicUsize, Ordering};

const OPAQUE: u32 = 0xFF00_0000;

/// The selected language's text for `key`, else `english`.
pub fn lang(host: &dyn ClientHost, key: &str, english: &str) -> String {
    host.translate(key).unwrap_or_else(|| english.to_owned())
}

fn roman(level: u32) -> String {
    const R: [&str; 10] = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];
    R.get(level as usize - 1).map_or_else(|| level.to_string(), |r| (*r).to_owned())
}

/// `Enchantment.getFullname`: the enchantment's name and its level (none for a one-level one at 1), and whether it
/// is a curse.
pub fn ench_name(host: &dyn ClientHost, index: usize, level: u32) -> (String, bool) {
    let Some(e) = ENCHANTMENTS.get(index) else { return (String::new(), false) };
    let path = e.id.split_once(':').map_or(e.id, |(_, p)| p);
    let fallback: String = path
        .split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut name = lang(host, &format!("enchantment.{}", e.id.replacen(':', ".", 1)), &fallback);
    if level != 1 || e.max_level != 1 {
        name.push(' ');
        name.push_str(&host.translate(&format!("enchantment.level.{level}")).unwrap_or_else(|| roman(level.max(1))));
    }
    (name, e.curse)
}

/// The local player's experience level (high when unknown, so nothing reads as out of reach).
fn local_levels(host: &dyn ClientHost) -> i32 {
    let h: &dyn Host = host;
    host.local_player().and_then(|p| p.experience(h)).map_or(i32::MAX, |e| e.level)
}

/// Where the mouse is, from the screen's top left.
fn mouse(host: &dyn ClientHost, origin: [f32; 2]) -> [f32; 2] {
    let [x, y] = host.cursor();
    [x - origin[0], y - origin[1]]
}

fn inside(r: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= r[0] && at[0] < r[0] + r[2] && at[1] >= r[1] && at[1] < r[1] + r[3]
}

/// `text`, cut with an ellipsis to `width`.
fn fit(draw: &dyn DrawContext, text: &str, width: f32) -> String {
    if draw.text_width(text) <= width {
        return text.to_owned();
    }
    let mut s: String = text.to_owned();
    while !s.is_empty() && draw.text_width(&format!("{s}…")) > width {
        s.pop();
    }
    format!("{s}…")
}

/// An item's name by id: its block's name, else its item's.
fn item_name(host: &dyn ClientHost, id: &str) -> String {
    let (ns, path) = id.split_once(':').unwrap_or(("minecraft", id));
    host.translate(&format!("block.{ns}.{path}"))
        .or_else(|| host.translate(&format!("item.{ns}.{path}")))
        .unwrap_or_else(|| path.to_owned())
}

fn infusion_name(host: &dyn ClientHost, i: usize) -> String {
    INFUSIONS.get(i).map_or_else(String::new, |inf| item_name(host, inf.1))
}

// ---- the table -----------------------------------------------------------------------------------------------------

const GUI: &str = "vastenchanting:gui/table";
const OFFER: [f32; 4] = [60.0, 14.0, 108.0, 19.0];
const BARS_X: f32 = 52.0;
/// Where the special shelves' marks go: the free corner above the item and lapis slots.
const MARKS: [f32; 4] = [12.0, 24.0, 42.0, 12.0];
const BAR_W: f32 = 92.0;

pub fn table_screen() -> Screen {
    Screen {
        width: 176.0,
        height: 194.0,
        slots: vec![Some([15.0, 47.0]), Some([35.0, 47.0])],
        inventory_y: 112.0,
        inventory_x: 8.0,
        own_labels: true,
        own_background: true,
        ..Screen::default()
    }
}

fn offer_rect(i: usize) -> [f32; 4] {
    [OFFER[0], OFFER[1] + 19.0 * i as f32, OFFER[2], OFFER[3]]
}

pub fn table_draw(host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
    let [ox, oy] = event.origin;
    let data = event.data;
    let draw = &mut *event.draw;
    draw.sprite(GUI, [0.0, 0.0, 176.0, 194.0], [ox, oy, 176.0, 194.0], None);
    draw.text(&lang(host, "container.vastenchanting.table", "Vast Enchanting Table"), [ox + 8.0, oy + 5.0], OPAQUE | 0x404040, false);
    if data.len() < DATA_SLOTS {
        return;
    }
    let at = mouse(host, event.origin);
    let levels = local_levels(host);
    for i in 0..3 {
        let r = offer_rect(i);
        let cost = data[D_COST + i];
        let off = cost <= 0;
        let v = if off { 233.0 } else if inside(r, at) { 214.0 } else { 195.0 };
        draw.sprite(GUI, [0.0, v, 108.0, 19.0], [ox + r[0], oy + r[1], 108.0, 19.0], None);
        if off {
            continue;
        }
        // The badge: lapis it takes (1 to 3); the level it needs on the right.
        draw.text(&(i + 1).to_string(), [ox + r[0] + 7.0, oy + r[1] + 6.0], OPAQUE | 0x9AD6FF, true);
        let level = cost.to_string();
        let colour = if levels >= cost { 0x80FF20 } else { 0xFF6A6A };
        let lw = draw.text_width(&level);
        draw.text(&level, [ox + r[0] + r[2] - lw - 4.0, oy + r[1] + 6.0], OPAQUE | colour, true);
        // An infusion offer shows its result's name in pink; the tooltip says "Infuse into".
        let (text, tint) = if i == 2 && data[D_INFUSE] > 0 {
            (infusion_name(host, data[D_INFUSE] as usize - 1), 0xFF9CFF)
        } else {
            let first = enchant::clue(data, i, 0).map(|(index, level)| ench_name(host, index, level).0);
            (first.unwrap_or_else(|| lang(host, "vastenchanting.unknown", "Unknown enchantments")), 0xE6DCF0)
        };
        let room = r[2] - 18.0 - lw - 5.0;
        let text = fit(draw, &text, room);
        draw.text(&text, [ox + r[0] + 17.0, oy + r[1] + 6.0], OPAQUE | tint, true);
    }
    // The stats.
    let rows = [
        ("vastenchanting.power", "Power", D_POWER),
        ("vastenchanting.chaos", "Chaos", D_CHAOS),
        ("vastenchanting.rarity", "Rarity", D_RARITY),
    ];
    for (k, (key, english, d)) in rows.iter().enumerate() {
        let y = oy + 77.0 + k as f32 * 10.0;
        draw.text(&lang(host, key, english), [ox + 10.0, y], OPAQUE | 0x3A2A4A, false);
        let value = data[*d] as f32 / 10.0;
        draw.sprite(GUI, [110.0, 195.0, BAR_W, 6.0], [ox + BARS_X, y + 1.0, BAR_W, 6.0], None);
        let w = (value / 100.0).clamp(0.0, 1.0) * BAR_W;
        draw.sprite(GUI, [110.0, 201.0 + k as f32 * 6.0, w, 6.0], [ox + BARS_X, y + 1.0, w, 6.0], None);
        let shown = format!("{value:.0}");
        let sw = draw.text_width(&shown);
        draw.text(&shown, [ox + 167.0 - sw, y], OPAQUE | 0x3A2A4A, false);
    }
    let clues = lang(host, "vastenchanting.clues", "Clues: %s").replacen("%s", &data[D_CLUES].to_string(), 1);
    let cw = draw.text_width(&clues);
    draw.text(&clues, [ox + 168.0 - cw, oy + 5.0], OPAQUE | 0x404040, false);
    // Marks for the special shelves, above the slots: a gold star for treasure, a red count for vetoes.
    let mut x = ox + MARKS[0] + 2.0;
    if data[D_TREASURE] > 0 {
        draw.text("✦", [x, oy + MARKS[1] + 2.0], OPAQUE | 0xC88A00, false);
        x += draw.text_width("✦") + 4.0;
    }
    if data[D_BLOCKED] > 0 {
        draw.text(&format!("×{}", data[D_BLOCKED]), [x, oy + MARKS[1] + 2.0], OPAQUE | 0xB02020, false);
    }
}

pub fn table_click(event: &ScreenClickEvent<'_>) -> Option<i32> {
    if event.right || event.data.len() < DATA_SLOTS {
        return None;
    }
    (0..3).find(|i| event.data[D_COST + i] > 0 && inside(offer_rect(*i), event.at)).map(|i| i as i32)
}

pub fn table_tooltip(host: &dyn ClientHost, event: &mut ScreenTooltipEvent<'_>) {
    let (data, at) = (event.data, event.at);
    if data.len() < DATA_SLOTS {
        return;
    }
    let lines = &mut *event.lines;
    for i in 0..3 {
        let cost = data[D_COST + i];
        if cost <= 0 || !inside(offer_rect(i), at) {
            continue;
        }
        if i == 2 && data[D_INFUSE] > 0 {
            let name = infusion_name(host, data[D_INFUSE] as usize - 1);
            lines.push(format!("§d{}", lang(host, "vastenchanting.infuse", "Infuse into %s").replacen("%s", &name, 1)));
        } else {
            let mut shown = 0;
            for k in 0..MAX_CLUES {
                if let Some((index, level)) = enchant::clue(data, i, k) {
                    let (name, curse) = ench_name(host, index, level);
                    lines.push(format!("{}{name}", if curse { "§c" } else { "§7" }));
                    shown += 1;
                }
            }
            if shown == 0 {
                lines.push(format!("§7{}", lang(host, "vastenchanting.unknown", "Unknown enchantments")));
            }
            let more = data[D_TOTAL + i] - shown;
            if more > 0 && shown > 0 {
                lines.push(format!("§8{}", lang(host, "vastenchanting.more", "...and %s more").replacen("%s", &more.to_string(), 1)));
            }
        }
        let colour = if local_levels(host) >= cost { "§a" } else { "§c" };
        let cost_line = lang(host, "vastenchanting.levels", "%s levels, %s lapis")
            .replacen("%s", &cost.to_string(), 1)
            .replacen("%s", &(i + 1).to_string(), 1);
        lines.push(format!("{colour}{cost_line}"));
        return;
    }
    if inside([96.0, 3.0, 76.0, 11.0], at) || (inside(MARKS, at) && (data[D_TREASURE] > 0 || data[D_BLOCKED] > 0)) {
        lines.push(format!("§f{}", lang(host, "vastenchanting.clues", "Clues: %s").replacen("%s", &data[D_CLUES].to_string(), 1)));
        lines.push(format!("§7{}", lang(host, "vastenchanting.clues.desc", "How many of each offer's enchantments are shown before you choose.")));
        if data[D_TREASURE] > 0 {
            lines.push(format!("§6{}", lang(host, "vastenchanting.treasure", "Treasure enchantments offered (a relic shelf)")));
        }
        if data[D_BLOCKED] > 0 {
            let line = lang(host, "vastenchanting.blocked", "%s enchantments kept out (veto shelves)");
            lines.push(format!("§c{}", line.replacen("%s", &data[D_BLOCKED].to_string(), 1)));
        }
        return;
    }
    let rows = [
        ("vastenchanting.power", "Power", "How strong the offers get: the top offer costs this many levels."),
        ("vastenchanting.chaos", "Chaos", "How much each offer's strength varies, up or down."),
        ("vastenchanting.rarity", "Rarity", "Favours rare enchantments; at 25 and 75 an offer gains another."),
    ];
    for (k, (key, english, desc)) in rows.iter().enumerate() {
        if inside([8.0, 76.0 + k as f32 * 10.0, 160.0, 10.0], at) {
            lines.push(format!("§f{}", lang(host, key, english)));
            lines.push(format!("§7{}", lang(host, &format!("{key}.desc"), desc)));
        }
    }
}

// ---- the archive ---------------------------------------------------------------------------------------------------

const ARCHIVE_GUI: &str = "vastenchanting:gui/archive";
const LIST: [f32; 4] = [32.0, 14.0, 128.0, 90.0];
const ROW_H: f32 = 15.0;
const ROWS: usize = 6;
const TRACK: [f32; 4] = [163.0, 14.0, 8.0, 92.0];

/// The list's first row shown.
static SCROLL: AtomicUsize = AtomicUsize::new(0);

fn scroll(rows: usize) -> usize {
    let s = SCROLL.load(Ordering::Relaxed).min(rows.saturating_sub(ROWS));
    SCROLL.store(s, Ordering::Relaxed);
    s
}

fn row_rect(k: usize) -> [f32; 4] {
    [LIST[0], LIST[1] + ROW_H * k as f32, LIST[2], ROW_H]
}

pub fn archive_screen() -> Screen {
    Screen {
        width: 176.0,
        height: 194.0,
        slots: vec![Some([8.0, 18.0]), Some([8.0, 80.0])],
        inventory_y: 112.0,
        inventory_x: 8.0,
        own_labels: true,
        own_background: true,
        ..Screen::default()
    }
}

pub fn archive_draw(host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
    let [ox, oy] = event.origin;
    let at = mouse(host, event.origin);
    let (data, block) = (event.data, event.block);
    let draw = &mut *event.draw;
    draw.sprite(ARCHIVE_GUI, [0.0, 0.0, 176.0, 194.0], [ox, oy, 176.0, 194.0], None);
    draw.text(&item_name(host, block), [ox + 8.0, oy + 4.0], OPAQUE | 0x404040, false);
    let rows = archive::rows(data);
    if rows.is_empty() {
        return;
    }
    let first = scroll(rows.len());
    for k in 0..ROWS {
        let Some(&(index, points, level)) = rows.get(first + k) else { break };
        let r = row_rect(k);
        let can = affordable(points, level);
        let v = if can == 0 { 225.0 } else if inside(r, at) { 210.0 } else { 195.0 };
        draw.sprite(ARCHIVE_GUI, [0.0, v, 128.0, 15.0], [ox + r[0], oy + r[1], 128.0, 15.0], None);
        let (name, curse) = ench_name(host, index, can.max(1));
        let shown = points.to_string();
        let sw = draw.text_width(&shown);
        draw.text(&shown, [ox + r[0] + r[2] - sw - 3.0, oy + r[1] + 4.0], OPAQUE | 0xB8A8D8, true);
        let tint = if curse { 0xFF7A7A } else if can == 0 { 0x9A90A8 } else { 0xE6DCF0 };
        let name = fit(draw, &name, r[2] - sw - 10.0);
        draw.text(&name, [ox + r[0] + 3.0, oy + r[1] + 4.0], OPAQUE | tint, true);
    }
    // The scrollbar's thumb.
    if rows.len() > ROWS {
        let span = TRACK[3] - 15.0;
        let y = TRACK[1] + span * first as f32 / (rows.len() - ROWS) as f32;
        draw.sprite(ARCHIVE_GUI, [240.0, 195.0, 8.0, 15.0], [ox + TRACK[0], oy + y, 8.0, 15.0], None);
    }
}

pub fn archive_click(event: &ScreenClickEvent<'_>) -> Option<i32> {
    let rows = archive::rows(event.data);
    if inside(TRACK, event.at) && rows.len() > ROWS {
        // The track: jump the thumb to where it was clicked.
        let t = ((event.at[1] - TRACK[1] - 7.5) / (TRACK[3] - 15.0)).clamp(0.0, 1.0);
        SCROLL.store((t * (rows.len() - ROWS) as f32).round() as usize, Ordering::Relaxed);
        return None;
    }
    let first = scroll(rows.len());
    (0..ROWS).find_map(|k| {
        let &(index, _, _) = rows.get(first + k)?;
        inside(row_rect(k), event.at).then_some(index as i32 * 2 + i32::from(event.right))
    })
}

pub fn archive_tooltip(host: &dyn ClientHost, event: &mut ScreenTooltipEvent<'_>) {
    let (data, at, block) = (event.data, event.at, event.block);
    if data.len() < archive::DATA_SLOTS {
        return;
    }
    let lines = &mut *event.lines;
    let rows = archive::rows(data);
    if rows.is_empty() {
        if inside(LIST, at) {
            lines.push(format!("§7{}", lang(host, "vastenchanting.archive.empty", "Nothing stored yet: put an enchanted book in the top slot.")));
        }
        return;
    }
    let first = scroll(rows.len());
    for k in 0..ROWS {
        let Some(&(index, points, level)) = rows.get(first + k) else { break };
        if !inside(row_rect(k), at) {
            continue;
        }
        let can = affordable(points, level);
        let (name, curse) = ench_name(host, index, can.max(1));
        lines.push(format!("{}{name}", if curse { "§c" } else { "§f" }));
        let stored = if points == 1 {
            lang(host, "vastenchanting.archive.point", "1 point stored")
        } else {
            lang(host, "vastenchanting.archive.points", "%s points stored").replacen("%s", &points.to_string(), 1)
        };
        lines.push(format!("§7{stored}"));
        let cap = archive::cap_of(block).unwrap_or(16);
        let max = lang(host, "vastenchanting.archive.max", "Up to level %s (this archive keeps up to %s)")
            .replacen("%s", &level.to_string(), 1)
            .replacen("%s", &cap.to_string(), 1);
        lines.push(format!("§7{max}"));
        if can < level {
            let next = can + 1;
            let cost = lang(host, "vastenchanting.archive.cost", "Level %s costs %s points from a plain book")
                .replacen("%s", &next.to_string(), 1)
                .replacen("%s", &points_for(next).to_string(), 1);
            lines.push(format!("§8{cost}"));
        }
        lines.push(format!("§d{}", lang(host, "vastenchanting.archive.click", "Left click: one level more on the bottom book")));
        lines.push(format!("§d{}", lang(host, "vastenchanting.archive.click_max", "Right click: as high as the points allow")));
        return;
    }
}

/// `screen_mouse_scrolled_pre`: the wheel scrolls the archive's list.
pub fn wheel(event: &mut ScreenMouseScrolledPreEvent<'_>) -> Flow {
    if event.screen.menu != Some(archive::MENU) || event.scroll[1] == 0.0 {
        return Flow::Continue;
    }
    let s = SCROLL.load(Ordering::Relaxed);
    SCROLL.store(if event.scroll[1] > 0.0 { s.saturating_sub(1) } else { s + 1 }, Ordering::Relaxed);
    Flow::Cancel
}
