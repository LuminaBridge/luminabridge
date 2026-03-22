# API Key Authentication Quick Start Guide

## 🚀 Quick Start

### 1. Create an API Token

First, create a token using the admin API (requires JWT authentication):

```bash
curl -X POST http://localhost:8080/api/v1/tokens \
  -H "Authorization: Bearer <your-jwt-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My API Token",
    "quota_limit": 100000,
    "allowed_models": ["gpt-3.5-turbo", "gpt-4"],
    "expire_at": 1735689600000
  }'
```

Response:
```json
{
  "id": 1,
  "key": "sk-a1b2c3d4e5f6g7h8",
  "name": "My API Token",
  "quota_limit": 100000,
  "status": "active"
}
```

### 2. Use the API Token

Use the token in the `Authorization` header:

```bash
curl -X POST http://localhost:8080/api/v1/relay/v1/chat/completions \
  -H "Authorization: Bearer sk-a1b2c3d4e5f6g7h8" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "user", "content": "Hello, world!"}
    ]
  }'
```

## 📋 API Reference

### Endpoints Requiring API Key Auth

All relay endpoints under `/api/v1/relay/v1/` require API key authentication:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/chat/completions` | POST | Chat completions (OpenAI-compatible) |
| `/completions` | POST | Text completions (legacy) |
| `/models` | GET | List available models |
| `/models/:id` | GET | Get model details |

### Request Headers

```
Authorization: Bearer sk-<your-api-key>
Content-Type: application/json
```

### Response Codes

| Code | Meaning | When |
|------|---------|------|
| 200 | OK | Request successful |
| 401 | Unauthorized | Token expired or invalid format |
| 403 | Forbidden | Model not permitted or IP not allowed |
| 404 | Not Found | Token not found |
| 429 | Too Many Requests | Quota exceeded |

## 🔐 Token Configuration Options

### Unlimited Token

```json
{
  "name": "Unlimited Token",
  "quota_limit": 0,
  "allowed_models": [],
  "allowed_ips": []
}
```

- `quota_limit: 0` = unlimited quota
- `allowed_models: []` = all models allowed
- `allowed_ips: []` = all IPs allowed

### Model-Restricted Token

```json
{
  "name": "GPT-3.5 Only",
  "quota_limit": 50000,
  "allowed_models": ["gpt-3.5-turbo"]
}
```

Only allows access to `gpt-3.5-turbo` model.

### Wildcard Model Access

```json
{
  "name": "GPT Family",
  "quota_limit": 100000,
  "allowed_models": ["gpt-*"]
}
```

Allows access to all GPT models (gpt-3.5-turbo, gpt-4, gpt-4-turbo, etc.)

### IP-Restricted Token

```json
{
  "name": "Office Only",
  "quota_limit": 100000,
  "allowed_ips": ["192.168.1.0/24", "10.0.0.1"]
}
```

Only allows requests from specified IPs.

### Time-Limited Token

```json
{
  "name": "30-Day Token",
  "quota_limit": 100000,
  "expire_at": 1738281600000
}
```

Token expires at the specified timestamp (Unix milliseconds).

## 💡 Best Practices

### 1. Use Descriptive Names

```json
{
  "name": "Production - GPT-4 - Marketing Team"
}
```

### 2. Apply Least Privilege

Only grant access to models that are actually needed:

```json
{
  "allowed_models": ["gpt-3.5-turbo"]
}
```

### 3. Set Reasonable Quotas

Prevent unexpected usage:

```json
{
  "quota_limit": 100000
}
```

### 4. Use Expiration Dates

For temporary access:

```json
{
  "expire_at": 1735689600000
}
```

### 5. Monitor Usage

Regularly check token usage stats:

```bash
curl http://localhost:8080/api/v1/tokens/{token_id}/stats \
  -H "Authorization: Bearer <jwt-token>"
```

## 🔍 Troubleshooting

### "Token not found" (404)

- Check that you're using the correct API key
- Verify the token exists in the database
- Ensure you're using `Bearer` prefix (not `Basic` or other)

### "Token expired" (401)

- Check the `expire_at` timestamp
- Create a new token with a future expiration date

### "Token quota exceeded" (429)

- Check current usage: `quota_used` vs `quota_limit`
- Increase quota or create a new token
- Wait for quota reset (if configured)

### "Model not permitted" (403)

- Check `allowed_models` configuration
- Use a wildcard pattern like `gpt-*` for flexibility
- Create a new token with broader model access

### "IP address not allowed" (403)

- Check `allowed_ips` configuration
- Verify your client IP address
- Add your IP to the whitelist or use `[]` for all IPs

## 📊 Monitoring Token Usage

### Check Token Status

```bash
curl http://localhost:8080/api/v1/tokens/{token_id} \
  -H "Authorization: Bearer <jwt-token>"
```

Response:
```json
{
  "id": 1,
  "key": "sk-xxx",
  "name": "My Token",
  "quota_limit": 100000,
  "quota_used": 45000,
  "status": "active",
  "last_used_at": "2024-01-15T10:30:00Z"
}
```

### Usage Statistics

```bash
curl "http://localhost:8080/api/v1/stats/usage?token_id=1" \
  -H "Authorization: Bearer <jwt-token>"
```

## 🔒 Security Notes

1. **Never expose API keys in client-side code**
   - Use backend proxy for frontend applications
   
2. **Rotate keys regularly**
   - Create new tokens and delete old ones periodically
   
3. **Use HTTPS in production**
   - API keys are transmitted in headers
   
4. **Monitor for unusual usage**
   - Set up alerts for quota thresholds
   
5. **Use IP restrictions when possible**
   - Adds an extra layer of security

## 📚 Additional Resources

- [Full Implementation Documentation](AUTH_IMPLEMENTATION.md)
- [API Reference](docs/api-reference.md)
- [Token Management Guide](docs/token-management.md)
