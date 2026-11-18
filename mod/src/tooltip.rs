//! What the mod's items say in their tooltips: a shelf what it gives a table, a tome and an archive what they do.

use crate::archive;
use crate::enchant::{self, RELIC_SHELF, VETO_SHELF};
use crate::tomes::TOMES;
use mod_api::{ItemStack, Text};

fn line(key: &str, args: Vec<Text>, colour: &str) -> (Text, u32) {
    let mut text = Text::translate(key, args);
    text.style.color = Some(colour.to_owned());
    (text, u32::MAX)
}

fn lit(s: String) -> Text {
    Text::literal(s)
}

/// A number as the tooltip shows it: no ".0".
fn num(v: f32) -> String {
    if v.fract() == 0.0 { format!("{v:.0}") } else { format!("{v}") }
}

fn signed(v: f32) -> String {
    if v >= 0.0 { format!("+{}", num(v)) } else { num(v) }
}

/// The tooltip lines of item `id` (`stack`).
pub fn lines(id: &str, stack: &ItemStack, out: &mut Vec<(Text, u32)>) {
    let desc = |kind: &str| format!("{kind}.{}.desc", id.replacen(':', ".", 1));
    if archive::cap_of(id).is_some() {
        out.push(line(&desc("block"), Vec::new(), "gray"));
        let held = archive::held_count(stack);
        if held > 0 {
            out.push(line("vastenchanting.archive.holds", vec![lit(held.to_string())], "light_purple"));
        }
        return;
    }
    if TOMES.contains(&id) {
        out.push(line(&desc("item"), Vec::new(), "gray"));
        return;
    }
    let Some((max, power, chaos, rarity, clues)) = enchant::shelf_stats(id) else { return };
    if matches!(id, RELIC_SHELF | VETO_SHELF) {
        out.push(line(&desc("block"), Vec::new(), "gold"));
    }
    if power != 0.0 || chaos != 0.0 || rarity != 0.0 {
        let args = vec![lit(num(power)), lit(num(max)), lit(signed(chaos)), lit(signed(rarity))];
        out.push(line("vastenchanting.shelf.stats", args, "gray"));
    }
    if clues != 0 {
        out.push(line("vastenchanting.shelf.clues", vec![lit(signed(clues as f32))], "gray"));
    }
}
