# Migration Guide: API Key Authentication

## Overview

This guide helps you migrate from the previous authentication system (hard-coded tenant ID) to the new API key authentication system.

## ⚠️ Breaking Changes

### Before (Old System)

```rust
// Hard-coded tenant ID
async fn extract_tenant_id() -> i64 { 1 }
```

**Issues:**
- All requests used tenant ID = 1
- No API key validation
- No quota management
- No access control

### After (New System)

```rust
// API key authentication required
Extension(auth): Extension<ApiKeyAuthExtension>
let tenant_id = auth.token.tenant_id;
```

**Benefits:**
- Secure token-based authentication
- Per-tenant isolation
- Quota management
- Model access control
- IP whitelisting

## 🔄 Migration Steps

### Step 1: Create API Tokens

Before the migration, create API tokens for all your applications:

```bash
# Create token for production
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <admin-jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Production App",
    "quota_limit": 1000000,
    "allowed_models": ["gpt-3.5-turbo", "gpt-4"],
    "allowed_ips": []
  }'
```

Save the returned API key: `sk-xxxxx`

### Step 2: Update Client Applications

Update all client applications to include the API key in requests:

**Before:**
```bash
curl -X POST http://localhost:8080/api/v1/relay/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-3.5-turbo", "messages": [...]}'
```

**After:**
```bash
curl -X POST http://localhost:8080/api/v1/relay/v1/chat/completions \
  -H "Authorization: Bearer sk-xxxxx" \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-3.5-turbo", "messages": [...]}'
```

### Step 3: Update Environment Variables

Add API keys to your environment configuration:

**.env**
```bash
# Old (no API key needed)
DATABASE_URL=postgres://...

# New
DATABASE_URL=postgres://...
API_KEY=sk-xxxxx
```

### Step 4: Test Authentication

Verify that authentication is working:

```bash
# Test with valid API key
curl -X POST http://localhost:8080/api/v1/relay/v1/chat/completions \
  -H "Authorization: Bearer sk-xxxxx" \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-3.5-turbo", "messages": [{"role": "user", "content": "Hello"}]}'

# Expected: 200 OK with response

# Test with invalid API key
curl -X POST http://localhost:8080/api/v1/relay/v1/chat/completions \
  -H "Authorization: Bearer sk-invalid" \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-3.5-turbo", "messages": [{"role": "user", "content": "Hello"}]}'

# Expected: 404 Token not found
```

### Step 5: Monitor Usage

Check token usage to ensure tracking is working:

```bash
curl http://localhost:8080/api/v1/tokens/1 \
  -H "Authorization: Bearer <admin-jwt>"
```

Expected response:
```json
{
  "id": 1,
  "key": "sk-xxxxx",
  "quota_limit": 1000000,
  "quota_used": 150,
  "last_used_at": "2024-01-15T10:30:00Z"
}
```

## 📝 Code Changes

### JavaScript/Node.js

**Before:**
```javascript
const response = await fetch('http://localhost:8080/api/v1/relay/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    model: 'gpt-3.5-turbo',
    messages: [{ role: 'user', content: 'Hello' }]
  })
});
```

**After:**
```javascript
const response = await fetch('http://localhost:8080/api/v1/relay/v1/chat/completions', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${process.env.API_KEY}`,
  },
  body: JSON.stringify({
    model: 'gpt-3.5-turbo',
    messages: [{ role: 'user', content: 'Hello' }]
  })
});
```

### Python

**Before:**
```python
import requests

response = requests.post(
    'http://localhost:8080/api/v1/relay/v1/chat/completions',
    json={
        'model': 'gpt-3.5-turbo',
        'messages': [{'role': 'user', 'content': 'Hello'}]
    }
)
```

**After:**
```python
import requests
import os

response = requests.post(
    'http://localhost:8080/api/v1/relay/v1/chat/completions',
    headers={
        'Authorization': f'Bearer {os.environ["API_KEY"]}'
    },
    json={
        'model': 'gpt-3.5-turbo',
        'messages': [{'role': 'user', 'content': 'Hello'}]
    }
)
```

### Rust

**Before:**
```rust
let response = client
    .post("http://localhost:8080/api/v1/relay/v1/chat/completions")
    .json(&request)
    .send()
    .await?;
```

**After:**
```rust
let response = client
    .post("http://localhost:8080/api/v1/relay/v1/chat/completions")
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&request)
    .send()
    .await?;
```

## 🔧 Configuration Examples

### Development Environment

Create a development token with unlimited quota:

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <admin-jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Development",
    "quota_limit": 0,
    "allowed_models": [],
    "allowed_ips": []
  }'
```

**Features:**
- `quota_limit: 0` = unlimited
- `allowed_models: []` = all models
- `allowed_ips: []` = all IPs

### Production Environment

Create a production token with restrictions:

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <admin-jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Production",
    "quota_limit": 10000000,
    "allowed_models": ["gpt-3.5-turbo", "gpt-4"],
    "allowed_ips": ["52.168.1.1", "52.168.1.2"],
    "expire_at": 1767225600000
  }'
```

**Features:**
- Limited quota
- Specific models only
- IP whitelist
- Expiration date

## 🚨 Rollback Plan

If you need to rollback to the old system:

### Option 1: Disable Middleware

In `src/routes/mod.rs`, remove the api_key_auth layer:

```rust
// Comment out this line:
// .layer(axum::middleware::from_fn_with_state(
//     state.clone(),
//     api_key_auth,
// ))
```

### Option 2: Create Catch-All Token

Create a token that allows everything:

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <admin-jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Catch-All",
    "quota_limit": 0,
    "allowed_models": [],
    "allowed_ips": []
  }'
```

Use this token in all clients during transition.

## 📊 Monitoring During Migration

### Check Token Usage

```bash
# List all tokens
curl http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <admin-jwt>"

# Check specific token
curl http://localhost:8080/api/v1/tokens/{token_id} \
  -H "Authorization: Bearer <admin-jwt>"

# Get usage statistics
curl "http://localhost:8080/api/v1/stats/usage?token_id={token_id}" \
  -H "Authorization: Bearer <admin-jwt>"
```

### Monitor Errors

Watch for authentication errors in logs:

```bash
# Look for auth errors
tail -f logs/luminabridge.log | grep -i "auth\|token\|401\|403\|404"
```

Common errors during migration:
- `401 Unauthorized` - Missing or invalid API key format
- `404 Token not found` - Invalid API key
- `403 Model not permitted` - Model not in allowed list
- `429 Token quota exceeded` - Quota limit reached

## ✅ Migration Checklist

- [ ] Create API tokens for all environments
- [ ] Update all client applications
- [ ] Update environment variables
- [ ] Test authentication with valid tokens
- [ ] Test error handling with invalid tokens
- [ ] Verify usage tracking is working
- [ ] Monitor logs during migration
- [ ] Document token keys securely
- [ ] Set up quota alerts
- [ ] Plan token rotation schedule

## 🆘 Troubleshooting

### Issue: "Missing authorization header"

**Cause:** Client not sending API key  
**Solution:** Add `Authorization: Bearer sk-xxx` header

### Issue: "Token not found"

**Cause:** Invalid API key  
**Solution:** Verify API key is correct and token exists

### Issue: "Model not permitted"

**Cause:** Model not in token's allowed_models  
**Solution:** Update token to include the model or use wildcard

### Issue: "Token quota exceeded"

**Cause:** Token has used all quota  
**Solution:** Increase quota_limit or create new token

### Issue: Requests still using tenant ID = 1

**Cause:** Old code still deployed  
**Solution:** Restart server with new code

## 📚 Additional Resources

- [API Key Authentication Guide](API_KEY_AUTH_GUIDE.md)
- [Implementation Details](AUTH_IMPLEMENTATION.md)
- [Error Codes Reference](AUTH_IMPLEMENTATION.md#error-codes)

## 📞 Support

If you encounter issues during migration:

1. Check the troubleshooting section above
2. Review server logs for detailed error messages
3. Verify token configuration in database
4. Test with curl to isolate client issues

---

**Migration Status:** Ready  
**Estimated Time:** 1-2 hours  
**Risk Level:** Medium (breaking change)  
**Rollback Available:** Yes
