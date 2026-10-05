# Cybercore design-system components

`cybercore::tokens::CSS` provides shared typography, spacing, radii, and
motion. `cybercore::components::CSS` provides reusable component styles that
read CYBERGRID semantic colors. Load them in this order:

```html
<link rel="stylesheet" href="/vendor/tokens.css">
<style id="theme-vars">:root { --bg:#100d19; --fg:#f3efff; /* theme CSS */ }</style>
<link rel="stylesheet" href="/vendor/components.css">
```

Rust handlers can serve the same embedded assets without copying the files:

```rust
let tokens = cybercore::tokens::CSS;
let components = cybercore::components::CSS;
let active_theme = catalog
    .get(catalog.active_id())
    .expect("active theme")
    .document
    .to_css(catalog.active_appearance());
```

The public component classes include `.window` and `.titlebar` for framed
panels, `.theme-dropdown` and `.theme-item` for theme selection, `.res-table`
for data, `.text-input` for controls, `.empty-state` for no-result messages,
`.markdown-body` for rendered notes, and `.viewer-overlay` / `.viewer-window`
for modal viewers. Page-shell layout remains the consuming app's
responsibility; the shared stylesheet intentionally does not style bare
`main` elements. The `[hidden]` state always hides components, including
overlays that use flex layout.

The crate's quality tests check that every embedded theme validates, emits all
eleven color roles in both appearance modes, and preserves the shared
component selectors. Consumers can use these assets directly or adapt the
semantic CSS variables to another UI toolkit.
