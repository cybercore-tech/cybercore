//! Canonical Cybercore tool status contract and atomic publishing helpers.
//!
//! Every Cybercore tool publishes its health, metrics, and actions as one JSON
//! file per tool. Readers (the TUI hub, the Omarchy HUD) never observe partial
//! JSON: each snapshot is written to a fresh temporary file in the same
//! directory and atomically renamed into place.
//!
//! Location, first match wins:
//!
//! 1. `$XDG_RUNTIME_DIR/cybercore/<tool>.json`
//! 2. `$HOME/.local/state/cybercore/status/<tool>.json`
//!
//! There is deliberately no shared `/tmp` fallback: a world-writable
//! directory would let another local user pre-create the path or plant
//! symlinks. On Unix the status directory is created with mode `0700`.
//!
//! Enabled with the `status` feature.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Current version of the status JSON contract (`schema/status.json`).
pub const SCHEMA_VERSION: u32 = 1;

/// Health status level for a tool.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    #[default]
    Ok,
    Watch,
    Warning,
    Urgent,
}

/// A quantitative metric reported by the tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metric {
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// A recent event or log item reported by the tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub time: String,
    /// `"info"`, `"warn"` or `"error"`.
    pub severity: String,
    pub text: String,
}

/// An interactive action or launcher linked to this tool (for example, open its TUI).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub label: String,
    pub argv: Vec<String>,
}

/// Complete status snapshot published by a Cybercore tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolStatus {
    pub schema_version: u32,
    pub tool: String,
    pub version: String,
    pub host: String,
    /// RFC 3339 timestamp.
    pub updated_at: String,
    pub health: Health,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<Metric>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
}

impl Default for ToolStatus {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            tool: "cybercore-tool".to_string(),
            version: "0.1.0".to_string(),
            host: hostname(),
            updated_at: now_rfc3339(),
            health: Health::Ok,
            summary: "Operational".to_string(),
            metrics: Vec::new(),
            events: Vec::new(),
            actions: Vec::new(),
        }
    }
}

/// This machine's hostname, or `"localhost"` if it cannot be read.
#[must_use]
pub fn hostname() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "localhost".to_string())
}

/// The current UTC time as an RFC 3339 string, for `updated_at` and event times.
#[must_use]
pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Validate a tool name for use as a file name: 1–64 ASCII letters, digits,
/// `-` or `_`, not starting with `-`.
///
/// # Errors
///
/// Returns [`io::ErrorKind::InvalidInput`] for any other name, so a name such
/// as `../x` can never escape the status directory.
pub fn validate_tool_name(tool_name: &str) -> io::Result<()> {
    let ok = !tool_name.is_empty()
        && tool_name.len() <= 64
        && !tool_name.starts_with('-')
        && tool_name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    if ok {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid cybercore tool name {tool_name:?}: use letters, digits, '-' or '_'"),
        ))
    }
}

/// The directory status files live in (see the module docs for the order).
///
/// # Errors
///
/// Returns [`io::ErrorKind::NotFound`] when neither `XDG_RUNTIME_DIR` nor
/// `HOME` is set.
pub fn dir() -> io::Result<PathBuf> {
    let non_empty = |key: &str| std::env::var_os(key).filter(|v| !v.is_empty());
    if let Some(runtime) = non_empty("XDG_RUNTIME_DIR") {
        return Ok(PathBuf::from(runtime).join("cybercore"));
    }
    if let Some(home) = non_empty("HOME") {
        return Ok(PathBuf::from(home).join(".local/state/cybercore/status"));
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "neither XDG_RUNTIME_DIR nor HOME is set; cannot locate the cybercore status directory",
    ))
}

/// Path of a tool's status JSON inside `dir`.
///
/// # Errors
///
/// Returns an error if `tool_name` is not a valid tool name.
pub fn path_in(dir: &Path, tool_name: &str) -> io::Result<PathBuf> {
    validate_tool_name(tool_name)?;
    Ok(dir.join(format!("{tool_name}.json")))
}

/// Canonical path of a tool's status JSON.
///
/// # Errors
///
/// Returns an error if the tool name is invalid or no status directory can be located.
pub fn path(tool_name: &str) -> io::Result<PathBuf> {
    path_in(&dir()?, tool_name)
}

/// Atomically publish a status snapshot to the canonical location.
///
/// # Errors
///
/// Returns an error when the tool name is invalid, no status directory can be
/// located, or serialization, writing, or renaming fails.
pub fn write(status: &ToolStatus) -> io::Result<PathBuf> {
    write_in(&dir()?, status)
}

/// Atomically publish a status snapshot into `dir`.
///
/// The snapshot is written to a uniquely named temporary file opened with
/// `create_new` (so an existing file or planted symlink is never followed),
/// synced, then renamed over `<tool>.json`. The temporary file is removed if
/// any step fails.
///
/// # Errors
///
/// Returns an error when the tool name is invalid or serialization, directory
/// creation, writing, syncing, or renaming fails.
pub fn write_in(dir: &Path, status: &ToolStatus) -> io::Result<PathBuf> {
    let destination = path_in(dir, &status.tool)?;
    create_private_dir(dir)?;

    let mut json = serde_json::to_vec_pretty(status).map_err(io::Error::other)?;
    json.push(b'\n');

    let temporary = dir.join(format!(".{}.{}.tmp", status.tool, unique_suffix()));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(&json)?;
        file.sync_all()?;
        fs::rename(&temporary, &destination)
    })();

    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(io::Error::new(
            error.kind(),
            format!("publishing status {}: {error}", destination.display()),
        ));
    }
    Ok(destination)
}

/// Read and parse a status file.
///
/// # Errors
///
/// Returns an error when the file cannot be read or is not valid status JSON.
pub fn read(path: &Path) -> io::Result<ToolStatus> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn create_private_dir(dir: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir)
}

fn unique_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    format!(
        "{}.{nanos}.{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(tool: &str) -> ToolStatus {
        ToolStatus {
            tool: tool.to_string(),
            version: "0.6.0".to_string(),
            host: "blackbox".to_string(),
            updated_at: "2026-09-27T19:00:00Z".to_string(),
            summary: "All systems nominal".to_string(),
            metrics: vec![Metric {
                label: "Score".to_string(),
                value: "100".to_string(),
                unit: Some("pts".to_string()),
            }],
            ..ToolStatus::default()
        }
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cybercore-status-test-{}-{name}-{}",
            std::process::id(),
            unique_suffix()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn serializes_to_the_schema_shape() {
        let json = serde_json::to_value(sample("omniscient")).unwrap();
        assert_eq!(json["schema_version"], SCHEMA_VERSION);
        assert_eq!(json["tool"], "omniscient");
        assert_eq!(json["health"], "ok");
        assert_eq!(json["metrics"][0]["label"], "Score");
        assert!(json.get("events").is_none(), "empty lists are omitted");
    }

    #[test]
    fn accepts_plain_tool_names() {
        for name in ["omniscient", "cyber-vault", "sentry_grid", "a1"] {
            assert!(validate_tool_name(name).is_ok(), "{name}");
        }
    }

    #[test]
    fn rejects_names_that_could_escape_the_directory() {
        for name in [
            "",
            "../x",
            "a/b",
            "a\\b",
            ".hidden",
            "-flag",
            "x.json",
            &"a".repeat(65),
        ] {
            let err = validate_tool_name(name).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidInput, "{name:?}");
        }
    }

    #[test]
    fn write_then_read_round_trips() {
        let dir = scratch_dir("roundtrip");
        let status = sample("roundtrip");
        let path = write_in(&dir, &status).unwrap();
        assert_eq!(path, dir.join("roundtrip.json"));
        assert_eq!(read(&path).unwrap(), status);
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temporary files are cleaned up");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rewrite_replaces_the_previous_snapshot() {
        let dir = scratch_dir("rewrite");
        let mut status = sample("rewrite");
        write_in(&dir, &status).unwrap();
        status.health = Health::Urgent;
        status.summary = "Disk almost full".to_string();
        let path = write_in(&dir, &status).unwrap();
        let back = read(&path).unwrap();
        assert_eq!(back.health, Health::Urgent);
        assert_eq!(back.summary, "Disk almost full");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn invalid_tool_name_writes_nothing() {
        let dir = scratch_dir("invalid");
        let err = write_in(&dir, &sample("../escape")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(!dir.exists(), "no directory or file is created");
    }

    #[cfg(unix)]
    #[test]
    fn status_directory_and_file_are_private() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch_dir("private");
        let path = write_in(&dir, &sample("private")).unwrap();
        let dir_mode = fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
        let file_mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700);
        assert_eq!(file_mode, 0o600);
        fs::remove_dir_all(&dir).unwrap();
    }
}
