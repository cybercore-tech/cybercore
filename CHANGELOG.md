# Changelog

All notable changes to `cybercore` are documented in this file.

## [0.5.0] - Unreleased

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
