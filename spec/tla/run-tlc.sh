#!/usr/bin/env bash
# Run TLC on the Track hub sync model (ADR 0006).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

CONFIG="${1:-HubSync.cfg}"
SPEC="${2:-HubSync.tla}"

run_local() {
  if command -v tlc >/dev/null 2>&1; then
    tlc -config "$CONFIG" "$SPEC"
    return 0
  fi
  if command -v java >/dev/null 2>&1 && [[ -n "${TLA_TOOLS_JAR:-}" ]]; then
    java -cp "$TLA_TOOLS_JAR" tlc2.TLC -config "$CONFIG" "$SPEC"
    return 0
  fi
  return 1
}

run_docker() {
  docker run --rm \
    -v "$ROOT:/spec" \
    -w /spec \
    ghcr.io/tlaplus/tlaplus:latest \
    tlc -config "$CONFIG" "$SPEC"
}

if run_local; then
  exit 0
fi

echo "Local TLC not found; trying Docker image ghcr.io/tlaplus/tlaplus:latest..." >&2
run_docker
