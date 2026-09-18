//! Shared UI component styles for the Cybercore Systems Framework —
//! cards, status badges, data tables, form inputs, popup viewers, and
//! the theme-picker dropdown itself — for every Cybercore surface that
//! wants the same look cyberdeck established (cards/badges/topbar/window
//! conventions), not just cyberdeck itself.
//!
//! Colours and typography come from [`crate::schema`] and
//! [`crate::tokens`]; this module only supplies component shapes/layout.
//! Load order: reset → `tokens::CSS` → theme colours → `components::CSS`.
//!
//! ```ignore
//! let css = cybercore::components::CSS;
//! ```

/// The component stylesheet: cards, badges, tables, the theme-picker
/// dropdown, popup viewers, form inputs. Load after tokens and theme
/// colours.
pub const CSS: &str = include_str!("../css/cybercomponents.css");

/// `text/css` content-type string, handy for HTTP handlers.
pub const CSS_CONTENT_TYPE: &str = "text/css; charset=utf-8";
