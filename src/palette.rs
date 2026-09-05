//! True-color ANSI helpers, sourced from the canonical schema in
//! `schema/cybergrid.json` rather than hand-copied hex values per
//! project (which is how omniscient and cyberplug each ended up with
//! their own slightly-drifting copy of the same palette before this
//! crate existed).

use crate::schema;

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

fn rgb(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    format!("\x1b[38;2;{r};{g};{b}m")
}

pub fn bg() -> String {
    rgb(&schema::load().palette.bg)
}
pub fn acid_green() -> String {
    rgb(&schema::load().palette.acid_green)
}
pub fn hot_pink() -> String {
    rgb(&schema::load().palette.hot_pink)
}
pub fn purple() -> String {
    rgb(&schema::load().palette.purple)
}
pub fn cyan() -> String {
    rgb(&schema::load().palette.cyan)
}
pub fn orange() -> String {
    rgb(&schema::load().palette.orange)
}
pub fn red() -> String {
    rgb(&schema::load().palette.red)
}
pub fn white() -> String {
    rgb(&schema::load().palette.white)
}

/// The raw hex string (no `#`, no ANSI wrapping) — useful for anything
/// that wants the color value itself rather than a terminal escape,
/// e.g. generating a GTK CSS file or a QML color binding.
pub fn hex(name: &str) -> Option<String> {
    let p = &schema::load().palette;
    Some(
        match name {
            "bg" => &p.bg,
            "acid_green" => &p.acid_green,
            "hot_pink" => &p.hot_pink,
            "purple" => &p.purple,
            "cyan" => &p.cyan,
            "orange" => &p.orange,
            "red" => &p.red,
            "white" => &p.white,
            _ => return None,
        }
        .clone(),
    )
}
