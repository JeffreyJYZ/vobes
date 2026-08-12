#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<EOF
install-vbs — install the vbs CLI via cargo (from crates.io)

usage: install-vbs.sh [options]

  --print-target  print the install path cargo will use and exit
  -h, --help      show this help

Requires Rust + cargo (https://rustup.rs).
EOF
}

print_target() {
  if command -v cargo >/dev/null 2>&1; then
    cargo install --list --quiet 2>/dev/null >/dev/null || true
    echo "$HOME/.cargo/bin/vbs"
  else
    echo "$HOME/.cargo/bin/vbs"
  fi
}

while [ $# -gt 0 ]; do
  case "$1" in
    --print-target) print_target; exit 0;;
    -h|--help) usage; exit 0;;
    *) echo "unknown arg: $1" >&2; usage; exit 2;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found." >&2
  echo "install Rust from https://rustup.rs, then re-run this script." >&2
  exit 1
fi

echo "installing vobes-cli via cargo"
exec cargo install vobes-cli --locked