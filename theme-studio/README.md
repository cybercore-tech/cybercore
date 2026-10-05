# Cybercore Theme Studio

Cybercore Theme Studio is the standalone visual creator for CYBERGRID themes.
It is a small local Rust web app that uses the `cybercore::theme::ThemeCatalog`
directly, so themes it saves are immediately available to Cyberdeck Hub,
Cyberdesk, and other apps using the same Cybercore user configuration.

## Run locally

From the Cybercore repository root:

```bash
cargo run -p cybercore-theme-studio
```

Then open <http://127.0.0.1:8761/>. The server only binds to loopback and
rejects unexpected `Host` headers. For a friendly local hostname, a name under
`.localhost` (for example, `http://cybercore-tech.localhost:8761/`) is accepted
and resolves to loopback without a hosts-file change. Set
`CYBERCORE_THEME_STUDIO_BIND` to use a different loopback address or port.
Cyberdeck Hub has a **Theme Studio** link to the default URL.

The studio provides a searchable theme library, app-reference previews with a
mobile viewport, and an inspector organized around theme identity, semantic
colors, design tokens, and quality checks. App-reference previews cover
Cyberdeck Hub, Dockspace, and Cyberdesk, with grayscale proof and keyboard
focus visibility. These are illustrative app-pattern previews rather than
embedded live app sessions. Undo/redo (Ctrl+Z / Ctrl+Shift+Z) and a
saved/new/unsaved indicator protect the draft workflow. It supports built-in
and custom themes, dark and optional light palettes, typography, density,
corners, and motion tokens. Theme edits update the preview as you work, with
contrast feedback for both appearances. Import and export versioned theme
documents or share complete theme packs.

The **Curated pack library** offers the checked-in Cybercore theme families
as collections. Each preview shows pack version, author, license, engine
compatibility, and palette swatches before installation. **Add collection to my library** copies the selected family
under `curated-<family>` with namespaced IDs; it leaves embedded themes and
the active selection alone. Repeating the action safely skips themes already
copied. The library endpoints are `GET /api/library` and
`POST /api/library/:family/install`.

Themes and active selection use the same storage contract documented in the
[Cybercore theme engine guide](../docs/theme-engine.md):
`$XDG_CONFIG_HOME/cybercore` or `~/.config/cybercore`, with
`CYBERCORE_CONFIG_DIR` available for an explicit config root.
