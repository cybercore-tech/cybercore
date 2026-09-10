//! cybercore — the shared definition repository for the Cybercore
//! Systems Framework. The single source of truth for the CYBERGRID
//! palette, canonical filesystem paths, and (over time) other schema
//! shared across cyberdeck, cyberdock, cyberterm, cyberplug,
//! omniscient, and whatever comes next.
//!
//! The actual data lives in `schema/cybergrid.json`, not in this
//! source code — this crate is a typed, compile-time-embedded loader
//! for that JSON, not the data's owner. Edit the JSON; everything
//! else follows.

pub mod palette;
pub mod paths;
pub mod schema;
pub mod tokens;
