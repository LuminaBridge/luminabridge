# LuminaBridge Performance Benchmark Script (PowerShell)
# This script runs various performance benchmarks for the LuminaBridge API

param(
    [string]$BaseUrl = "http://localhost:3000",
    [string]$ApiToken = "sk-test-token",
    [string]$Duration = "30s",
    [int]$Connections = 100,
    [int]$Threads = 12
)

Write-Host "🌉 LuminaBridge Performance Benchmark" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host "Base URL: $BaseUrl"
Write-Host "Duration: $Duration"
Write-Host "Connections: $Connections"
Write-Host "Threads: $Threads"
Write-Host ""

# Check if wrk is installed
$wrkPath = Get-Command wrk -ErrorAction SilentlyContinue
if (-not $wrkPath) {
    Write-Host "❌ wrk is not installed. Please install it first:" -ForegroundColor Red
    Write-Host "   Windows: Download from https://github.com/wg/wrk"
    Write-Host "   Or use: chocolatey install wrk"
    exit 1
}

# Health check endpoint benchmark
Write-Host "📊 Benchmark: Health Check Endpoint" -ForegroundColor Yellow
Write-Host "-----------------------------------" -ForegroundColor Yellow
& wrk -t$Threads -c$Connections -d$Duration "$BaseUrl/health"
Write-Host ""

# API versions endpoint benchmark
Write-Host "📊 Benchmark: API Versions Endpoint" -ForegroundColor Yellow
Write-Host "------------------------------------" -ForegroundColor Yellow
& wrk -t$Threads -c$Connections -d$Duration "$BaseUrl/v1"
Write-Host ""

# Models endpoint benchmark (requires auth)
Write-Host "📊 Benchmark: Models Endpoint (Authenticated)" -ForegroundColor Yellow
Write-Host "---------------------------------------------" -ForegroundColor Yellow
& wrk -t$Threads -c$Connections -d$Duration `
    -H "Authorization: Bearer $ApiToken" `
    "$BaseUrl/v1/models"
Write-Host ""

# Chat completions endpoint benchmark (requires auth and POST)
Write-Host "📊 Benchmark: Chat Completions Endpoint" -ForegroundColor Yellow
Write-Host "----------------------------------------" -ForegroundColor Yellow

# Create temporary body file
$bodyFile = [System.IO.Path]::GetTempFileName()
@'
{
    "model": "gpt-3.5-turbo",
    "messages": [
        {"role": "user", "content": "Hello"}
    ],
    "max_tokens": 10
}
'@ | Out-File -FilePath $bodyFile -Encoding utf8

& wrk -t$Threads -c$Connections -d$Duration `
    -H "Authorization: Bearer $ApiToken" `
    -H "Content-Type: application/json" `
    --post $bodyFile `
    "$BaseUrl/v1/chat/completions"

# Cleanup
Remove-Item $bodyFile -Force

Write-Host ""
Write-Host "✅ Benchmark completed!" -ForegroundColor Green
