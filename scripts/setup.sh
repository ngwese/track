#!/usr/bin/env bash
# Install the Rust toolchain and optional build helpers for Track.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

die() {
  echo "error: $*" >&2
  exit 1
}

info() {
  echo "==> $*"
}

ensure_path() {
  if [[ -f "${HOME}/.cargo/env" ]]; then
    # shellcheck disable=SC1091
    source "${HOME}/.cargo/env"
  fi
}

ensure_rustup() {
  if command -v rustup >/dev/null 2>&1; then
    return
  fi

  info "rustup not found; installing"
  if ! command -v curl >/dev/null 2>&1; then
    die "curl is required to install rustup"
  fi

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
  ensure_path
  command -v rustup >/dev/null 2>&1 || die "rustup install failed; add ~/.cargo/bin to PATH"
}

read_toml_array() {
  local file="$1"
  local key="$2"
  grep "^${key} = " "${file}" \
    | sed 's/.*\[\(.*\)\]/\1/' \
    | tr -d '"' \
    | tr ',' '\n' \
    | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' \
    | sed '/^$/d'
}

ensure_toolchain() {
  local toolchain_file="${ROOT}/rust-toolchain.toml"
  [[ -f "${toolchain_file}" ]] || die "missing rust-toolchain.toml"

  # Running cargo in a repo with rust-toolchain.toml installs the pinned channel.
  info "ensuring Rust toolchain from rust-toolchain.toml"
  cargo --version >/dev/null

  local toolchain
  toolchain="$(rustup show active-toolchain 2>/dev/null | awk '{print $1}')"
  [[ -n "${toolchain}" ]] || die "no active Rust toolchain"

  info "active toolchain: ${toolchain}"

  local item
  while IFS= read -r item; do
    if ! rustup component list --installed --toolchain "${toolchain}" | grep -q "^${item}-"; then
      info "installing component ${item}"
      rustup component add "${item}" --toolchain "${toolchain}"
    fi
  done < <(read_toml_array "${toolchain_file}" "components")

  while IFS= read -r item; do
    if ! rustup target list --installed --toolchain "${toolchain}" | grep -q "^${item}\$"; then
      info "installing target ${item}"
      rustup target add "${item}" --toolchain "${toolchain}"
    fi
  done < <(read_toml_array "${toolchain_file}" "targets")

  rustup show
}

cargo_install_if_missing() {
  local crate="$1"
  local version="$2"
  local bin_name="${3:-${crate##*/}}"

  if command -v "${bin_name}" >/dev/null 2>&1; then
    info "${bin_name} already installed ($(command -v "${bin_name}"))"
    return
  fi

  info "installing ${crate} ${version} via cargo install"
  cargo install "${crate}" --version "${version}" --locked
}

ensure_cargo_tools() {
  # Build-time WIT vendoring uses the wit-deps library crate. The wit-deps-cli
  # package provides the `wit-deps` binary for updating wit/deps.toml pins.
  cargo_install_if_missing "wit-deps-cli" "0.6.0" "wit-deps"
}

main() {
  ensure_path
  ensure_rustup
  ensure_toolchain
  ensure_cargo_tools

  info "setup complete"
  echo "Next: make build"
}

main "$@"
