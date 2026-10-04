# CYBERGRID theme engine

`cybercore::theme` provides the runtime contract shared by Cybercore apps.
The existing embedded palettes remain the built-in source of truth; the
engine adds portable custom themes and persists a user's selected theme and
appearance without requiring a rebuild.

## Theme document format

Version 1 preserves all eleven established CYBERGRID color roles. The
`palette` is the default/dark appearance. `variants.light` is optional; when
absent, the base palette is used for light appearance as a compatibility
fallback. `design` controls fonts, density, corners, and motion.

```json
{
  "format_version": 1,
  "metadata": {
    "id": "midnight-terminal",
    "name": "Midnight Terminal",
    "family": "custom",
    "description": "A low-glare terminal palette",
    "author": "Local user"
  },
  "palette": {
    "bg": "0e091d", "white": "ffffff", "acid_green": "c8e967",
    "hot_pink": "fd3e6a", "purple": "9147a8", "cyan": "14b9b5",
    "orange": "ff7f41", "red": "f93d3b", "panel": "171329",
    "line": "332a4a", "muted": "a39bb8"
  },
  "variants": {
    "light": {
      "bg": "f5f3fa", "white": "171329", "acid_green": "527a00",
      "hot_pink": "bf2454", "purple": "704080", "cyan": "087c79",
      "orange": "a84d1c", "red": "b32626", "panel": "ffffff",
      "line": "d5d0df", "muted": "625b70"
    }
  },
  "design": {
    "font_ui": "Inter", "font_mono": "JetBrains Mono",
    "font_display": "Oxanium", "density": "comfortable",
    "corners": "sharp", "motion": "full"
  }
}
```

IDs are lowercase letters, digits, and hyphens (1–64 characters). Palette
colors are six-digit hex values with or without `#`. Font family strings are
restricted to letters, digits, spaces, underscores, and hyphens. Density is
`compact`, `comfortable`, or `spacious`; corners is `sharp`, `soft`, or
`rounded`; motion is `full`, `reduced`, or `none`. Unsupported document
versions and invalid fields are rejected.

Old flat JSON palette files (without `format_version`, `metadata`, or
`palette`) remain readable through `ThemeDocument::from_json`; callers pass
the ID to assign to that imported palette. The Hub creator imports either
shape and exports versioned documents.

## Rust API

```rust,no_run
use cybercore::theme::{Appearance, ThemeCatalog, ThemeDocument};

let mut catalog = ThemeCatalog::load()?;
let active = catalog.get(catalog.active_id()).expect("active theme");
let css = active.document.to_css(catalog.active_appearance());

let document = ThemeDocument::from_json(&std::fs::read_to_string("theme.json")?, "my-theme")?;
document.validate()?;
let contrast = document.contrast_report(Appearance::Dark);
catalog.save(document)?;
catalog.select("my-theme")?;
catalog.set_appearance(Appearance::Light)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

`ThemeDocument::to_css` emits the established CYBERGRID variables and shared
design variables. `contrast_report` reports WCAG contrast ratios for common
text and accent/surface pairs; it is feedback for authors, not a guarantee
that every component or state meets accessibility requirements.

## Shared storage and precedence

Custom theme documents live in `themes/<id>.json`, and the active selection
in `active-theme.json`. The config root is chosen in this order:

1. `CYBERCORE_CONFIG_DIR`, if set (the directory itself is the config root).
2. `$XDG_CONFIG_HOME/cybercore`.
3. `$HOME/.config/cybercore`.

`CYBERGRID_THEME` can override the stored active theme for a process when it
names a theme in the catalog. Otherwise, the stored selection wins, followed
by the schema's built-in active theme. Theme and selection writes use a
temporary file and rename to avoid exposing partially written JSON.

Built-in theme family names are embedded alongside the palette catalog, so
consumers do not need access to the Cybercore source checkout to group themes.
Custom themes may use the `family` field for organization in creator UIs.

## Hub integration

Cyberdeck Hub exposes `GET /api/cybergrid/themes`, `POST /api/cybergrid/themes`,
`DELETE /api/cybergrid/themes/:id`, `POST /api/cybergrid/active/:id`,
`POST /api/cybergrid/appearance/:mode`, `POST /api/cybergrid/validate`, and
`GET /api/cybergrid/css/:name?appearance=dark|light`. The creator saves to
the shared config catalog, so themes are available to other Cybercore apps
running as the same user.

## Current integration scope

The crate owns the document format, validation, catalog, selection, contrast
report, and CSS generation. Cyberdeck Hub currently provides the visual
creator and import/export flow. Cyberdesk consumes the shared catalog and
custom themes. Other applications can adopt the same engine incrementally;
older palette consumers continue to work with the original roles.
