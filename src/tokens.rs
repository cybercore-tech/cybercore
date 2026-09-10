//! Shared **non-colour** design tokens — fonts, a type scale, spacing,
//! radii, motion, z-index — for the Cybercore Systems Framework.
//!
//! Colours are handled separately by [`crate::schema`] /
//! `schema/cybergrid.json`. This module just hands you the ready-made
//! token stylesheet (and a JSON mirror) so every Cybercore surface —
//! cyberdesk, cyberdeck, cyberterm, a marketing site — shares one
//! typographic system.
//!
//! ```ignore
//! // serve it, or inline it into your <head>
//! let css = cybercore::tokens::CSS;
//! ```

/// The token stylesheet: a single `:root { … }` block. Load it after your
/// CSS reset and before the theme colours and component styles.
pub const CSS: &str = include_str!("../css/cybertokens.css");

/// The same tokens as structured JSON, for non-CSS consumers.
pub const JSON: &str = include_str!("../schema/tokens.json");

/// `text/css` content-type string, handy for HTTP handlers.
pub const CSS_CONTENT_TYPE: &str = "text/css; charset=utf-8";
