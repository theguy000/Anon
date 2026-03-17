# Local CI check script — mirrors .github/workflows/ci.yml
# Run before pushing to catch issues early.

$ErrorActionPreference = "Stop"

function Write-Step($msg) {
    Write-Host "`n==> $msg" -ForegroundColor Cyan
}

$failed = @()

# ── Backend checks ──

Write-Step "Rust Format Check"
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
if ($LASTEXITCODE -ne 0) { $failed += "Rust Format" }

Write-Step "Clippy"
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
if ($LASTEXITCODE -ne 0) { $failed += "Clippy" }

Write-Step "Cargo Check"
cargo check --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { $failed += "Cargo Check" }

# ── Frontend checks ──

Write-Step "Svelte Check"
npm run check
if ($LASTEXITCODE -ne 0) { $failed += "Svelte Check" }

Write-Step "Frontend Build"
npm run build
if ($LASTEXITCODE -ne 0) { $failed += "Frontend Build" }

# ── Summary ──

Write-Host ""
if ($failed.Count -gt 0) {
    Write-Host "FAILED: $($failed -join ', ')" -ForegroundColor Red
    exit 1
} else {
    Write-Host "All checks passed!" -ForegroundColor Green
}
