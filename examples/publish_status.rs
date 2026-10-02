//! Publish a sample status snapshot and read it back.
//!
//!     cargo run --example publish_status --features status

use cybercore::status::{self, Action, Event, Health, Metric, ToolStatus};

fn main() -> std::io::Result<()> {
    let snapshot = ToolStatus {
        tool: "cybercore-demo".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        health: Health::Ok,
        summary: "Status contract verified locally".to_string(),
        metrics: vec![Metric {
            label: "Checks passed".to_string(),
            value: "100".to_string(),
            unit: Some("%".to_string()),
        }],
        events: vec![Event {
            time: status::now_rfc3339(),
            severity: "info".to_string(),
            text: "Published from the publish_status example".to_string(),
        }],
        actions: vec![Action {
            label: "Open Omniscient".to_string(),
            argv: vec!["omniscient".to_string()],
        }],
        ..ToolStatus::default()
    };

    let path = status::write(&snapshot)?;
    println!("published {}", path.display());
    let back = status::read(&path)?;
    println!(
        "read back: {} is {:?}: {}",
        back.tool, back.health, back.summary
    );
    Ok(())
}
