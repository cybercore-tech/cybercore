use crate::theme::{Appearance, ThemeDocument};

/// Release gate: every checked-in palette must satisfy the current portable
/// theme contract and produce the CSS roles all consumers depend on.
#[test]
fn every_embedded_theme_validates_and_renders_both_appearances() {
    let schema = crate::schema::load();
    assert!(
        schema.themes.len() >= 80,
        "the curated catalog unexpectedly shrank"
    );

    let mut families = std::collections::BTreeSet::new();
    for (id, palette) in &schema.themes {
        let mut document = ThemeDocument::new(id, id, palette.clone());
        document.metadata.family = schema.theme_family(id).unwrap_or("default").to_owned();
        document
            .validate()
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        families.insert(document.metadata.family.clone());

        for appearance in [Appearance::Dark, Appearance::Light] {
            let css = document.to_css(appearance);
            for role in [
                "--bg:",
                "--fg:",
                "--acid:",
                "--pink:",
                "--purple:",
                "--cyan:",
                "--orange:",
                "--red:",
                "--panel:",
                "--line:",
                "--muted:",
            ] {
                assert!(css.contains(role), "{id} {appearance:?} is missing {role}");
            }
        }
    }
    assert!(
        families.len() >= 6,
        "curated families unexpectedly disappeared"
    );
}

#[test]
fn shared_component_styles_keep_the_public_component_contract() {
    let css = crate::components::CSS;
    for selector in [
        ".window",
        ".theme-dropdown",
        ".res-table",
        ".text-input",
        ".viewer-overlay",
        ".cc-button-primary",
        ".cc-nav",
        ".cc-card",
        ".cc-alert",
        ".cc-form-field",
    ] {
        assert!(
            css.contains(selector),
            "missing shared component {selector}"
        );
    }
    assert!(css.contains("[hidden] { display: none !important;"));
    assert!(css.contains(":focus-visible"));
    assert!(css.contains("prefers-reduced-motion: reduce"));
}

#[test]
fn strict_theme_quality_report_covers_dark_and_light_accessibility() {
    let mut document = ThemeDocument::new(
        "quality-check",
        "Quality Check",
        crate::schema::Palette {
            bg: "000000".into(),
            white: "ffffff".into(),
            acid_green: "ffffff".into(),
            hot_pink: "ff00ff".into(),
            purple: "ff00ff".into(),
            cyan: "ffffff".into(),
            orange: "ffffff".into(),
            red: "ff0000".into(),
            panel: "000000".into(),
            line: "ffffff".into(),
            muted: "ffffff".into(),
        },
    );
    document.variants.insert(
        Appearance::Light,
        crate::schema::Palette {
            bg: "ffffff".into(),
            white: "000000".into(),
            acid_green: "000000".into(),
            hot_pink: "800040".into(),
            purple: "400060".into(),
            cyan: "005555".into(),
            orange: "663300".into(),
            red: "800000".into(),
            panel: "ffffff".into(),
            line: "000000".into(),
            muted: "000000".into(),
        },
    );
    let report = document.quality_report();
    assert!(report.has_explicit_light_variant);
    assert_eq!(report.checks.len(), 24);
    assert_eq!(report.failures().count(), 0);
}
