# Run CRAP change-risk gates (regression + absolute threshold).
#
# Policy and thresholds come from repo-root `.cargo-crap.toml`. Coverage is read
# from `lcov.info` (gitignored); regenerate with --update-coverage.
#
# Tool versions match CI (see .github/workflows/ci.yml):
#   cargo install cargo-llvm-cov --locked --version 0.8.7
#   cargo install cargo-crap --locked --version 0.2.2
#
# Usage:
#   .\scripts\crap-check.ps1
#   .\scripts\crap-check.ps1 --update-coverage
#   .\scripts\crap-check.ps1 -c

#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Show-Usage {
    @'
Run CRAP change-risk gates (regression + absolute threshold).

Policy and thresholds come from repo-root `.cargo-crap.toml`. Coverage is read
from `lcov.info` (gitignored); regenerate with --update-coverage.

Tool versions match CI (see .github/workflows/ci.yml):
  cargo install cargo-llvm-cov --locked --version 0.8.7
  cargo install cargo-crap --locked --version 0.2.2

Usage:
  .\scripts\crap-check.ps1
  .\scripts\crap-check.ps1 --update-coverage
  .\scripts\crap-check.ps1 -c
'@
}

function Write-Err {
    param([string]$Message)
    [Console]::Error.WriteLine($Message)
}

function Require-Command {
    param([string]$Name)

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Err "error: $Name not found in PATH"
        Write-Err 'hint: see tool install lines in scripts/crap-check.ps1'
        exit 1
    }
}

function Invoke-Cargo {
    param([Parameter(Mandatory)][string[]]$Arguments)

    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

$UpdateCoverage = $false
$ShowHelp = $false

foreach ($arg in $args) {
    switch ($arg) {
        { $_ -in '-c', '--update-coverage' } { $UpdateCoverage = $true }
        { $_ -in '-h', '--help', '-?' } { $ShowHelp = $true }
        default {
            Write-Err "error: unknown argument: $arg"
            Write-Err (Show-Usage)
            exit 1
        }
    }
}

if ($ShowHelp) {
    Show-Usage
    exit 0
}

$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location -LiteralPath $Root

$Lcov = Join-Path $Root 'lcov.info'

if ($UpdateCoverage) {
    Require-Command 'cargo-llvm-cov'
    Write-Host 'Generating workspace LCOV at lcov.info ...'
    Invoke-Cargo @('llvm-cov', '--workspace', '--lcov', '--output-path', $Lcov)
}
elseif (-not (Test-Path -LiteralPath $Lcov)) {
    Write-Err "error: $Lcov not found"
    Write-Err 'hint: run with --update-coverage to generate coverage first'
    exit 1
}

Require-Command 'cargo-crap'
Invoke-Cargo @(
    'crap',
    '--workspace',
    '--lcov', $Lcov,
    '--baseline', (Join-Path $Root 'crap_baseline.json')
)
