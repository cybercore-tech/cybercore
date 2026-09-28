//! Canonical Cybercore tool status contract and atomic publishing helpers.
//!
//! Every Cybercore tool publishes its health, metrics, and actions to
//! `$XDG_RUNTIME_DIR/cybercore/<tool>.json` using atomic writes (temp file +
//! rename), ensuring readers (the TUI hub, Omarchy HUD) never observe partial JSON.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// Health status level for a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Ok,
    Watch,
    Warning,
    Urgent,
}

impl Default for Health {
    fn default() -> Self {
        Self::Ok
    }
}

/// A quantitative metric reported by the tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// A recent event or log item reported by the tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub time: String,
    pub severity: String, // "info", "warn", "error"
    pub text: String,
}

/// An interactive action/launcher linked to this tool (e.g., open TUI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub label: String,
    pub argv: Vec<String>,
}

/// Complete status snapshot published by a Cybercore tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStatus {
    pub schema_version: u32,
    pub tool: String,
    pub version: String,
    pub host: String,
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
        let host = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "localhost".to_string());
        let updated_at = chrono::Utc::now().to_rfc3339();
        Self {
            schema_version: 1,
            tool: "cybercore-tool".to_string(),
            version: "0.1.0".to_string(),
            host,
            updated_at,
            health: Health::Ok,
            summary: "Operational".to_string(),
            metrics: vec![],
            events: vec![],
            actions: vec![],
        }
    }
}

/// Resolve the canonical runtime path for a tool's status JSON.
///
/// `$XDG_RUNTIME_DIR/cybercore/<tool>.json`, falling back to `/tmp/cybercore`.
#[must_use]
pub fn path(tool_name: &str) -> PathBuf {
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime.trim().is_empty() {
            return PathBuf::from(runtime)
                .join("cybercore")
                .join(format!("{}.json", tool_name));
            }
    }
    std::env::temp_dir()
        .join("cybercore")
        .join(format!("{}.json", tool_name))
}

/// Atomically publish a tool status snapshot so readers never observe partial JSON.
///
/// # Errors
///
/// Returns an error when serialization, creation, writing, or renaming fails.
pub fn write(status: &ToolStatus) -> anyhow::Result<PathBuf> {
    let destination = path(&status.tool);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("creating status directory {}: {}", parent.display(), e))?;
    }

    let temporary = destination.with_extension("json.tmp");
    let mut file = File::create(&temporary)
        .map_err(|e| anyhow::anyhow!("creating temporary status file {}: {}", temporary.display(), e))?;
    serde_json::to_writer_pretty(&mut file, status)
        .map_err(|e| anyhow::anyhow!("encoding tool status: {}", e))?;
    file.write_all(b"\n")
        .map_err(|e| anyhow::anyhow!("terminating tool status file: {}", e))?;
    file.sync_all()
        .map_err(|e| anyhow::anyhow!("flushing tool status file: {}", e))?;
    fs::rename(&temporary, &destination)
        .map_err(|e| anyhow::anyhow!("publishing status file {} from {}: {}", destination.display(), temporary.display(), e))?;
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_serialization() {
        let status = ToolStatus {
            schema_version: 1,
            tool: "omniscient".to_string(),
            version: "0.5.0".to_string(),
            host: "blackbox".to_string(),
            updated_at: "2026-09-27T19:00:00Z".to_string(),
            health: Health::Ok,
            summary: "All systems nominal".to_string(),
            metrics: vec![Metric {
                label: "Score".to_string(),
                value: "100".to_string(),
                unit: Some("pts".to_string()),
            }],
            events: vec![],
            actions: vec![],
        };

        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["tool"], "omniscient");
        assert_eq!(json["health"], "ok");
        assert_eq!(json["metrics"][0]["label"], "Score");
    }
}
