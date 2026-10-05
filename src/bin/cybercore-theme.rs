use cybercore::theme::{ThemeDocument, ThemePackDocument};
use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(command) = args.first() else {
        usage();
        return ExitCode::from(2);
    };
    let strict = args.iter().any(|arg| arg == "--strict");
    let Some(path) = args.iter().skip(1).find(|arg| arg.as_str() != "--strict") else {
        usage();
        return ExitCode::from(2);
    };
    let input = match fs::read_to_string(path) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };

    match command.as_str() {
        "check" => check_theme(&input, path, strict),
        "validate-pack" => validate_pack(&input, path),
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn check_theme(input: &str, path: &str, strict: bool) -> ExitCode {
    let id = std::path::Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("imported-theme");
    let document = match ThemeDocument::from_json(input, id) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("theme validation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let report = document.quality_report();
    let failures: Vec<_> = report.failures().collect();
    for check in &report.checks {
        println!(
            "{}: {} — {:.2}:1 (minimum {:.1}:1) {}",
            check.appearance.as_str(),
            check.check.label,
            check.check.ratio,
            check.check.minimum,
            if check.check.passes { "PASS" } else { "WARN" },
        );
    }
    if !report.has_explicit_light_variant {
        println!("light appearance: compatibility fallback in use");
    }
    if strict && (!failures.is_empty() || !report.has_explicit_light_variant) {
        eprintln!(
            "strict theme quality gate failed: {} contrast issue(s), explicit light variant: {}",
            failures.len(),
            report.has_explicit_light_variant
        );
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn validate_pack(input: &str, path: &str) -> ExitCode {
    let pack: ThemePackDocument = match serde_json::from_str(input) {
        Ok(pack) => pack,
        Err(error) => {
            eprintln!("cannot parse pack {path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    match pack.validate() {
        Ok(()) => {
            println!(
                "valid pack {} v{}: {} theme(s), license {}, Cybercore {}",
                pack.metadata.id,
                pack.metadata.version,
                pack.themes.len(),
                if pack.metadata.license.is_empty() {
                    "unspecified"
                } else {
                    &pack.metadata.license
                },
                pack.metadata.compatibility,
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("pack validation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn usage() {
    eprintln!("Usage: cybercore-theme check <theme.json> [--strict]\n       cybercore-theme validate-pack <pack.cyberpack.json>");
}
