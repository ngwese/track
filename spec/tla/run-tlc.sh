#!/usr/bin/env bash
# Run TLC on the Track hub sync model (ADR 0006).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

CONFIG="${1:-HubSync.cfg}"
SPEC="${2:-HubSync.tla}"

# Pin to the same release as .github/workflows/ci.yml (tlc-hub-sync job).
TLC_VERSION="${TLC_VERSION:-1.8.0}"
TLA_TOOLS_JAR="${TLA_TOOLS_JAR:-$ROOT/.cache/tla2tools.jar}"

ensure_jar() {
  if [[ -f "$TLA_TOOLS_JAR" ]]; then
    return 0
  fi
  if ! command -v curl >/dev/null 2>&1; then
    echo "error: curl is required to download tla2tools.jar" >&2
    return 1
  fi
  local url="https://github.com/tlaplus/tlaplus/releases/download/v${TLC_VERSION}/tla2tools.jar"
  echo "Downloading TLC v${TLC_VERSION} to ${TLA_TOOLS_JAR}..." >&2
  mkdir -p "$(dirname "$TLA_TOOLS_JAR")"
  curl -fsSL -o "$TLA_TOOLS_JAR" "$url"
}

TLC_ARGS=(-cleanup -metadir states -config "$CONFIG" "$SPEC")

run_java_tlc() {
  if ! command -v java >/dev/null 2>&1; then
    return 1
  fi
  ensure_jar
  java -cp "$TLA_TOOLS_JAR" tlc2.TLC "${TLC_ARGS[@]}"
}

run_local() {
  if command -v tlc >/dev/null 2>&1; then
    tlc "${TLC_ARGS[@]}"
    return 0
  fi
  if run_java_tlc; then
    return 0
  fi
  return 1
}

run_docker() {
  docker run --rm \
    -v "$ROOT:/spec" \
    -w /spec \
    ghcr.io/tlaplus/tlaplus:latest \
    tlc -cleanup -metadir states -config "$CONFIG" "$SPEC"
}

if run_local; then
  exit 0
fi

if command -v docker >/dev/null 2>&1; then
  echo "Local TLC not found; trying Docker image ghcr.io/tlaplus/tlaplus:latest..." >&2
  if run_docker; then
    exit 0
  fi
fi

cat >&2 <<EOF
error: could not run TLC.

Install Java 11+ and re-run this script; it will download tla2tools.jar
v${TLC_VERSION} to:
  ${TLA_TOOLS_JAR}

Or set TLA_TOOLS_JAR to an existing tla2tools.jar, or install the TLC CLI
from https://github.com/tlaplus/tlaplus/releases
EOF
exit 1
