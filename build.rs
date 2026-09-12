//! Merges `schema/cybergrid.json` (metadata: version, active theme, paths)
//! with every theme file under `schema/themes/**/*.json` into one JSON
//! document at build time, so the crate can still `include_str!` a single
//! embedded blob (zero runtime file dependency) while the source stays
//! chunked — one palette per file, grouped into folders by theme "family"
//! — instead of one hand-edited, ever-growing cybergrid.json.
//!
//! Each theme file's name (minus `.json`) becomes its slug; the folder
//! it's in is organizational only and isn't part of the merged schema.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_themes(dir: &Path, out: &mut serde_json::Map<String, serde_json::Value>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        // Re-run if any individual theme file changes — watching just the
        // parent directory isn't reliably enough for cargo to notice an
        // in-place edit to an existing file on every platform.
        println!("cargo:rerun-if-changed={}", path.display());

        if path.is_dir() {
            collect_themes(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let slug = path.file_stem().unwrap().to_string_lossy().to_string();
            let raw = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read theme file {}: {}", path.display(), e));
            let value: serde_json::Value = serde_json::from_str(&raw)
                .unwrap_or_else(|e| panic!("failed to parse theme file {} as JSON: {}", path.display(), e));
            if out.insert(slug.clone(), value).is_some() {
                panic!(
                    "duplicate theme slug \"{}\" — two files under schema/themes/ produce the same name ({})",
                    slug,
                    path.display()
                );
            }
        }
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let schema_dir = Path::new(&manifest_dir).join("schema");
    let meta_path = schema_dir.join("cybergrid.json");
    let themes_dir = schema_dir.join("themes");

    println!("cargo:rerun-if-changed={}", meta_path.display());
    println!("cargo:rerun-if-changed={}", themes_dir.display());

    let meta_raw = fs::read_to_string(&meta_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", meta_path.display(), e));
    let mut meta: serde_json::Value = serde_json::from_str(&meta_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {}", meta_path.display(), e));

    let mut themes = serde_json::Map::new();
    collect_themes(&themes_dir, &mut themes);

    meta.as_object_mut()
        .expect("cybergrid.json must be a JSON object")
        .insert("themes".to_string(), serde_json::Value::Object(themes));

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = PathBuf::from(out_dir).join("cybergrid_merged.json");
    fs::write(&out_path, serde_json::to_string_pretty(&meta).unwrap())
        .unwrap_or_else(|e| panic!("failed to write {}: {}", out_path.display(), e));
}
