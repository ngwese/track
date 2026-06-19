#!/usr/bin/env bash
# Show the top-N highest CRAP-score functions (human-readable table).
#
# Policy and thresholds come from repo-root `.cargo-crap.toml`. Coverage is read
# from `lcov.info` (gitignored); regenerate with --update-coverage.
#
# Tool versions match CI (see .github/workflows/ci.yml):
#   cargo install cargo-llvm-cov --locked --version 0.8.7
#   cargo install cargo-crap --locked --version 0.2.2
#
# Usage:
#   ./scripts/crap-top.sh
#   ./scripts/crap-top.sh --update-coverage
#   ./scripts/crap-top.sh -c --top 50

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

LCOV="$ROOT/lcov.info"
TOP=30
UPDATE_COVERAGE=0

usage() {
  sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//'
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: $1 not found in PATH" >&2
    echo "hint: see tool install lines in scripts/crap-top.sh" >&2
    exit 1
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -c | --update-coverage)
      UPDATE_COVERAGE=1
      shift
      ;;
    -n | --top)
      if [[ $# -lt 2 ]]; then
        echo "error: $1 requires a number" >&2
        exit 1
      fi
      TOP="$2"
      shift 2
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "$UPDATE_COVERAGE" -eq 1 ]]; then
  require_cmd cargo-llvm-cov
  echo "Generating workspace LCOV at lcov.info ..."
  cargo llvm-cov --workspace --lcov --output-path "$LCOV"
elif [[ ! -f "$LCOV" ]]; then
  echo "error: $LCOV not found" >&2
  echo "hint: run with --update-coverage to generate coverage first" >&2
  exit 1
fi

require_cmd cargo-crap
cargo crap --workspace --lcov "$LCOV" --top "$TOP" --format human
