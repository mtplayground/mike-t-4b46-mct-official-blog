#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

env_file="${ENV_FILE:-.env.production}"
binary="${RELEASE_BINARY:-target/release/mike-t-4b46-mct-official-blog}"

if [[ -f "$env_file" ]]; then
  set -a
  # shellcheck disable=SC1090
  . "$env_file"
  set +a
fi

export LEPTOS_SITE_ADDR="${LEPTOS_SITE_ADDR:-0.0.0.0:8080}"
export LEPTOS_SITE_ROOT="${LEPTOS_SITE_ROOT:-target/site}"

if [[ ! -x "$binary" ]]; then
  echo "release binary is missing or not executable: $binary" >&2
  echo "Run ./scripts/build-release.sh first, or set RELEASE_BINARY." >&2
  exit 1
fi

if [[ ! -d "$LEPTOS_SITE_ROOT/pkg" ]]; then
  echo "static asset bundle is missing under $LEPTOS_SITE_ROOT/pkg" >&2
  echo "Run ./scripts/build-release.sh first, or set LEPTOS_SITE_ROOT." >&2
  exit 1
fi

exec "$binary"
