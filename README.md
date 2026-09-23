<p align="center">
  <img src="assets/header-stacked.svg" width="820" alt="Cybercore — Single source of truth for schemas, palette, and paths">
</p>

<p align="center">
  <a href="https://darkstardevx.github.io/cybercore/">Live theme gallery & docs →</a>
</p>

[![crates.io](https://img.shields.io/crates/v/cybercore.svg)](https://crates.io/crates/cybercore)
[![downloads](https://img.shields.io/crates/d/cybercore.svg)](https://crates.io/crates/cybercore)
[![docs.rs](https://img.shields.io/docsrs/cybercore)](https://docs.rs/cybercore)

The master definition repository and underpinning system engine of the
Cybercore Systems Framework. The single source of truth for global
configuration schemas, the CYBERGRID color palette, and canonical
filesystem paths shared across every downstream project — cyberdeck,
cyberdock, cyberterm, cyberplug, omniscient, and whatever comes next.

## The actual source of truth

**`schema/cybergrid.json`** — not any Rust source file. This is
deliberate: JSON is readable by every language in the framework, not
just Rust. cyberdock's C++/QML can load it directly at runtime with
Qt's `QJsonDocument`; Rust projects get it embedded at compile time
(below) for zero runtime cost and zero drift risk.

Everything else in this repo is a typed loader for that one file, not
an owner of the data. **Edit the JSON. Never hand-edit palette values
or paths anywhere else.**

## Design tokens (fonts, sizing, spacing) — since v0.3.0

Colours are `cybergrid.json`. **Everything else about the feel** — font
families, a type scale, spacing, radii, motion, z-index — lives in
**`css/cybertokens.css`** (with a `schema/tokens.json` mirror for
non-CSS consumers).

```rust
// a ready-to-serve :root {} stylesheet
let css = cybercore::tokens::CSS;
```

Load order in any project: `reset → cybertokens.css → theme colours →
components`. Every Cybercore surface pulls the same `--font-mono`,
`--fs-*`, `--space-*`, `--radius-*` so a new site or tool matches the
rest without copy-paste.

## For Rust projects

Add as a dependency:

```toml
[dependencies]
cybercore = { git = "https://github.com/darkstardevx/cybercore.git", tag = "v0.1.0" }
```

(or `path = "../cybercore"` for local development against an
unpublished change)

Then:

```rust
use cybercore::palette;
use cybercore::paths;

println!("{}Hello{}", palette::acid_green(), palette::RESET);
let sysops = paths::sysops_root(); // ~/.sysops, expanded
```

The JSON is embedded into your binary via `include_str!` at compile
time — no runtime file dependency, and the exact schema version your
binary was built against travels with it.

## For C++/QML projects (cyberdock)

Load `schema/cybergrid.json` directly:

```cpp
QFile file(":/cybercore/schema/cybergrid.json");
file.open(QIODevice::ReadOnly);
QJsonDocument doc = QJsonDocument::fromJson(file.readAll());
QString acidGreen = doc["palette"]["acid_green"].toString();
```

Or in QML, since JSON is just JS object syntax:

```qml
property var cybergrid: JSON.parse(cybergrid_json_string)
color: "#" + cybergrid.palette.acid_green
```

(Wiring the actual file into a Qt resource bundle or reading it from a
known filesystem path is left to cyberdock's own build — this repo
just guarantees the JSON's shape and location.)

## Schema

```json
{
  "schema_version": 1,
  "palette": {
    "bg": "0e091d",
    "acid_green": "c8e967",
    "hot_pink": "FD3E6A",
    "purple": "9147a8",
    "cyan": "14B9B5",
    "orange": "FF7F41",
    "red": "f93d3b",
    "white": "ffffff"
  },
  "paths": {
    "sysops_root": "~/.sysops",
    "cargo_target_root": "~/.cargo-target",
    "arch_sys_root": "~/.arch-sys",
    "omarchy_config_root": "~/.config/omarchy"
  }
}
```

Hex values have no leading `#` (each consumer prepends what it needs —
a `#` for CSS/QML, nothing for raw RGB parsing). Paths use `~/` for
home-relative locations; `cybercore::paths` expands this for Rust
consumers.

## Versioning

`schema_version` bumps on any breaking change to the JSON's shape
(renaming or removing a field). Adding a new field is non-breaking —
consumers using an older `cybercore` crate version simply won't see
the new field yet, nothing errors.

Tag releases (`v0.1.0`, `v0.2.0`, ...) so downstream `Cargo.toml`
dependencies can pin to a known-good version rather than tracking
`main` directly.

## Layout

    schema/cybergrid.json   — the actual source of truth
    src/lib.rs               — module declarations
    src/schema.rs            — embeds + parses the JSON once per process
    src/palette.rs           — true-color ANSI helpers + raw hex accessor
    src/paths.rs              — canonical filesystem locations, ~-expanded

## Adding to the schema

1. Add the field to `schema/cybergrid.json`
2. Add the matching field to the relevant struct in `src/schema.rs`
3. Expose it through `src/palette.rs` or `src/paths.rs` (or a new
   module, if it's a new category of shared data)
4. Every Rust project depending on this crate picks it up on their
   next `cargo update` + rebuild. Non-Rust consumers see it the next
   time they read the JSON.

## License

MIT
