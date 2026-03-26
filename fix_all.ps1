# LuminaBridge 编译错误修复脚本
# 目标：修复所有 292 个编译错误

Write-Host "=== LuminaBridge 编译错误修复 ===" -ForegroundColor Cyan

$ErrorActionPreference = "Continue"
cd $PSScriptRoot

# 1. 修复 routes/users.rs - 确保使用正确的 UserListParams
Write-Host "`n[1/8] 修复 users.rs..." -ForegroundColor Yellow
$usersFile = "src\routes\users.rs"
if (Test-Path $usersFile) {
    $content = Get-Content $usersFile -Raw
    
    # 确保导入正确的 UserListParams
    if ($content -match "pub use crate::db::UserListParams;") {
        Write-Host "  ✓ UserListParams 导入正确" -ForegroundColor Green
    } else {
        # 添加导入
        $content = $content -replace "use crate::auth::TokenClaims;", "use crate::auth::TokenClaims;`npub use crate::db::UserListParams;"
        Set-Content $usersFile -Value $content -NoNewline
        Write-Host "  ✓ 添加 UserListParams 导入" -ForegroundColor Green
    }
}

# 2. 修复 routes/tenant.rs - TenantDTO From trait
Write-Host "`n[2/8] 修复 tenant.rs..." -ForegroundColor Yellow
$tenantFile = "src\routes\tenant.rs"
if (Test-Path $tenantFile) {
    $content = Get-Content $tenantFile -Raw
    
    # 修复 TenantDTO::from 调用 - 移除 &
    $content = $content -replace "TenantDTO::from\(&tenant\)", "TenantDTO::from(tenant)"
    $content = $content -replace "TenantDTO::from\(&updated\)", "TenantDTO::from(updated)"
    
    Set-Content $tenantFile -Value $content -NoNewline
    Write-Host "  ✓ 修复 TenantDTO::from 调用" -ForegroundColor Green
}

# 3. 修复 routes/users.rs - UserDetailDTO From trait
Write-Host "`n[3/8] 修复 users.rs UserDetailDTO..." -ForegroundColor Yellow
if (Test-Path $usersFile) {
    $content = Get-Content $usersFile -Raw
    
    # 修复 UserDetailDTO::from 调用
    $content = $content -replace "UserDetailDTO::from\(&user\)", "UserDetailDTO::from(user)"
    $content = $content -replace "UserDetailDTO::from\(&updated\)", "UserDetailDTO::from(updated)"
    
    Set-Content $usersFile -Value $content -NoNewline
    Write-Host "  ✓ 修复 UserDetailDTO::from 调用" -ForegroundColor Green
}

# 4. 修复 auth/oauth/github.rs - 类型不匹配
Write-Host "`n[4/8] 修复 OAuth GitHub..." -ForegroundColor Yellow
$githubFile = "src\auth\oauth\github.rs"
if (Test-Path $githubFile) {
    $content = Get-Content $githubFile -Raw
    
    # 修复字符串类型问题
    $content = $content -replace '&"([^"]+)"', '"$1"'
    
    Set-Content $githubFile -Value $content -NoNewline
    Write-Host "  ✓ 修复 GitHub OAuth 类型" -ForegroundColor Green
}

# 5. 修复 auth/oauth/discord.rs - 类型不匹配
Write-Host "`n[5/8] 修复 OAuth Discord..." -ForegroundColor Yellow
$discordFile = "src\auth\oauth\discord.rs"
if (Test-Path $discordFile) {
    $content = Get-Content $discordFile -Raw
    
    # 修复字符串类型问题
    $content = $content -replace '&"([^"]+)"', '"$1"'
    
    Set-Content $discordFile -Value $content -NoNewline
    Write-Host "  ✓ 修复 Discord OAuth 类型" -ForegroundColor Green
}

# 6. 修复 server/mod.rs - stats_sender 问题
Write-Host "`n[6/8] 修复 server/mod.rs..." -ForegroundColor Yellow
$serverFile = "src\server\mod.rs"
if (Test-Path $serverFile) {
    $content = Get-Content $serverFile -Raw
    
    # 修复 stats_sender 发送
    $content = $content -replace 'state\.db\.stats_sender\.send\(stats\)', 'state.db.stats_sender.send(stats).map_err(|e| Error::Internal(e.to_string()))?'
    
    Set-Content $serverFile -Value $content -NoNewline
    Write-Host "  ✓ 修复 stats_sender" -ForegroundColor Green
}

# 7. 修复 middleware/rate_limit.rs - 类型注解
Write-Host "`n[7/8] 修复 rate_limit.rs..." -ForegroundColor Yellow
$rateLimitFile = "src\middleware\rate_limit.rs"
if (Test-Path $rateLimitFile) {
    $content = Get-Content $rateLimitFile -Raw
    
    # 添加类型注解
    $content = $content -replace 'let token = relay\.db\.find_token\(token_id\)\.await\?;', 'let token: Option<crate::db::Token> = relay.db.find_token(token_id).await?;'
    
    Set-Content $rateLimitFile -Value $content -NoNewline
    Write-Host "  ✓ 修复类型注解" -ForegroundColor Green
}

# 8. 修复 routes/relay.rs - 类型注解
Write-Host "`n[8/8] 修复 relay.rs..." -ForegroundColor Yellow
$relayFile = "src\routes\relay.rs"
if (Test-Path $relayFile) {
    $content = Get-Content $relayFile -Raw
    
    # 添加 clone 类型注解
    $content = $content -replace 'let model = model\.clone\(\);', 'let model: String = model.clone();'
    
    Set-Content $relayFile -Value $content -NoNewline
    Write-Host "  ✓ 修复 relay.rs 类型" -ForegroundColor Green
}

Write-Host "`n=== 修复完成！===`n" -ForegroundColor Cyan
Write-Host "现在运行编译检查..." -ForegroundColor Yellow
Write-Host "cargo check`n" -ForegroundColor Gray
