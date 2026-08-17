# slotgate stage 2 gate: CRAP scores and mirrored test coverage.
# Run alongside stage 1; if it is not green, the work is not done.

Set-Location (Split-Path -Parent $PSScriptRoot)

function Invoke-Crap4RustGate {
    param(
        [string]$Name,
        [string[]]$Packages,
        [double]$Threshold = 15
    )

    Write-Host ""
    Write-Host "=== $Name ===" -ForegroundColor Cyan

    if (-not (Get-Command cargo-crap4rust -ErrorAction SilentlyContinue)) {
        Write-Host "cargo-crap4rust is not installed." -ForegroundColor Red
        Write-Host "Install it with: cargo install cargo-crap4rust" -ForegroundColor Red
        exit 1
    }

    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path
    $args = @("crap4rust", "--manifest-path", $manifestPath)
    foreach ($package in $Packages) {
        $args += @("--package", $package)
    }
    $args += @("--warn-only", "--threshold", $Threshold.ToString())

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo @args 2>&1
    $ErrorActionPreference = $previousErrorActionPreference
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }

    if ($exitCode -ne 0) {
        Write-Host "FAILED: $Name (exit $exitCode)" -ForegroundColor Red
        exit $exitCode
    }

    $summaryLine = $output | Select-String -Pattern "summary:\s+total_functions=.*crappy_functions=(\d+)"
    if (-not $summaryLine) {
        Write-Host "FAILED: $Name (could not parse crap4rust summary)" -ForegroundColor Red
        exit 1
    }

    $crappyCount = [int]$summaryLine.Matches[0].Groups[1].Value
    if ($crappyCount -gt 0) {
        Write-Host "FAILED: $Name ($crappyCount crappy functions detected)" -ForegroundColor Red
        exit 1
    }
}

function Invoke-Twin4RustGate {
    param(
        [string]$Name,
        [string[]]$Packages
    )

    Write-Host ""
    Write-Host "=== $Name ===" -ForegroundColor Cyan

    if (-not (Get-Command cargo-twin4rust -ErrorAction SilentlyContinue)) {
        Write-Host "cargo-twin4rust is not installed." -ForegroundColor Red
        Write-Host "Install it with: cargo install cargo-twin4rust" -ForegroundColor Red
        exit 1
    }

    $manifestPath = (Resolve-Path (Join-Path $PSScriptRoot "..\Cargo.toml")).Path
    $args = @("twin4rust", "--manifest-path", $manifestPath)
    foreach ($package in $Packages) {
        $args += @("--package", $package)
    }

    $previousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo @args 2>&1
    $ErrorActionPreference = $previousErrorActionPreference
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }

    if ($exitCode -ne 0) {
        Write-Host "FAILED: $Name (source files without a mirrored test)" -ForegroundColor Red
        exit 1
    }
}

Invoke-Crap4RustGate "CRAP slotgate" @("slotgate")
Invoke-Twin4RustGate "Mirrored tests slotgate" @("slotgate")

Write-Host ""
Write-Host "slotgate stage 2 passed!" -ForegroundColor Green
