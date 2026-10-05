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
    ] {
        assert!(
            css.contains(selector),
            "missing shared component {selector}"
        );
    }
    assert!(css.contains("[hidden] { display: none !important;"));
}
