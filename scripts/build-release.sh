#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

binary="target/release/mike-t-4b46-mct-official-blog"
site_root="${LEPTOS_SITE_ROOT:-target/site}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to build the release binary." >&2
  exit 1
fi

if ! cargo leptos --version >/dev/null 2>&1; then
  echo "cargo-leptos is required. Install it with: cargo install cargo-leptos" >&2
  exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "npm is required to install frontend build dependencies." >&2
  exit 1
fi

npm ci
cargo leptos build --release

if [[ ! -x "$binary" ]]; then
  echo "release binary was not created at $binary" >&2
  exit 1
fi

if [[ ! -d "$site_root/pkg" ]]; then
  echo "static asset bundle was not created under $site_root/pkg" >&2
  exit 1
fi

echo "Release build ready:"
echo "  binary: $binary"
echo "  static assets: $site_root"
