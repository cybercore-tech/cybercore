fn main() -> anyhow::Result<()> {
    let status = cybercore::status::ToolStatus {
        schema_version: 1,
        tool: "cybercore-test".to_string(),
        version: "0.5.0".to_string(),
        host: hostname::get().map(|h| h.to_string_lossy().into_owned()).unwrap_or_else(|_| "localhost".to_string()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        health: cybercore::status::Health::Ok,
        summary: "Dogfooding test status published successfully".to_string(),
        metrics: vec![
            cybercore::status::Metric {
                label: "Dogfooding Score".to_string(),
                value: "100".to_string(),
                unit: Some("%".to_string()),
            },
        ],
        events: vec![
            cybercore::status::Event {
                time: chrono::Utc::now().to_rfc3339(),
                severity: "info".to_string(),
                text: "Status contract verified locally".to_string(),
            },
        ],
        actions: vec![
            cybercore::status::Action {
                label: "Launch TUI".to_string(),
                argv: vec!["omniscient".to_string()],
            },
        ],
    };

    let path = cybercore::status::write(&status)?;
    println!("✅ Published status atomically to: {}", path.display());

    let content = std::fs::read_to_string(&path)?;
    println!("📄 Content read back:\n{}", content);

    Ok(())
}
