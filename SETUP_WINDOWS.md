# Windows development setup

This document describes how to set up a **Windows 10/11** machine for Track
development. It reflects a verified setup (2026-06-21).

## Quick start

From an elevated or normal PowerShell window at the repo root:

```powershell
.\scripts\setup-windows.ps1
```

The script installs rustup (if missing), the pinned Rust toolchain, change-risk
cargo subcommands, `llvm-tools-preview`, and Node.js. It is idempotent — safe to
re-run after toolchain or dependency updates.

Optional flags (see [scripts/setup-windows.ps1](scripts/setup-windows.ps1)):

```powershell
.\scripts\setup-windows.ps1 -IncludeDocs    # mdbook + mdbook-mermaid
.\scripts\setup-windows.ps1 -IncludeTla     # Temurin JDK 17
.\scripts\setup-windows.ps1 -SkipNode       # skip Node.js install
```

Open a **new shell** after the script finishes so `cargo` and `node` are on
`PATH`, then verify:

```powershell
cargo build --workspace
cargo test --workspace
.\scripts\crap-check.ps1 --update-coverage
```

## Verified results

| Step | Command | Result |
| --- | --- | --- |
| Build | `cargo build --workspace` | Pass |
| Test | `cargo test --workspace` | Pass (1 ignored test) |
| Change risk | `.\scripts\crap-check.ps1 --update-coverage` | Pass (0 regressions) |

## What gets installed

The sections below explain each tool. [scripts/setup-windows.ps1](scripts/setup-windows.ps1)
installs the **core** and **change-risk** items by default; **optional** items
require `-IncludeDocs` or `-IncludeTla`.

### Core prerequisites (Rust workspace)

Required for any change under `crates/`. The setup script reads
[rust-toolchain.toml](rust-toolchain.toml) and installs the pinned channel.

| Tool | Version | Role |
| --- | --- | --- |
| [Rust](https://rustup.rs/) | **1.96.0** (pinned) | Compiler and Cargo workspace |
| `rustfmt` | bundled with 1.96.0 | `cargo fmt --all` |
| `clippy` | bundled with 1.96.0 | `cargo clippy --workspace --all-targets` |

On first use, rustup downloads toolchain `1.96.0-x86_64-pc-windows-msvc` and
its components.

Manual install (if not using the setup script):

1. Install [rustup](https://rustup.rs/)
2. `cd` into the repo — `rust-toolchain.toml` selects the toolchain

### Change-risk gates

Required for the CRAP checklist in [AGENTS.md](AGENTS.md). Versions match
[`.github/workflows/ci.yml`](.github/workflows/ci.yml). Installed by
`setup-windows.ps1`.

| Tool | Version | Role |
| --- | --- | --- |
| `cargo-llvm-cov` | 0.8.7 | Workspace LCOV coverage (`lcov.info`) |
| `cargo-crap` | 0.2.2 | CRAP scores vs `crap_baseline.json` |
| `llvm-tools-preview` | (matches toolchain) | LLVM profdata for `cargo-llvm-cov` |

Both cargo subcommands install to `%USERPROFILE%\.cargo\bin` (must be on
`PATH`).

Manual install (change-risk only):

```powershell
cargo install cargo-llvm-cov --locked --version 0.8.7
cargo install cargo-crap --locked --version 0.2.2
rustup component add llvm-tools-preview --toolchain 1.96.0-x86_64-pc-windows-msvc
```

### Markdown lint (default with setup script)

Required when editing `.md` files per [AGENTS.md](AGENTS.md). Node.js is
installed via `winget` unless `-SkipNode` is passed. `markdownlint-cli2` is
fetched on demand by `npx` — no global install.

| Tool | Version | Role |
| --- | --- | --- |
| Node.js | 24 (CI) | Runs `npx markdownlint-cli2` |
| `markdownlint-cli2` | 0.22.1 | Lint all `**/*.md` |

```powershell
npx --yes markdownlint-cli2@0.22.1 "**/*.md"
```

### Optional tools (CI parity)

Not required for build, test, or change-risk gates. Pass `-IncludeDocs` or
`-IncludeTla` to `setup-windows.ps1`, or install manually.

| Tool | Version | Used for | Setup script flag |
| --- | --- | --- | --- |
| `mdbook` | 0.5.3 | Developer/user guides | `-IncludeDocs` |
| `mdbook-mermaid` | 0.17.0 | Mermaid in dev guide | `-IncludeDocs` |
| Java (Temurin) | 17 | TLA+ model checking | `-IncludeTla` |
| TLC (`tla2tools.jar`) | 1.8.0 | Hub sync formal verification | Downloaded on first run; see `spec/tla/run-tlc.sh` |

## Build and test

From the repo root:

```powershell
cargo build --workspace
cargo test --workspace
```

Full Rust checklist from [AGENTS.md](AGENTS.md):

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
.\scripts\crap-check.ps1 --update-coverage
```

## Change-risk scripts

PowerShell helpers (preferred on Windows):

```powershell
# Regenerate coverage, then check gates
.\scripts\crap-check.ps1 --update-coverage

# Use existing lcov.info
.\scripts\crap-check.ps1

# Top-N highest CRAP scores
.\scripts\crap-top.ps1 --top 50
```

Bash equivalents ([Git Bash](https://git-scm.com/download/win) required):

```bash
./scripts/crap-check.sh --update-coverage
./scripts/crap-top.sh --top 50
```

Policy lives in [`.cargo-crap.toml`](.cargo-crap.toml). `lcov.info` is
gitignored; do not commit it.

Equivalent manual commands:

```powershell
cargo llvm-cov --workspace --lcov --output-path lcov.info
cargo crap --workspace --lcov lcov.info --baseline crap_baseline.json
```

## Troubleshooting

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| `cargo` not found after setup | PATH not refreshed | Open a new PowerShell window |
| `cargo-llvm-cov not found` | Subcommand not installed | Re-run `.\scripts\setup-windows.ps1` |
| `lcov.info not found` | Coverage not generated yet | `.\scripts\crap-check.ps1 --update-coverage` |
| `$'\r': command not found` in bash | WSL bash or CRLF issues | Use PowerShell scripts or Git Bash |
| `llvm-tools-preview` prompt | First `cargo llvm-cov` run | Allow rustup to install, or re-run setup script |
| CRAP regression failure | Score increased vs baseline | Add tests or refresh baseline per [AGENTS.md](AGENTS.md) |
| `winget not found` | App installer missing | Install Node/Java manually; see tables above |

## See also

- [scripts/setup-windows.ps1](scripts/setup-windows.ps1) — automated Windows setup
- [AGENTS.md](AGENTS.md) — agent and contributor completion checklist
- [docs/plans/cargo-crap-integration-plan.md](docs/plans/cargo-crap-integration-plan.md)
  — change-risk policy
- [.github/workflows/ci.yml](.github/workflows/ci.yml) — CI job definitions and
  pinned tool versions
