//! Canonical filesystem locations, sourced from the schema. Every
//! project that currently hardcodes `~/.sysops`, `~/.cargo-target`,
//! or similar should call these instead — one edit to the schema
//! updates every consuming project on its next build.

use crate::schema;
use std::path::PathBuf;

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        home.join(rest)
    } else {
        PathBuf::from(path)
    }
}

pub fn sysops_root() -> PathBuf {
    expand_home(&schema::load().paths.sysops_root)
}

pub fn cargo_target_root() -> PathBuf {
    expand_home(&schema::load().paths.cargo_target_root)
}

pub fn arch_sys_root() -> PathBuf {
    expand_home(&schema::load().paths.arch_sys_root)
}

pub fn omarchy_config_root() -> PathBuf {
    expand_home(&schema::load().paths.omarchy_config_root)
}
