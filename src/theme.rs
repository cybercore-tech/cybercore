//! The Cybercore theme engine: versioned theme documents, validation,
//! contrast reports, built-in and user theme catalogs, selection, and CSS.
//!
//! Existing palette files remain the built-in source of truth. Custom
//! themes use [`ThemeDocument`] and live under the user's Cybercore config
//! directory, so consumers can load them without rebuilding their binary.

use crate::schema::{self, Palette};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const FORMAT_VERSION: u32 = 1;
const BUILTIN_FAMILY: &str = "default";

/// A saved theme, portable as JSON and independent of a specific app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeDocument {
    pub format_version: u32,
    pub metadata: ThemeMetadata,
    /// The established CYBERGRID color roles. New design properties are
    /// additive so older Cybercore consumers can still use this palette.
    pub palette: Palette,
    #[serde(default)]
    pub variants: BTreeMap<Appearance, Palette>,
    #[serde(default)]
    pub design: DesignTokens,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeMetadata {
    /// Stable, lowercase identifier used by apps and filenames.
    pub id: String,
    pub name: String,
    #[serde(default = "default_family")]
    pub family: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    Dark,
    Light,
}

impl Appearance {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

impl Default for Appearance {
    fn default() -> Self {
        Self::Dark
    }
}

/// Shared typography, density, corners, and motion settings. These are
/// deliberately enums/allowlisted values so imported themes cannot inject
/// arbitrary CSS into an application's page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DesignTokens {
    pub font_ui: String,
    pub font_mono: String,
    pub font_display: String,
    pub density: Density,
    pub corners: Corners,
    pub motion: Motion,
}

impl Default for DesignTokens {
    fn default() -> Self {
        Self {
            font_ui: "Inter".into(),
            font_mono: "JetBrains Mono".into(),
            font_display: "Oxanium".into(),
            density: Density::Comfortable,
            corners: Corners::Sharp,
            motion: Motion::Full,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    Compact,
    Comfortable,
    Spacious,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Corners {
    Sharp,
    Soft,
    Rounded,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Motion {
    Full,
    Reduced,
    None,
}

impl ThemeDocument {
    pub fn new(id: impl Into<String>, name: impl Into<String>, palette: Palette) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            metadata: ThemeMetadata {
                id: id.into(),
                name: name.into(),
                family: default_family(),
                description: String::new(),
                author: String::new(),
            },
            palette,
            variants: BTreeMap::new(),
            design: DesignTokens::default(),
        }
    }

    /// Read a current document or a legacy flat 11-color Cybercore palette.
    /// Legacy import needs an ID because the old file shape had none.
    pub fn from_json(input: &str, legacy_id: &str) -> Result<Self, ThemeError> {
        if let Ok(document) = serde_json::from_str::<Self>(input) {
            document.validate()?;
            return Ok(document);
        }
        let palette = serde_json::from_str::<Palette>(input)
            .map_err(|error| ThemeError::InvalidDocument(error.to_string()))?;
        let mut document = Self::new(legacy_id, humanize(legacy_id), palette);
        document.metadata.family = "imported".into();
        document.validate()?;
        Ok(document)
    }

    pub fn validate(&self) -> Result<(), ThemeError> {
        if self.format_version != FORMAT_VERSION {
            return Err(ThemeError::UnsupportedVersion(self.format_version));
        }
        validate_id(&self.metadata.id)?;
        if self.metadata.name.trim().is_empty() || self.metadata.name.len() > 96 {
            return Err(ThemeError::InvalidDocument(
                "name must contain 1 to 96 characters".into(),
            ));
        }
        for (label, value) in [
            ("family", &self.metadata.family),
            ("description", &self.metadata.description),
            ("author", &self.metadata.author),
        ] {
            if value.len() > 256 || value.chars().any(char::is_control) {
                return Err(ThemeError::InvalidDocument(format!(
                    "{label} must be at most 256 characters and contain no control characters"
                )));
            }
        }
        for (mode, palette) in std::iter::once(("default", &self.palette)).chain(
            self.variants
                .iter()
                .map(|(mode, palette)| (mode.as_str(), palette)),
        ) {
            validate_palette(palette, mode)?;
        }
        for (label, font) in [
            ("font_ui", &self.design.font_ui),
            ("font_mono", &self.design.font_mono),
            ("font_display", &self.design.font_display),
        ] {
            if font.is_empty()
                || font.len() > 64
                || !font
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_'))
            {
                return Err(ThemeError::InvalidDocument(format!(
                    "{label} must use letters, numbers, spaces, hyphens, or underscores"
                )));
            }
        }
        Ok(())
    }

    /// Resolve the requested appearance, falling back to the base palette.
    pub fn palette_for(&self, appearance: Appearance) -> &Palette {
        self.variants.get(&appearance).unwrap_or(&self.palette)
    }

    /// Build a contrast report for text and accent colors on common surfaces.
    pub fn contrast_report(&self, appearance: Appearance) -> ContrastReport {
        let palette = self.palette_for(appearance);
        ContrastReport {
            checks: vec![
                ContrastCheck::new("text on background", &palette.white, &palette.bg, 4.5),
                ContrastCheck::new("text on panel", &palette.white, &palette.panel, 4.5),
                ContrastCheck::new("muted text on background", &palette.muted, &palette.bg, 4.5),
                ContrastCheck::new("muted text on panel", &palette.muted, &palette.panel, 4.5),
                ContrastCheck::new(
                    "primary accent on background",
                    &palette.acid_green,
                    &palette.bg,
                    3.0,
                ),
                ContrastCheck::new(
                    "secondary accent on background",
                    &palette.hot_pink,
                    &palette.bg,
                    3.0,
                ),
            ],
        }
    }

    /// CSS variables understood by the existing Cybercore apps plus shared
    /// typography, density, corner, and motion tokens.
    pub fn to_css(&self, appearance: Appearance) -> String {
        let p = self.palette_for(appearance);
        let radius = match self.design.corners {
            Corners::Sharp => "2px",
            Corners::Soft => "8px",
            Corners::Rounded => "14px",
        };
        let spacing = match self.design.density {
            Density::Compact => "0.85",
            Density::Comfortable => "1",
            Density::Spacious => "1.15",
        };
        let duration = match self.design.motion {
            Motion::Full => "0.24s",
            Motion::Reduced => "0.08s",
            Motion::None => "0s",
        };
        format!(
            ":root{{--bg:#{bg};--fg:#{fg};--acid:#{acid};--pink:#{pink};--purple:#{purple};--cyan:#{cyan};--orange:#{orange};--red:#{red};--panel:#{panel};--line:#{line};--muted:#{muted};--font-ui:\"{font_ui}\",sans-serif;--font-mono:\"{font_mono}\",monospace;--font-display:\"{font_display}\",sans-serif;--radius:{radius};--density:{spacing};--dur-mid:{duration};--reduced-motion:{reduced};}}\n",
            bg = p.bg.trim_start_matches('#'),
            fg = p.white.trim_start_matches('#'),
            acid = p.acid_green.trim_start_matches('#'),
            pink = p.hot_pink.trim_start_matches('#'),
            purple = p.purple.trim_start_matches('#'),
            cyan = p.cyan.trim_start_matches('#'),
            orange = p.orange.trim_start_matches('#'),
            red = p.red.trim_start_matches('#'),
            panel = p.panel.trim_start_matches('#'),
            line = p.line.trim_start_matches('#'),
            muted = p.muted.trim_start_matches('#'),
            font_ui = self.design.font_ui,
            font_mono = self.design.font_mono,
            font_display = self.design.font_display,
            reduced = if self.design.motion == Motion::Full { "0" } else { "1" },
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ContrastCheck {
    pub label: &'static str,
    pub ratio: f64,
    pub minimum: f64,
    pub passes: bool,
}

impl ContrastCheck {
    fn new(label: &'static str, foreground: &str, background: &str, minimum: f64) -> Self {
        let ratio = contrast_ratio(foreground, background).unwrap_or(0.0);
        Self {
            label,
            ratio,
            minimum,
            passes: ratio >= minimum,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ContrastReport {
    pub checks: Vec<ContrastCheck>,
}

impl ContrastReport {
    pub fn passes(&self) -> bool {
        self.checks.iter().all(|check| check.passes)
    }
}

pub fn contrast_ratio(foreground: &str, background: &str) -> Option<f64> {
    let luminance = |hex: &str| -> Option<f64> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |start| {
            let byte = u8::from_str_radix(&hex[start..start + 2], 16).ok()? as f64 / 255.0;
            Some(if byte <= 0.04045 {
                byte / 12.92
            } else {
                ((byte + 0.055) / 1.055).powf(2.4)
            })
        };
        Some(0.2126 * channel(0)? + 0.7152 * channel(2)? + 0.0722 * channel(4)?)
    };
    let (fg, bg) = (luminance(foreground)?, luminance(background)?);
    let (lighter, darker) = if fg >= bg { (fg, bg) } else { (bg, fg) };
    Some((lighter + 0.05) / (darker + 0.05))
}

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    pub document: ThemeDocument,
    pub builtin: bool,
}

/// Read-only theme index. Built-ins come from the crate's embedded schema;
/// custom documents are discovered from the shared user theme directory.
#[derive(Debug, Clone)]
pub struct ThemeCatalog {
    entries: BTreeMap<String, ThemeEntry>,
    active: String,
    appearance: Appearance,
}

impl ThemeCatalog {
    pub fn load() -> Result<Self, ThemeError> {
        let schema = schema::load();
        let mut entries = BTreeMap::new();
        for (id, palette) in &schema.themes {
            let mut document = ThemeDocument::new(id.clone(), humanize(id), palette.clone());
            document.metadata.family = schema
                .theme_family(id)
                .unwrap_or(BUILTIN_FAMILY)
                .to_string();
            document.validate()?;
            entries.insert(
                id.clone(),
                ThemeEntry {
                    document,
                    builtin: true,
                },
            );
        }
        let root = user_themes_dir()?;
        if root.is_dir() {
            for item in fs::read_dir(&root).map_err(ThemeError::Io)? {
                let item = item.map_err(ThemeError::Io)?;
                let path = item.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                    continue;
                }
                let stem = path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                validate_id(stem)?;
                let input = fs::read_to_string(&path).map_err(ThemeError::Io)?;
                let document = ThemeDocument::from_json(&input, stem)?;
                if document.metadata.id != stem {
                    return Err(ThemeError::InvalidDocument(format!(
                        "theme file '{}' contains id '{}'",
                        path.display(),
                        document.metadata.id
                    )));
                }
                if entries.contains_key(stem) {
                    return Err(ThemeError::DuplicateId(stem.into()));
                }
                entries.insert(
                    stem.into(),
                    ThemeEntry {
                        document,
                        builtin: false,
                    },
                );
            }
        }
        let selection = read_active_selection().ok().flatten();
        let preferred = std::env::var("CYBERGRID_THEME")
            .ok()
            .filter(|id| entries.contains_key(id))
            .or_else(|| {
                selection
                    .as_ref()
                    .map(|value| value.id.clone())
                    .filter(|id| entries.contains_key(id))
            })
            .unwrap_or_else(|| schema.active.clone());
        Ok(Self {
            entries,
            active: preferred,
            appearance: selection.map(|value| value.appearance).unwrap_or_default(),
        })
    }

    pub fn get(&self, id: &str) -> Option<&ThemeEntry> {
        self.entries.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &ThemeEntry)> {
        self.entries.iter().map(|(id, entry)| (id.as_str(), entry))
    }

    pub fn active_id(&self) -> &str {
        &self.active
    }

    pub fn active_appearance(&self) -> Appearance {
        self.appearance
    }

    /// Save a portable custom theme to the shared user theme folder.
    pub fn save(&mut self, document: ThemeDocument) -> Result<(), ThemeError> {
        document.validate()?;
        if self
            .entries
            .get(&document.metadata.id)
            .is_some_and(|entry| entry.builtin)
        {
            return Err(ThemeError::ReservedId(document.metadata.id));
        }
        let root = user_themes_dir()?;
        fs::create_dir_all(&root).map_err(ThemeError::Io)?;
        let path = root.join(format!("{}.json", document.metadata.id));
        atomic_write(
            &path,
            &serde_json::to_vec_pretty(&document).map_err(ThemeError::Json)?,
        )?;
        self.entries.insert(
            document.metadata.id.clone(),
            ThemeEntry {
                document,
                builtin: false,
            },
        );
        Ok(())
    }

    pub fn select(&mut self, id: &str) -> Result<(), ThemeError> {
        if !self.entries.contains_key(id) {
            return Err(ThemeError::UnknownTheme(id.into()));
        }
        write_active_selection(id, self.appearance)?;
        self.active = id.into();
        Ok(())
    }

    pub fn set_appearance(&mut self, appearance: Appearance) -> Result<(), ThemeError> {
        write_active_selection(&self.active, appearance)?;
        self.appearance = appearance;
        Ok(())
    }

    pub fn remove_custom(&mut self, id: &str) -> Result<bool, ThemeError> {
        let Some(entry) = self.entries.get(id) else {
            return Ok(false);
        };
        if entry.builtin {
            return Err(ThemeError::ReservedId(id.into()));
        }
        fs::remove_file(user_themes_dir()?.join(format!("{id}.json"))).map_err(ThemeError::Io)?;
        self.entries.remove(id);
        if self.active == id {
            let fallback = schema::load().active.clone();
            write_active_selection(&fallback, self.appearance)?;
            self.active = fallback;
        }
        Ok(true)
    }
}

fn validate_id(id: &str) -> Result<(), ThemeError> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || id.starts_with('-')
        || id.ends_with('-')
    {
        return Err(ThemeError::InvalidDocument(
            "id must be 1-64 lowercase letters, digits, or hyphens".into(),
        ));
    }
    Ok(())
}

fn validate_palette(palette: &Palette, mode: &str) -> Result<(), ThemeError> {
    for (role, value) in [
        ("bg", &palette.bg),
        ("white", &palette.white),
        ("acid_green", &palette.acid_green),
        ("hot_pink", &palette.hot_pink),
        ("purple", &palette.purple),
        ("cyan", &palette.cyan),
        ("orange", &palette.orange),
        ("red", &palette.red),
        ("panel", &palette.panel),
        ("line", &palette.line),
        ("muted", &palette.muted),
    ] {
        let hex = value.trim_start_matches('#');
        if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ThemeError::InvalidColor {
                mode: mode.into(),
                role: role.into(),
                value: value.clone(),
            });
        }
    }
    Ok(())
}

fn user_config_dir() -> Result<PathBuf, ThemeError> {
    if let Some(path) = std::env::var_os("CYBERCORE_CONFIG_DIR") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("cybercore"));
    }
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".config/cybercore"))
        .ok_or(ThemeError::MissingConfigHome)
}

fn user_themes_dir() -> Result<PathBuf, ThemeError> {
    Ok(user_config_dir()?.join("themes"))
}

fn active_path() -> Result<PathBuf, ThemeError> {
    Ok(user_config_dir()?.join("active-theme.json"))
}

fn read_active_selection() -> Result<Option<ActiveTheme>, ThemeError> {
    let path = active_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(ThemeError::Io)?;
    let value: ActiveTheme = serde_json::from_str(&raw).map_err(ThemeError::Json)?;
    Ok(Some(value))
}

fn write_active_selection(id: &str, appearance: Appearance) -> Result<(), ThemeError> {
    let path = active_path()?;
    let parent = path.parent().ok_or(ThemeError::MissingConfigHome)?;
    fs::create_dir_all(parent).map_err(ThemeError::Io)?;
    let value = serde_json::to_vec_pretty(&ActiveTheme {
        id: id.into(),
        appearance,
    })
    .map_err(ThemeError::Json)?;
    atomic_write(&path, &value)
}

#[derive(Serialize, Deserialize)]
struct ActiveTheme {
    id: String,
    #[serde(default)]
    appearance: Appearance,
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ThemeError> {
    let parent = path.parent().ok_or(ThemeError::MissingConfigHome)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = parent.join(format!(".theme-{}-{nonce}.tmp", std::process::id()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(ThemeError::Io)?;
        file.write_all(bytes).map_err(ThemeError::Io)?;
        file.sync_all().map_err(ThemeError::Io)?;
        fs::rename(&temporary, path).map_err(ThemeError::Io)?;
        if let Ok(directory) = OpenOptions::new().read(true).open(parent) {
            let _ = directory.sync_all();
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn default_family() -> String {
    BUILTIN_FAMILY.into()
}

fn humanize(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().chain(chars).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug)]
pub enum ThemeError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidDocument(String),
    InvalidColor {
        mode: String,
        role: String,
        value: String,
    },
    UnsupportedVersion(u32),
    UnknownTheme(String),
    DuplicateId(String),
    ReservedId(String),
    MissingConfigHome,
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "theme file error: {error}"),
            Self::Json(error) => write!(f, "theme JSON error: {error}"),
            Self::InvalidDocument(message) => write!(f, "invalid theme: {message}"),
            Self::InvalidColor { mode, role, value } => {
                write!(f, "invalid {mode} palette color for {role}: {value:?}")
            }
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported theme format version {version}")
            }
            Self::UnknownTheme(id) => write!(f, "unknown theme '{id}'"),
            Self::DuplicateId(id) => write!(f, "theme id '{id}' is already in use"),
            Self::ReservedId(id) => write!(f, "cannot replace or remove built-in theme '{id}'"),
            Self::MissingConfigHome => {
                write!(f, "set CYBERCORE_CONFIG_DIR, XDG_CONFIG_HOME, or HOME")
            }
        }
    }
}

impl std::error::Error for ThemeError {}
