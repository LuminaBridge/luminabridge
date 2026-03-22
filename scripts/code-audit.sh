#!/bin/bash
# LuminaBridge Code Audit Script
# Runs comprehensive code quality and security checks

set -e

echo "🔍 LuminaBridge Code Audit"
echo "=========================="
echo ""

# Rust code audit
echo "📦 Running Rust Clippy..."
echo "-------------------------"
cargo clippy --all-targets --all-features -- -D warnings || echo "⚠️  Clippy found some issues"
echo ""

echo "📦 Running Rust Format Check..."
echo "-------------------------------"
cargo fmt -- --check || echo "⚠️  Code formatting needs attention"
echo ""

echo "📦 Running Cargo Audit..."
echo "-------------------------"
if command -v cargo-audit &> /dev/null; then
    cargo audit || echo "⚠️  Security vulnerabilities found in dependencies"
else
    echo "⚠️  cargo-audit not installed. Install with: cargo install cargo-audit"
fi
echo ""

echo "📦 Running Cargo Outdated..."
echo "----------------------------"
if command -v cargo-outdated &> /dev/null; then
    cargo outdated || echo "⚠️  Some dependencies are outdated"
else
    echo "⚠️  cargo-outdated not installed. Install with: cargo install cargo-outdated"
fi
echo ""

# Frontend code audit
echo "📦 Running Frontend Lint..."
echo "--------------------------"
cd luminabridge-web
npm run lint || echo "⚠️  ESLint found some issues"
echo ""

echo "📦 Running Frontend Format Check..."
echo "------------------------------------"
npx prettier --check "src/**/*.{ts,tsx,css,md}" || echo "⚠️  Code formatting needs attention"
echo ""

echo "📦 Running npm Audit..."
echo "----------------------"
npm audit || echo "⚠️  Security vulnerabilities found in npm dependencies"
echo ""

cd ..

echo "✅ Code audit completed!"
echo ""
echo "Summary:"
echo "- Check Clippy warnings above"
echo "- Check ESLint errors above"
echo "- Review security vulnerabilities"
echo "- Fix formatting issues if any"
