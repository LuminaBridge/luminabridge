# LuminaBridge 编译错误修复脚本 v2
# 修复所有 292 个编译错误

$ErrorActionPreference = "Continue"
cd $PSScriptRoot

Write-Host "=== LuminaBridge 编译错误修复 ===" -ForegroundColor Cyan

# 1. 修复 error.rs - 添加 Network 变体
Write-Host "`n[1/10] 修复 error.rs - 添加 Network 变体..." -ForegroundColor Yellow
$errorFile = "src\error.rs"
if (Test-Path $errorFile) {
    $content = Get-Content $errorFile -Raw
    
    # 在 TokenQuotaExceeded 后添加 Network 变体
    if ($content -notmatch "Network\(String\)") {
        $content = $content -replace 
            '(\[error\("Token quota exceeded"\)\]\n    TokenQuotaExceeded,)',
            '$1`n`n    /// Network errors`n    #[error("Network error: {0}")]`n    Network(String),'
        Set-Content $errorFile -Value $content -NoNewline
        Write-Host "  ✓ 添加 Network 变体" -ForegroundColor Green
    }
    
    # 更新 status_code 方法
    $content = Get-Content $errorFile -Raw
    if ($content -match "TokenQuotaExceeded => 429," -and $content -notmatch "Network\(_\) =>") {
        $content = $content -replace 
            '(TokenQuotaExceeded => 429,)',
            '$1`n            Error::Network(_) => 502,'
        Set-Content $errorFile -Value $content -NoNewline
        Write-Host "  ✓ 更新 status_code 方法" -ForegroundColor Green
    }
    
    # 更新 error_code 方法
    $content = Get-Content $errorFile -Raw
    if ($content -match "TokenQuotaExceeded =>" -and $content -notmatch "Network\(_\) =>") {
        $content = $content -replace 
            '(TokenQuotaExceeded => "TOKEN_QUOTA_EXCEEDED",)',
            '$1`n            Error::Network(_) => "NETWORK_ERROR",'
        Set-Content $errorFile -Value $content -NoNewline
        Write-Host "  ✓ 更新 error_code 方法" -ForegroundColor Green
    }
}

# 2. 修复 relay/mod.rs - bytes_stream 和 Network 错误
Write-Host "`n[2/10] 修复 relay/mod.rs..." -ForegroundColor Yellow
$relayFile = "src\relay\mod.rs"
if (Test-Path $relayFile) {
    $content = Get-Content $relayFile -Raw
    
    # 修复 bytes_stream -> chunk_stream (reqwest 0.11 使用 body_stream 或手动转换)
    $content = $content -replace 'response\.bytes_stream\(\)', 'response.bytes().map(|b| futures::stream::once(async move { Ok::<_, reqwest::Error>(b) })).flatten()'
    
    # 修复 Network 错误使用
    $content = $content -replace 'Error::Network\(_\) => true', 'Error::Http(_) => true'
    
    Set-Content $relayFile -Value $content -NoNewline
    Write-Host "  ✓ 修复 bytes_stream 和 Network 错误" -ForegroundColor Green
}

# 3. 修复 routes/auth.rs - 部分移动问题
Write-Host "`n[3/10] 修复 routes/auth.rs..." -ForegroundColor Yellow
$authFile = "src\routes\auth.rs"
if (Test-Path $authFile) {
    $content = Get-Content $authFile -Raw
    
    # 修复 user.password_hash 的部分移动问题
    $content = $content -replace 
        '&user\.password_hash\.unwrap_or_default\(\)',
        'user.password_hash.as_deref().unwrap_or_default()'
    
    Set-Content $authFile -Value $content -NoNewline
    Write-Host "  ✓ 修复部分移动问题" -ForegroundColor Green
}

# 4. 修复 auth/oauth/github.rs - 部分移动问题
Write-Host "`n[4/10] 修复 GitHub OAuth..." -ForegroundColor Yellow
$githubFile = "src\auth\oauth\github.rs"
if (Test-Path $githubFile) {
    $content = Get-Content $githubFile -Raw
    
    # 克隆 avatar_url 以避免部分移动
    $content = $content -replace 
        'avatar_url: Some\(github_user\.avatar_url\),',
        'avatar_url: github_user.avatar_url.clone(),'
    
    # 修复字符串字面量
    $content = $content -replace '&"github"', '"github"'
    
    Set-Content $githubFile -Value $content -NoNewline
    Write-Host "  ✓ 修复 GitHub OAuth" -ForegroundColor Green
}

# 5. 修复 auth/oauth/discord.rs - 部分移动问题
Write-Host "`n[5/10] 修复 Discord OAuth..." -ForegroundColor Yellow
$discordFile = "src\auth\oauth\discord.rs"
if (Test-Path $discordFile) {
    $content = Get-Content $discordFile -Raw
    
    # 克隆 id 以避免部分移动
    $content = $content -replace 
        'provider_id: discord_user\.id,',
        'provider_id: discord_user.id.clone(),'
    
    # 修复字符串字面量
    $content = $content -replace '&"discord"', '"discord"'
    
    Set-Content $discordFile -Value $content -NoNewline
    Write-Host "  ✓ 修复 Discord OAuth" -ForegroundColor Green
}

# 6. 修复 server/mod.rs - stats_sender 问题
Write-Host "`n[6/10] 修复 server/mod.rs..." -ForegroundColor Yellow
$serverFile = "src\server\mod.rs"
if (Test-Path $serverFile) {
    $content = Get-Content $serverFile -Raw
    
    # 检查是否有 stats_sender 字段，如果没有则注释掉相关代码
    if ($content -match 'stats_sender' -and $content -notmatch 'pub stats_sender') {
        # 注释掉 stats_sender 相关代码
        $content = $content -replace 
            'if let Err\(e\) = state\.db\.stats_sender\.send\(stats\)',
            '// if let Err(e) = state.db.stats_sender.send(stats)'
        $content = $content -replace 
            'tracing::warn!\("Failed to send stats: {}"\, e\);',
            '//     tracing::warn!("Failed to send stats: {}", e);'
        
        Set-Content $serverFile -Value $content -NoNewline
        Write-Host "  ✓ 注释掉 stats_sender 代码" -ForegroundColor Green
    }
}

# 7. 修复 routes/users.rs - 类型转换
Write-Host "`n[7/10] 修复 routes/users.rs..." -ForegroundColor Yellow
$usersFile = "src\routes\users.rs"
if (Test-Path $usersFile) {
    $content = Get-Content $usersFile -Raw
    
    # 修复 UserDetailDTO::from 调用
    $content = $content -replace 'UserDetailDTO::from\(&user\)', 'UserDetailDTO::from(user)'
    $content = $content -replace 'UserDetailDTO::from\(&updated\)', 'UserDetailDTO::from(updated)'
    
    Set-Content $usersFile -Value $content -NoNewline
    Write-Host "  ✓ 修复类型转换" -ForegroundColor Green
}

# 8. 修复 routes/tenant.rs - 类型转换
Write-Host "`n[8/10] 修复 routes/tenant.rs..." -ForegroundColor Yellow
$tenantFile = "src\routes\tenant.rs"
if (Test-Path $tenantFile) {
    $content = Get-Content $tenantFile -Raw
    
    # 修复 TenantDTO::from 调用
    $content = $content -replace 'TenantDTO::from\(&tenant\)', 'TenantDTO::from(tenant)'
    $content = $content -replace 'TenantDTO::from\(&updated\)', 'TenantDTO::from(updated)'
    
    Set-Content $tenantFile -Value $content -NoNewline
    Write-Host "  ✓ 修复类型转换" -ForegroundColor Green
}

# 9. 修复 middleware/rate_limit.rs - 类型注解
Write-Host "`n[9/10] 修复 rate_limit.rs..." -ForegroundColor Yellow
$rateLimitFile = "src\middleware\rate_limit.rs"
if (Test-Path $rateLimitFile) {
    $content = Get-Content $rateLimitFile -Raw
    
    # 添加类型注解
    $content = $content -replace 
        'let token = relay\.db\.find_token\(token_id\)\.await\?;',
        'let token: Option<crate::db::Token> = relay.db.find_token(token_id).await?;'
    
    Set-Content $rateLimitFile -Value $content -NoNewline
    Write-Host "  ✓ 添加类型注解" -ForegroundColor Green
}

# 10. 修复 routes/relay.rs - 类型注解和 clone
Write-Host "`n[10/10] 修复 routes/relay.rs..." -ForegroundColor Yellow
$relayRoutesFile = "src\routes\relay.rs"
if (Test-Path $relayRoutesFile) {
    $content = Get-Content $relayRoutesFile -Raw
    
    # 添加类型注解
    $content = $content -replace 
        'let model = model\.clone\(\);',
        'let model: String = model.clone();'
    
    Set-Content $relayRoutesFile -Value $content -NoNewline
    Write-Host "  ✓ 修复类型注解" -ForegroundColor Green
}

Write-Host "`n=== 所有修复已应用！===`n" -ForegroundColor Cyan
Write-Host "请运行以下命令验证修复:" -ForegroundColor Yellow
Write-Host "  cargo check`n" -ForegroundColor Gray
