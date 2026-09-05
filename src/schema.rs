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

use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub schema_version: u32,
    pub palette: Palette,
    pub paths: Paths,
}

#[derive(Debug, Deserialize)]
pub struct Palette {
    pub bg: String,
    pub acid_green: String,
    pub hot_pink: String,
    pub purple: String,
    pub cyan: String,
    pub orange: String,
    pub red: String,
    pub white: String,
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
pub fn load() -> &'static Schema {
    SCHEMA.get_or_init(|| {
        serde_json::from_str(RAW).expect(
            "schema/cybergrid.json failed to parse — this is a cybercore bug, \
             not a caller error, since the schema is embedded at compile time",
        )
    })
}
