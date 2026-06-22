# Install the Rust toolchain and Track development dependencies on Windows.
#
# Versions match SETUP_WINDOWS.md and .github/workflows/ci.yml.
#
# Usage:
#   .\scripts\setup-windows.ps1
#   .\scripts\setup-windows.ps1 -IncludeDocs
#   .\scripts\setup-windows.ps1 -IncludeDocs -IncludeTla
#   .\scripts\setup-windows.ps1 -SkipNode

#Requires -Version 5.1
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Pinned cargo-install crates (keep in sync with ci.yml and crap-check.ps1).
$Script:CargoLlvmCovVersion = '0.8.7'
$Script:CargoCrapVersion = '0.2.2'
$Script:MdbookVersion = '0.5.3'
$Script:MdbookMermaidVersion = '0.17.0'

function Show-Usage {
    @'
Install the Rust toolchain and Track development dependencies on Windows.

Installs rustup (if missing), the pinned workspace toolchain, coverage and
change-risk cargo subcommands, llvm-tools-preview, and Node.js (for markdown
lint via npx). Optional switches add mdBook and Temurin JDK 17.

Versions match SETUP_WINDOWS.md and .github/workflows/ci.yml.

Usage:
  .\scripts\setup-windows.ps1
  .\scripts\setup-windows.ps1 -IncludeDocs
  .\scripts\setup-windows.ps1 -IncludeDocs -IncludeTla
  .\scripts\setup-windows.ps1 -SkipNode
  .\scripts\setup-windows.ps1 -Help
'@
}

function Write-Err {
    param([string]$Message)
    [Console]::Error.WriteLine($Message)
}

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message"
}

function Invoke-Checked {
    param(
        [Parameter(Mandatory)]
        [string]$Label,
        [Parameter(Mandatory)]
        [scriptblock]$Command
    )

    Write-Host "  $Label"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed (exit $LASTEXITCODE)"
    }
}

function Ensure-CargoBinOnPath {
    $cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
    if (-not (Test-Path -LiteralPath $cargoBin)) {
        return
    }

    $pathParts = $env:PATH -split ';'
    if ($pathParts -notcontains $cargoBin) {
        $env:PATH = "$cargoBin;$env:PATH"
        Write-Host "  prepended $cargoBin to PATH for this session"
    }
}

function Test-CommandAvailable {
    param([string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Install-Rustup {
    if (Test-CommandAvailable 'rustup') {
        Write-Host "  rustup already installed"
        return
    }

    Write-Host "  downloading rustup-init.exe"
    $rustupInit = Join-Path $env:TEMP 'rustup-init.exe'
    Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile $rustupInit

    Invoke-Checked 'rustup-init' {
        & $rustupInit -y --default-toolchain none
    }
}

function Read-ToolchainConfig {
    param([string]$Root)

    $channel = '1.96.0'
    $components = @('rustfmt', 'clippy')

    $toolchainFile = Join-Path $Root 'rust-toolchain.toml'
    if (-not (Test-Path -LiteralPath $toolchainFile)) {
        return [pscustomobject]@{
            Channel    = $channel
            Components = $components
        }
    }

    $content = Get-Content -LiteralPath $toolchainFile -Raw
    if ($content -match 'channel\s*=\s*"([^"]+)"') {
        $channel = $Matches[1]
    }
    if ($content -match 'components\s*=\s*\[([^\]]+)\]') {
        $parsed = $Matches[1] -split ',' | ForEach-Object {
            $_.Trim().Trim('"').Trim("'")
        } | Where-Object { $_ }
        if ($parsed.Count -gt 0) {
            $components = @($parsed)
        }
    }

    return [pscustomobject]@{
        Channel    = $channel
        Components = $components
    }
}

function Install-RustToolchain {
    param(
        [string]$Root,
        [string]$Channel,
        [string[]]$Components
    )

    Set-Location -LiteralPath $Root

    $componentArgs = @()
    foreach ($component in $Components) {
        $componentArgs += '--component'
        $componentArgs += $component
    }

    Invoke-Checked "rustup toolchain install $Channel" {
        & rustup toolchain install $Channel @componentArgs
    }

    Invoke-Checked 'rustup default' {
        & rustup default $Channel
    }

    Invoke-Checked 'verify cargo' {
        & cargo --version
    }

    $activeToolchain = (& rustup show active-toolchain).Split()[0]
    Invoke-Checked "rustup component add llvm-tools-preview ($activeToolchain)" {
        & rustup component add llvm-tools-preview --toolchain $activeToolchain
    }
}

function Install-CargoSubcommand {
    param(
        [string]$Crate,
        [string]$Version
    )

    if (Test-CommandAvailable $Crate) {
        Write-Host "  $Crate already on PATH"
    }

    Invoke-Checked "cargo install $Crate $Version" {
        & cargo install $Crate --locked --version $Version
    }
}

function Test-WingetPackageInstalled {
    param([string]$Id)

    if (-not (Test-CommandAvailable 'winget')) {
        return $false
    }

    $output = & winget list --id $Id --accept-source-agreements 2>&1
    if ($LASTEXITCODE -ne 0) {
        return $false
    }

    return [bool]($output | Select-String -SimpleMatch $Id)
}

function Install-WingetPackage {
    param(
        [string]$Id,
        [string]$Label,
        [string]$ManualUrl
    )

    if (-not (Test-CommandAvailable 'winget')) {
        Write-Host "  warning: winget not found; install $Label manually:"
        Write-Host "           $ManualUrl"
        return
    }

    if (Test-WingetPackageInstalled $Id) {
        Write-Host "  $Label already installed ($Id)"
        return
    }

    Invoke-Checked "winget install $Id" {
        & winget install -e --id $Id `
            --accept-package-agreements `
            --accept-source-agreements
    }
}

function Install-Node {
    # CI uses Node 24; OpenJS.NodeJS tracks the current Node.js release line.
    Install-WingetPackage `
        -Id 'OpenJS.NodeJS' `
        -Label 'Node.js' `
        -ManualUrl 'https://nodejs.org/'
}

function Install-DocsToolchain {
    Install-CargoSubcommand -Crate 'mdbook' -Version $Script:MdbookVersion
    Install-CargoSubcommand -Crate 'mdbook-mermaid' -Version $Script:MdbookMermaidVersion
}

function Install-TlaToolchain {
    Install-WingetPackage `
        -Id 'EclipseAdoptium.Temurin.17.JDK' `
        -Label 'Temurin JDK 17' `
        -ManualUrl 'https://adoptium.net/'
}

function Show-Summary {
    param(
        [bool]$IncludeDocs,
        [bool]$IncludeTla,
        [bool]$SkipNode
    )

    Write-Step 'Setup complete'
    Write-Host @'

Next steps (open a new shell if cargo/node were just installed):

  cargo build --workspace
  cargo test --workspace
  .\scripts\crap-check.ps1 --update-coverage

Markdown lint (requires Node.js on PATH):

  npx --yes markdownlint-cli2@0.22.1 "**/*.md"

See SETUP_WINDOWS.md for the full checklist and troubleshooting.
'@

    if ($IncludeDocs) {
        Write-Host @'

mdBook builds:

  cd docs/dev; mdbook-mermaid install .; mdbook build
  cd docs/user; mdbook build
'@
    }

    if ($IncludeTla) {
        Write-Host @'

TLA+ model check downloads tla2tools.jar on first CI run; locally see spec/tla/run-tlc.sh.
'@
    }

    if ($SkipNode) {
        Write-Host 'Note: Node.js was skipped; install it before editing Markdown.'
    }
}

$IncludeDocs = $false
$IncludeTla = $false
$SkipNode = $false
$ShowHelp = $false

$i = 0
while ($i -lt $args.Count) {
    $arg = $args[$i]
    switch ($arg) {
        '-IncludeDocs' { $IncludeDocs = $true; $i++ }
        '-IncludeTla' { $IncludeTla = $true; $i++ }
        '-SkipNode' { $SkipNode = $true; $i++ }
        { $_ -in '-h', '--help', '-?', '-Help' } { $ShowHelp = $true; $i++ }
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

if ($env:OS -notlike 'Windows*') {
    Write-Host 'warning: this script targets Windows; continuing anyway'
}

$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$toolchain = Read-ToolchainConfig -Root $Root

Write-Step 'Installing rustup (if needed)'
Install-Rustup
Ensure-CargoBinOnPath

if (-not (Test-CommandAvailable 'cargo')) {
    Write-Err 'error: cargo not found after rustup install'
    Write-Err 'hint: open a new PowerShell window or add %USERPROFILE%\.cargo\bin to PATH'
    exit 1
}

Write-Step "Installing Rust $($toolchain.Channel) (+ $($toolchain.Components -join ', '))"
Install-RustToolchain -Root $Root -Channel $toolchain.Channel -Components $toolchain.Components

Write-Step 'Installing change-risk cargo subcommands'
Install-CargoSubcommand -Crate 'cargo-llvm-cov' -Version $Script:CargoLlvmCovVersion
Install-CargoSubcommand -Crate 'cargo-crap' -Version $Script:CargoCrapVersion

if (-not $SkipNode) {
    Write-Step 'Installing Node.js (markdown lint via npx)'
    Install-Node
}

if ($IncludeDocs) {
    Write-Step 'Installing mdBook toolchain'
    Install-DocsToolchain
}

if ($IncludeTla) {
    Write-Step 'Installing Temurin JDK 17 (TLA+ model checking)'
    Install-TlaToolchain
}

Show-Summary -IncludeDocs:$IncludeDocs -IncludeTla:$IncludeTla -SkipNode:$SkipNode
