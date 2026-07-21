# slotgate stage 1 gate: formatting, clippy, and all tests.
# Run before considering work complete; if it is not green, the work is not done.

Set-Location (Split-Path -Parent $PSScriptRoot)

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Command
    )
    Write-Host ""
    Write-Host "=== $Name ===" -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $Name (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Invoke-Step "Formatting" { cargo fmt }
Invoke-Step "Clippy" { cargo clippy --all-targets -- -D warnings }
Invoke-Step "Tests" { cargo test }

Write-Host ""
Write-Host "slotgate stage 1 passed!" -ForegroundColor Green
