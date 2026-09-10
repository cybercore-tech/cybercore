//! Loads `schema/cybergrid.json` — the canonical, language-agnostic
//! source of truth for the whole Cybercore Systems Framework.
//!
//! The JSON is embedded into the compiled binary at build time via
//! `include_str!`, so every Rust project depending on this crate gets
//! the same data with zero runtime file dependency and zero risk of
//! drift between what was compiled and what's on disk. Non-Rust
//! consumers (e.g. cyberdock's C++/QML) read the same file directly
//! at runtime — this file is the one and only place any of it is
//! ever hand-edited.
//!
//! Schema v2 carries a **map of named themes** plus an `active`
//! pointer. `load().palette` resolves to the active theme (overridable
//! per process with the `CYBERGRID_THEME` env var), so older callers
//! that just read `schema::load().palette.bg` keep working. New
//! callers can pick any theme by name with [`Schema::theme`].

use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// One theme: 11 semantic colour roles, each a bare 6-digit hex string
/// (no leading `#`).
#[derive(Debug, Deserialize, Clone, Default)]
pub struct Palette {
    pub bg: String,
    pub white: String,
    pub acid_green: String,
    pub hot_pink: String,
    pub purple: String,
    pub cyan: String,
    pub orange: String,
    pub red: String,
    /// raised surfaces (cards, headers)
    pub panel: String,
    /// hairline borders / dividers
    pub line: String,
    /// dim / secondary text
    pub muted: String,
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub schema_version: u32,
    /// name of the theme used when a caller doesn't ask for one
    pub active: String,
    /// every theme, keyed by slug
    pub themes: BTreeMap<String, Palette>,
    pub paths: Paths,

    /// The resolved active theme — filled in by [`load`], not present in
    /// the JSON. Kept as a field (not a method) so `schema::load().palette`
    /// keeps compiling for pre-v2 callers.
    #[serde(skip)]
    pub palette: Palette,
}

impl Schema {
    /// A theme by slug, or `None` if it isn't defined.
    pub fn theme(&self, name: &str) -> Option<&Palette> {
        self.themes.get(name)
    }

    /// The active theme (same value as `self.palette`).
    pub fn active_theme(&self) -> &Palette {
        self.themes
            .get(&self.active)
            .or_else(|| self.themes.values().next())
            .expect("cybergrid.json defines no themes")
    }

    /// All theme slugs, sorted.
    pub fn theme_names(&self) -> impl Iterator<Item = &str> {
        self.themes.keys().map(String::as_str)
    }
}

#[derive(Debug, Deserialize)]
pub struct Paths {
    pub sysops_root: String,
    pub cargo_target_root: String,
    pub arch_sys_root: String,
    pub omarchy_config_root: String,
}

static SCHEMA: OnceLock<Schema> = OnceLock::new();

const RAW: &str = include_str!("../schema/cybergrid.json");

/// The parsed schema, loaded once per process.
///
/// If `CYBERGRID_THEME` is set to a known theme slug, that becomes the
/// active theme for this process (handy when several Cybercore apps
/// share the schema but want different looks).
pub fn load() -> &'static Schema {
    SCHEMA.get_or_init(|| {
        let mut s: Schema = serde_json::from_str(RAW).expect(
            "schema/cybergrid.json failed to parse — this is a cybercore bug, \
             not a caller error, since the schema is embedded at compile time",
        );
        if let Ok(name) = std::env::var("CYBERGRID_THEME") {
            if !name.is_empty() && s.themes.contains_key(&name) {
                s.active = name;
            }
        }
        s.palette = s
            .themes
            .get(&s.active)
            .or_else(|| s.themes.values().next())
            .cloned()
            .expect("cybergrid.json defines no themes");
        s
    })
}
