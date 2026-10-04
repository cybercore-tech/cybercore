# Changelog

All notable changes to `cybercore` are documented in this file.

## [0.6.0] - Unreleased

### Added

- `cybercore::theme`: versioned portable theme documents, validation,
  dark/light palette variants, allowlisted design tokens, contrast reports,
  CSS generation, built-in family metadata, and shared user theme storage.
- `cybercore-theme-studio` workspace binary: standalone local visual editor
  with library selection, CRUD, dark/light editing, import/export, preview,
  and contrast feedback.
- CI now checks, lints, and tests every workspace package so Theme Studio is
  included in the standard Actions workflow.
- `docs/theme-engine.md` with the theme JSON contract, Rust API, storage
  precedence, and current app integration scope.
- `status` feature: the Cybercore tool status contract (`schema/status.json`)
  and `cybercore::status`, with `ToolStatus`, `Health`, `Metric`, `Event`,
  `Action` and atomic `write` / `write_in`, plus `read`, `path`, `dir` and
  `validate_tool_name`.
  - Status files live in `$XDG_RUNTIME_DIR/cybercore/<tool>.json`, falling
    back to `$HOME/.local/state/cybercore/status/`. There is no shared `/tmp`
    fallback. On Unix the directory is `0700` and files are `0600`.
  - Each write goes to a unique temporary file opened with `create_new`
    (existing files and symlinks are never followed), is synced, and is
    renamed into place. Tool names are restricted to letters, digits, `-` and
    `_`.
  - Off by default, so cybercore without `status` still depends only on serde.
- `publish_status` example (`--features status`).

 - Unreleased

### Added

- `components` module and `css/cybercomponents.css`: shared cards, status
  badges, data tables, form inputs, popup viewers and the theme-picker
  dropdown, coloured entirely from the active CYBERGRID theme.
- Brand assets (logos, header art, favicon, social card) under `assets/`.

### Fixed

- Removed a global `main {}` rule from the component CSS that broke the
  layout of every cyberdesk page.

### Changed

- Minimum supported Rust version is now **1.85**. 0.4.x declared 1.70, but
  its dependencies (serde, syn) no longer build on Rust 1.70, so the old value
  was not achievable.
- README: depend on `cybercore = "0.5"` from crates.io instead of a git tag.

## [0.4.1] - 2026-10-02

### Changed

- Metadata patch: repository and homepage links point to
  `github.com/cybercore-tech/cybercore`. No code changes from 0.4.0.

Earlier releases are tagged in Git (`v0.1.0` to `v0.4.0`).
