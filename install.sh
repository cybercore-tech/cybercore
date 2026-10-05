#!/bin/sh
set -eu

INSTALL_COMMAND='cargo install --locked --force cybercore --bin cybercore-theme'

case "${1:-}" in
  --help|-h)
    cat <<'EOF'
Install the Cybercore theme quality and pack-checking CLI.

Requirements: Rust and Cargo (https://rustup.rs/)
This installs the cybercore-theme binary. It does not install the Rust library
or the standalone Cybercore Theme Studio application.

Usage: install.sh [--help|--dry-run]
EOF
    exit 0
    ;;
  --dry-run)
    printf 'Would run: %s\n' "$INSTALL_COMMAND"
    exit 0
    ;;
  '') ;;
  *) printf 'Unknown option: %s\nRun with --help for usage.\n' "$1" >&2; exit 2 ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
  printf 'Error: Cargo was not found. Install Rust and Cargo from https://rustup.rs/ and run this command again.\n' >&2
  exit 1
fi

printf 'Installing the Cybercore theme checker…\n'
# Intentionally word-split to execute the fixed Cargo argument list above.
# shellcheck disable=SC2086
$INSTALL_COMMAND
printf '\nInstalled cybercore-theme. Ensure the Cargo bin directory (usually ~/.cargo/bin) is on PATH.\n'
