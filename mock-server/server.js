/**
 * LuminaBridge Mock Server
 * 
 * A simplified mock server for integration testing
 * Implements key API endpoints for frontend testing
 */

import express from 'express';
import cors from 'cors';
import jwt from 'jsonwebtoken';
import { v4 as uuidv4 } from 'uuid';
import bcrypt from 'bcryptjs';
import { WebSocketServer } from 'ws';
import { createServer } from 'http';

const JWT_SECRET = 'super-secret-jwt-key-for-luminabridge-development-2024';
const PORT = 3000;

// In-memory database
const db = {
  users: [],
  channels: [],
  tokens: [],
  tenants: [
    {
      id: 1,
      name: 'Default Tenant',
      slug: 'default',
      status: 'active',
      quota_limit: 0,
      quota_used: 0,
    }
  ]
};

// Create default admin user
const defaultPassword = await bcrypt.hash('Admin123!', 10);
db.users.push({
  id: 1,
  tenant_id: 1,
  email: 'admin@luminabridge.io',
  password_hash: defaultPassword,
  display_name: 'Admin User',
  role: 'admin',
  status: 'active',
  created_at: new Date().toISOString(),
});

const app = express();
app.use(cors());
app.use(express.json());

// Middleware to verify JWT token
const authMiddleware = (req, res, next) => {
  const authHeader = req.headers.authorization;
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({
      success: false,
      error: { code: 'UNAUTHORIZED', message: 'Missing or invalid authorization header' }
    });
  }

  const token = authHeader.split(' ')[1];
  try {
    const decoded = jwt.verify(token, JWT_SECRET);
    req.user = decoded;
    next();
  } catch (error) {
    return res.status(401).json({
      success: false,
      error: { code: 'UNAUTHORIZED', message: 'Invalid or expired token' }
    });
  }
};

// Success response helper
const successResponse = (data, message = '操作成功') => {
  return {
    success: true,
    message,
    data
  };
};

// Error response helper
const errorResponse = (code, message, status = 400) => {
  return {
    success: false,
    error: { code, message }
  };
};

// ============================================================================
// Health Check
// ============================================================================

app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: '1.0.0-mock'
  });
});

// ============================================================================
// Authentication Routes
// ============================================================================

// Register
app.post('/api/v1/auth/register', async (req, res) => {
  try {
    const { email, password, display_name } = req.body;

    if (!email || !password) {
      return res.status(400).json(errorResponse('VALIDATION_ERROR', 'Email and password are required'));
    }

    // Check if user exists
    const existingUser = db.users.find(u => u.email === email);
    if (existingUser) {
      return res.status(409).json(errorResponse('USER_EXISTS', 'User already exists'));
    }

    // Hash password
    const password_hash = await bcrypt.hash(password, 10);

    // Create user
    const newUser = {
      id: db.users.length + 1,
      tenant_id: 1,
      email,
      password_hash,
      display_name: display_name || email,
      role: 'user',
      status: 'active',
      created_at: new Date().toISOString(),
    };

    db.users.push(newUser);

    // Generate tokens
    const token = jwt.sign(
      { user_id: newUser.id, email: newUser.email, role: newUser.role },
      JWT_SECRET,
      { expiresIn: '24h' }
    );

    const refresh_token = `rt_${newUser.id}_${uuidv4()}`;

    res.status(201).json(successResponse({
      token,
      refresh_token,
      user: {
        id: newUser.id,
        email: newUser.email,
        display_name: newUser.display_name,
        role: newUser.role,
      }
    }, '注册成功'));
  } catch (error) {
    console.error('Register error:', error);
    res.status(500).json(errorResponse('SERVER_ERROR', 'Internal server error'));
  }
});

// Login
app.post('/api/v1/auth/login', async (req, res) => {
  try {
    const { email, password, remember_me } = req.body;

    if (!email || !password) {
      return res.status(400).json(errorResponse('VALIDATION_ERROR', 'Email and password are required'));
    }

    // Find user
    const user = db.users.find(u => u.email === email);
    if (!user) {
      return res.status(401).json(errorResponse('INVALID_CREDENTIALS', 'Invalid email or password'));
    }

    // Verify password
    const passwordValid = await bcrypt.compare(password, user.password_hash);
    if (!passwordValid) {
      return res.status(401).json(errorResponse('INVALID_CREDENTIALS', 'Invalid email or password'));
    }

    // Check user status
    if (user.status !== 'active') {
      return res.status(403).json(errorResponse('USER_INACTIVE', 'User account is not active'));
    }

    // Generate tokens
    const token = jwt.sign(
      { user_id: user.id, email: user.email, role: user.role },
      JWT_SECRET,
      { expiresIn: '24h' }
    );

    const refresh_token = `rt_${user.id}_${uuidv4()}`;

    res.json(successResponse({
      token,
      refresh_token,
      user: {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        role: user.role,
      }
    }, '登录成功'));
  } catch (error) {
    console.error('Login error:', error);
    res.status(500).json(errorResponse('SERVER_ERROR', 'Internal server error'));
  }
});

// Logout
app.post('/api/v1/auth/logout', (req, res) => {
  res.json(successResponse({ logged_out: true }, '登出成功'));
});

// Refresh token
app.post('/api/v1/auth/refresh', (req, res) => {
  const { refresh_token } = req.body;

  if (!refresh_token || !refresh_token.startsWith('rt_')) {
    return res.status(401).json(errorResponse('INVALID_TOKEN', 'Invalid refresh token'));
  }

  // Extract user ID
  const parts = refresh_token.split('_');
  const userId = parseInt(parts[1]);
  const user = db.users.find(u => u.id === userId);

  if (!user) {
    return res.status(404).json(errorResponse('USER_NOT_FOUND', 'User not found'));
  }

  // Generate new tokens
  const token = jwt.sign(
    { user_id: user.id, email: user.email, role: user.role },
    JWT_SECRET,
    { expiresIn: '24h' }
  );

  const newRefreshToken = `rt_${user.id}_${uuidv4()}`;

  res.json(successResponse({
    token,
    refresh_token: newRefreshToken,
    user: {
      id: user.id,
      email: user.email,
      display_name: user.display_name,
      role: user.role,
    }
  }, 'Token refreshed successfully'));
});

// ============================================================================
// User Routes
// ============================================================================

app.get('/api/v1/users/me', authMiddleware, (req, res) => {
  const user = db.users.find(u => u.id === req.user.user_id);
  if (!user) {
    return res.status(404).json(errorResponse('USER_NOT_FOUND', 'User not found'));
  }

  res.json(successResponse({
    id: user.id,
    email: user.email,
    display_name: user.display_name,
    avatar_url: user.avatar_url,
    role: user.role,
    status: user.status,
  }));
});

// ============================================================================
// Channel Routes
// ============================================================================

app.get('/api/v1/channels', authMiddleware, (req, res) => {
  const userChannels = db.channels.filter(c => c.tenant_id === 1);
  res.json(successResponse({
    channels: userChannels,
    total: userChannels.length,
    page: 1,
    page_size: 100,
  }));
});

app.post('/api/v1/channels', authMiddleware, (req, res) => {
  const { name, type, key, base_url, models } = req.body;

  if (!name || !type || !key) {
    return res.status(400).json(errorResponse('VALIDATION_ERROR', 'Name, type, and key are required'));
  }

  const newChannel = {
    id: db.channels.length + 1,
    tenant_id: 1,
    name,
    channel_type: type,
    key,
    base_url: base_url || null,
    models: models || [],
    weight: 10,
    status: 'active',
    priority: 0,
    timeout_ms: 30000,
    retry_count: 3,
    balance: 0,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };

  db.channels.push(newChannel);

  res.status(201).json(successResponse(newChannel, '渠道创建成功'));
});

app.get('/api/v1/channels/:id', authMiddleware, (req, res) => {
  const channel = db.channels.find(c => c.id === parseInt(req.params.id));
  if (!channel) {
    return res.status(404).json(errorResponse('CHANNEL_NOT_FOUND', 'Channel not found'));
  }
  res.json(successResponse(channel));
});

app.put('/api/v1/channels/:id', authMiddleware, (req, res) => {
  const channelIndex = db.channels.findIndex(c => c.id === parseInt(req.params.id));
  if (channelIndex === -1) {
    return res.status(404).json(errorResponse('CHANNEL_NOT_FOUND', 'Channel not found'));
  }

  const { name, type, key, base_url, models, weight, status } = req.body;
  
  db.channels[channelIndex] = {
    ...db.channels[channelIndex],
    name: name || db.channels[channelIndex].name,
    channel_type: type || db.channels[channelIndex].channel_type,
    key: key || db.channels[channelIndex].key,
    base_url: base_url || db.channels[channelIndex].base_url,
    models: models || db.channels[channelIndex].models,
    weight: weight !== undefined ? weight : db.channels[channelIndex].weight,
    status: status || db.channels[channelIndex].status,
    updated_at: new Date().toISOString(),
  };

  res.json(successResponse(db.channels[channelIndex], '渠道更新成功'));
});

app.delete('/api/v1/channels/:id', authMiddleware, (req, res) => {
  const channelIndex = db.channels.findIndex(c => c.id === parseInt(req.params.id));
  if (channelIndex === -1) {
    return res.status(404).json(errorResponse('CHANNEL_NOT_FOUND', 'Channel not found'));
  }

  db.channels.splice(channelIndex, 1);
  res.json(successResponse({ deleted: true }, '渠道删除成功'));
});

// Test channel connection
app.post('/api/v1/channels/:id/test', authMiddleware, async (req, res) => {
  const channel = db.channels.find(c => c.id === parseInt(req.params.id));
  if (!channel) {
    return res.status(404).json(errorResponse('CHANNEL_NOT_FOUND', 'Channel not found'));
  }

  // Simulate connection test
  await new Promise(resolve => setTimeout(resolve, 500));

  const testResponse = {
    success: true,
    message: '连接成功',
    latency_ms: Math.floor(Math.random() * 100) + 20,
    models: channel.models,
  };

  res.json(successResponse(testResponse, '渠道测试成功'));
});

// ============================================================================
// Token Routes
// ============================================================================

app.get('/api/v1/tokens', authMiddleware, (req, res) => {
  const userTokens = db.tokens.filter(t => t.tenant_id === 1);
  res.json(successResponse({
    tokens: userTokens,
    total: userTokens.length,
  }));
});

app.post('/api/v1/tokens', authMiddleware, (req, res) => {
  const { name, quota_limit, expire_at, allowed_models } = req.body;

  const key = `sk-${uuidv4().replace(/-/g, '').substring(0, 32)}`;

  const newToken = {
    id: db.tokens.length + 1,
    tenant_id: 1,
    user_id: req.user.user_id,
    key,
    name: name || 'Untitled Token',
    quota_limit: quota_limit || 0,
    quota_used: 0,
    expire_at: expire_at || null,
    status: 'active',
    allowed_ips: [],
    allowed_models: allowed_models || [],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };

  db.tokens.push(newToken);

  // Return token with full key (only shown once)
  res.status(201).json(successResponse({
    ...newToken,
    key_display: key,
  }, 'Token 创建成功 - 请妥善保管您的密钥'));
});

app.get('/api/v1/tokens/:id', authMiddleware, (req, res) => {
  const token = db.tokens.find(t => t.id === parseInt(req.params.id));
  if (!token) {
    return res.status(404).json(errorResponse('TOKEN_NOT_FOUND', 'Token not found'));
  }
  
  // Don't return full key for security
  const { key, ...tokenSafe } = token;
  res.json(successResponse({
    ...tokenSafe,
    key_masked: `${key.substring(0, 6)}...${key.substring(key.length - 4)}`
  }));
});

app.delete('/api/v1/tokens/:id', authMiddleware, (req, res) => {
  const tokenIndex = db.tokens.findIndex(t => t.id === parseInt(req.params.id));
  if (tokenIndex === -1) {
    return res.status(404).json(errorResponse('TOKEN_NOT_FOUND', 'Token not found'));
  }

  db.tokens.splice(tokenIndex, 1);
  res.json(successResponse({ deleted: true }, 'Token 删除成功'));
});

// ============================================================================
// Stats Routes
// ============================================================================

app.get('/api/v1/stats/realtime', authMiddleware, (req, res) => {
  res.json(successResponse({
    tps: Math.floor(Math.random() * 100),
    rpm: Math.floor(Math.random() * 1000),
    latency_ms: Math.floor(Math.random() * 200) + 50,
    error_rate: (Math.random() * 0.05).toFixed(4),
    active_channels: db.channels.filter(c => c.status === 'active').length,
    timestamp: new Date().toISOString(),
  }));
});

app.get('/api/v1/stats/usage', authMiddleware, (req, res) => {
  res.json(successResponse({
    total_requests: Math.floor(Math.random() * 10000),
    total_tokens: Math.floor(Math.random() * 1000000),
    total_cost: (Math.random() * 100).toFixed(2),
    requests_by_model: [
      { model: 'gpt-4', count: Math.floor(Math.random() * 1000) },
      { model: 'gpt-3.5-turbo', count: Math.floor(Math.random() * 5000) },
      { model: 'claude-3', count: Math.floor(Math.random() * 500) },
    ],
  }));
});

// ============================================================================
// Relay API (OpenAI-compatible)
// ============================================================================

app.post('/v1/chat/completions', async (req, res) => {
  const authHeader = req.headers.authorization;
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({
      error: { message: 'Missing authorization header', type: 'authentication_error' }
    });
  }

  const token = authHeader.split(' ')[1];
  
  // Find token
  const apiToken = db.tokens.find(t => t.key === token);
  if (!apiToken) {
    return res.status(401).json({
      error: { message: 'Invalid API key', type: 'authentication_error' }
    });
  }

  if (apiToken.status !== 'active') {
    return res.status(403).json({
      error: { message: 'Token is not active', type: 'permission_error' }
    });
  }

  const { model, messages, max_tokens, temperature } = req.body;

  // Simulate response
  const mockResponse = {
    id: `chatcmpl-${uuidv4()}`,
    object: 'chat.completion',
    created: Math.floor(Date.now() / 1000),
    model: model || 'gpt-3.5-turbo',
    choices: [{
      index: 0,
      message: {
        role: 'assistant',
        content: `Hello! This is a mock response from LuminaBridge. I received your message: "${messages?.[0]?.content || 'no content'}"`,
      },
      finish_reason: 'stop',
    }],
    usage: {
      prompt_tokens: 10,
      completion_tokens: 20,
      total_tokens: 30,
    },
  };

  res.json(mockResponse);
});

app.get('/v1/models', authMiddleware, (req, res) => {
  res.json(successResponse({
    models: [
      { id: 'gpt-4', name: 'GPT-4', provider: 'openai' },
      { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo', provider: 'openai' },
      { id: 'claude-3-opus', name: 'Claude 3 Opus', provider: 'anthropic' },
      { id: 'claude-3-sonnet', name: 'Claude 3 Sonnet', provider: 'anthropic' },
    ],
  }));
});

// ============================================================================
// WebSocket Server for Realtime Stats
// ============================================================================

const server = createServer(app);
const wss = new WebSocketServer({ server, path: '/api/v1/ws' });

wss.on('connection', (ws) => {
  console.log('WebSocket client connected');

  // Send stats every 2 seconds
  const interval = setInterval(() => {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        type: 'stats',
        data: {
          tps: Math.floor(Math.random() * 100),
          rpm: Math.floor(Math.random() * 1000),
          latency_ms: Math.floor(Math.random() * 200) + 50,
          error_rate: (Math.random() * 0.05).toFixed(4),
          active_channels: db.channels.filter(c => c.status === 'active').length,
          timestamp: new Date().toISOString(),
        }
      }));
    }
  }, 2000);

  ws.on('close', () => {
    console.log('WebSocket client disconnected');
    clearInterval(interval);
  });

  ws.on('error', (error) => {
    console.error('WebSocket error:', error);
    clearInterval(interval);
  });
});

// ============================================================================
// Start Server
// ============================================================================

server.listen(PORT, '0.0.0.0', () => {
  console.log(`
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   🌉 LuminaBridge Mock Server                             ║
║                                                           ║
║   Server running at: http://localhost:${PORT}              ║
║   WebSocket at: ws://localhost:${PORT}/api/v1/ws          ║
║                                                           ║
║   Default user: admin@luminabridge.io / Admin123!         ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
  `);
});
