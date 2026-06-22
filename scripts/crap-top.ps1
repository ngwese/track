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
#   .\scripts\crap-top.ps1
#   .\scripts\crap-top.ps1 --update-coverage
#   .\scripts\crap-top.ps1 -c --top 50

#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Show-Usage {
    @'
Show the top-N highest CRAP-score functions (human-readable table).

Policy and thresholds come from repo-root `.cargo-crap.toml`. Coverage is read
from `lcov.info` (gitignored); regenerate with --update-coverage.

Tool versions match CI (see .github/workflows/ci.yml):
  cargo install cargo-llvm-cov --locked --version 0.8.7
  cargo install cargo-crap --locked --version 0.2.2

Usage:
  .\scripts\crap-top.ps1
  .\scripts\crap-top.ps1 --update-coverage
  .\scripts\crap-top.ps1 -c --top 50
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
        Write-Err 'hint: see tool install lines in scripts/crap-top.ps1'
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

$Top = 30
$UpdateCoverage = $false
$ShowHelp = $false

$i = 0
while ($i -lt $args.Count) {
    $arg = $args[$i]
    switch ($arg) {
        { $_ -in '-c', '--update-coverage' } {
            $UpdateCoverage = $true
            $i++
        }
        { $_ -in '-n', '--top' } {
            if ($i + 1 -ge $args.Count) {
                Write-Err "error: $arg requires a number"
                exit 1
            }
            $Top = $args[$i + 1]
            $i += 2
        }
        { $_ -in '-h', '--help', '-?' } {
            $ShowHelp = $true
            $i++
        }
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
    '--top', "$Top",
    '--format', 'human'
)
