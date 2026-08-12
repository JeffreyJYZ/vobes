#!/usr/bin/env bash
set -euo pipefail

REPO="JeffreyJYZ/vobes"
DEFAULT_INSTALL="$HOME/.local/bin/vbs"

usage() {
  cat <<EOF
install-vbs — install the vbs CLI to $DEFAULT_INSTALL

usage: install-vbs.sh [options]

  --version TAG   install a specific release tag (default: latest)
  --source PATH   instead of downloading, install this local binary
  --print-target  print the install path and exit
  -h, --help      show this help

No sudo: the script refuses if the target exists or its directory is
not writable. Reinstalling means rm first.
EOF
}

VERSION=""
SOURCE=""

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2;;
    --source)  SOURCE="$2";  shift 2;;
    --print-target) printf '%s\n' "$DEFAULT_INSTALL"; exit 0;;
    -h|--help) usage; exit 0;;
    *) echo "unknown arg: $1" >&2; usage; exit 2;;
  esac
done

case "$(uname -s):$(uname -m)" in
  Darwin:arm64)   asset="vbs-macos-aarch64" ;;
  Darwin:x86_64)  asset="vbs-macos-x64" ;;
  Linux:x86_64)   asset="vbs-linux-x64" ;;
  Linux:aarch64)  asset="vbs-linux-aarch64" ;;
  MINGW*:*|CYGWIN*:*|MSYS*:*)
    echo "windows: install from PowerShell with the same asset name" >&2
    exit 1 ;;
  *) echo "no published asset for $(uname -s)/$(uname -m)" >&2; exit 1 ;;
esac

if [ -e "$DEFAULT_INSTALL" ]; then
  echo "exists: $DEFAULT_INSTALL" >&2
  echo "remove it first (no sudo by design): rm '$DEFAULT_INSTALL'" >&2
  exit 1
fi

install_dir="$(dirname "$DEFAULT_INSTALL")"
if [ ! -w "$install_dir" ] 2>/dev/null && ! mkdir -p "$install_dir" 2>/dev/null; then
  echo "cannot create $install_dir" >&2
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

if [ -n "$SOURCE" ]; then
  if [ ! -f "$SOURCE" ]; then
    echo "source not found: $SOURCE" >&2
    exit 1
  fi
  cp "$SOURCE" "$tmp/$asset"
else
  if [ -z "$VERSION" ]; then
    tag_json="$(curl -fsSL --proto '=https' \
      -H 'User-Agent: vbs-install' \
      -H 'Accept: application/vnd.github+json' \
      "https://api.github.com/repos/$REPO/releases/latest")"
    tag="$(printf '%s' "$tag_json" \
      | grep -m1 '"tag_name"' \
      | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/')"
    if [ -z "$tag" ]; then
      echo "could not determine latest tag" >&2
      exit 1
    fi
  else
    tag="$VERSION"
  fi

  case "$tag" in v*) tag="${tag#v}";; esac
  base="https://github.com/$REPO/releases/download/v$tag"
  case "$base" in
    https://github.com/*) ;;
    *) echo "refusing non-github url: $base" >&2; exit 1 ;;
  esac

  echo "fetching $base/$asset"
  curl -fSL --proto '=https' -H 'User-Agent: vbs-install' \
    -o "$tmp/$asset" "$base/$asset"

  echo "verifying sha256"
  curl -fSL --proto '=https' -H 'User-Agent: vbs-install' \
    -o "$tmp/$asset.sha256" "$base/$asset.sha256"

  expected="$(awk 'NF {print $1; exit}' "$tmp/$asset.sha256" | tr 'A-Z' 'a-z')"
  actual="$(sha256sum "$tmp/$asset" | awk '{print $1}' | tr 'A-Z' 'a-z')"
  if [ -z "$expected" ] || [ "$expected" != "$actual" ]; then
    echo "checksum mismatch: expected '$expected' got '$actual'" >&2
    exit 1
  fi
fi

mv "$tmp/$asset" "$DEFAULT_INSTALL"
chmod 0755 "$DEFAULT_INSTALL"

echo "installed: $DEFAULT_INSTALL"
"$DEFAULT_INSTALL" --version