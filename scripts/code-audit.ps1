# LuminaBridge Code Audit Script (PowerShell)
# Runs comprehensive code quality and security checks

Write-Host "🔍 LuminaBridge Code Audit" -ForegroundColor Cyan
Write-Host "==========================" -ForegroundColor Cyan
Write-Host ""

# Rust code audit
Write-Host "📦 Running Rust Clippy..." -ForegroundColor Yellow
Write-Host "-------------------------" -ForegroundColor Yellow
try {
    cargo clippy --all-targets --all-features -- -D warnings
} catch {
    Write-Host "⚠️  Clippy found some issues" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "📦 Running Rust Format Check..." -ForegroundColor Yellow
Write-Host "-------------------------------" -ForegroundColor Yellow
try {
    cargo fmt -- --check
} catch {
    Write-Host "⚠️  Code formatting needs attention" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "📦 Running Cargo Audit..." -ForegroundColor Yellow
Write-Host "-------------------------" -ForegroundColor Yellow
if (Get-Command cargo-audit -ErrorAction SilentlyContinue) {
    try {
        cargo audit
    } catch {
        Write-Host "⚠️  Security vulnerabilities found in dependencies" -ForegroundColor Yellow
    }
} else {
    Write-Host "⚠️  cargo-audit not installed. Install with: cargo install cargo-audit" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "📦 Running Cargo Outdated..." -ForegroundColor Yellow
Write-Host "----------------------------" -ForegroundColor Yellow
if (Get-Command cargo-outdated -ErrorAction SilentlyContinue) {
    try {
        cargo outdated
    } catch {
        Write-Host "⚠️  Some dependencies are outdated" -ForegroundColor Yellow
    }
} else {
    Write-Host "⚠️  cargo-outdated not installed. Install with: cargo install cargo-outdated" -ForegroundColor Yellow
}
Write-Host ""

# Frontend code audit
Write-Host "📦 Running Frontend Lint..." -ForegroundColor Yellow
Write-Host "--------------------------" -ForegroundColor Yellow
Set-Location luminabridge-web
try {
    npm run lint
} catch {
    Write-Host "⚠️  ESLint found some issues" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "📦 Running Frontend Format Check..." -ForegroundColor Yellow
Write-Host "------------------------------------" -ForegroundColor Yellow
try {
    npx prettier --check "src/**/*.{ts,tsx,css,md}"
} catch {
    Write-Host "⚠️  Code formatting needs attention" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "📦 Running npm Audit..." -ForegroundColor Yellow
Write-Host "----------------------" -ForegroundColor Yellow
try {
    npm audit
} catch {
    Write-Host "⚠️  Security vulnerabilities found in npm dependencies" -ForegroundColor Yellow
}
Write-Host ""

Set-Location ..

Write-Host "✅ Code audit completed!" -ForegroundColor Green
Write-Host ""
Write-Host "Summary:" -ForegroundColor Cyan
Write-Host "- Check Clippy warnings above"
Write-Host "- Check ESLint errors above"
Write-Host "- Review security vulnerabilities"
Write-Host "- Fix formatting issues if any"
