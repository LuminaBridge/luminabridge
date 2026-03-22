# LuminaBridge Docker Deployment Guide

🌉 Complete Docker configuration for containerized deployment of LuminaBridge.

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Services](#services)
- [Development](#development)
- [Production](#production)
- [Troubleshooting](#troubleshooting)

---

## 🚀 Quick Start

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- Git

### Basic Setup

```bash
# 1. Clone the repository
git clone https://github.com/LuminaBridge/luminabridge.git
cd luminabridge

# 2. Copy environment configuration
cp docker/.env.example .env

# 3. Customize environment variables (IMPORTANT: Change JWT_SECRET!)
# Edit .env file with your preferred editor

# 4. Build and start all services
docker-compose up -d --build

# 5. Check service status
docker-compose ps

# 6. View logs
docker-compose logs -f
```

### Access Points

| Service | URL | Port |
|---------|-----|------|
| Frontend | http://localhost | 80 |
| Backend API | http://localhost:3000 | 3000 |
| pgAdmin | http://localhost:8080 | 8080 |
| PostgreSQL | localhost | 5432 |
| Redis | localhost | 6379 |

---

## ⚙️ Configuration

### Environment Variables

Copy and customize the `.env` file from `docker/.env.example`:

```bash
cp docker/.env.example .env
```

#### Required Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `JWT_SECRET` | JWT signing secret (min 32 chars) | _Must set_ |
| `POSTGRES_PASSWORD` | Database password | _Must set_ |

#### Optional Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `POSTGRES_USER` | Database username | `luminabridge` |
| `POSTGRES_DB` | Database name | `luminabridge_dev` |
| `BACKEND_PORT` | Backend API port | `3000` |
| `FRONTEND_PORT` | Frontend port | `80` |
| `LOG_LEVEL` | Logging level | `info` |
| `PGADMIN_PORT` | pgAdmin port | `8080` |

### OAuth Configuration

For GitHub/Discord OAuth, set these in `.env`:

```bash
# GitHub
GITHUB_CLIENT_ID=your_client_id
GITHUB_CLIENT_SECRET=your_client_secret

# Discord
DISCORD_CLIENT_ID=your_client_id
DISCORD_CLIENT_SECRET=your_client_secret
```

---

## 🏗️ Services

### PostgreSQL

- **Image**: `postgres:15-alpine`
- **Port**: 5432
- **Volume**: Persistent data storage
- **Health Check**: Built-in

### Redis

- **Image**: `redis:7-alpine`
- **Port**: 6379
- **Volume**: AOF persistence
- **Health Check**: Built-in

### Backend

- **Build**: Multi-stage Rust build
- **Base**: `rust:1.75` → `debian:bookworm-slim`
- **Port**: 3000
- **Health Check**: `/health` endpoint

### Frontend

- **Build**: Multi-stage Node.js + Nginx
- **Base**: `node:20` → `nginx:alpine`
- **Port**: 80
- **Features**: 
  - Static file serving
  - API reverse proxy
  - SPA routing

### pgAdmin (Optional)

- **Image**: `dpage/pgadmin4:latest`
- **Port**: 8080
- **Profile**: `tools`

Enable with:
```bash
docker-compose --profile tools up -d
```

---

## 💻 Development

### Build Images

```bash
# Build all images
docker-compose build

# Build specific service
docker-compose build backend
docker-compose build frontend

# No cache (fresh build)
docker-compose build --no-cache backend
```

### Start Services

```bash
# Start all services
docker-compose up -d

# Start with logs
docker-compose up

# Start specific service
docker-compose up backend

# Rebuild and start
docker-compose up -d --build
```

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
docker-compose logs -f frontend

# Last 100 lines
docker-compose logs --tail=100 backend
```

### Run Commands

```bash
# Execute command in backend
docker-compose exec backend /bin/sh

# Execute command in frontend
docker-compose exec frontend /bin/sh

# Access PostgreSQL
docker-compose exec postgres psql -U luminabridge -d luminabridge_dev

# Access Redis
docker-compose exec redis redis-cli
```

### Stop Services

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (⚠️ deletes data!)
docker-compose down -v

# Stop specific service
docker-compose stop backend
```

---

## 🚢 Production

### Security Checklist

- [ ] Change `JWT_SECRET` to a secure random value
- [ ] Change default database passwords
- [ ] Change default pgAdmin credentials
- [ ] Disable pgAdmin in production (remove from compose)
- [ ] Use specific image versions (not `latest`)
- [ ] Enable rate limiting
- [ ] Configure proper CORS origins
- [ ] Use HTTPS/TLS termination
- [ ] Set up proper logging and monitoring

### Generate Secure JWT Secret

```bash
# Using openssl
openssl rand -base64 32

# Using Python
python -c "import secrets; print(secrets.token_urlsafe(32))"

# Using Node.js
node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
```

### Production Docker Compose

For production, create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  backend:
    environment:
      LUMINABRIDGE__LOGGING__LEVEL: warn
      LUMINABRIDGE__RATE_LIMIT__ENABLED: "true"
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '1'
          memory: 512M

  frontend:
    deploy:
      replicas: 2
      resources:
        limits:
          cpus: '0.5'
          memory: 256M
```

### Backup Strategy

```bash
# Backup PostgreSQL
docker-compose exec postgres pg_dump -U luminabridge luminabridge_dev > backup.sql

# Restore PostgreSQL
docker-compose exec -T postgres psql -U luminabridge -d luminabridge_dev < backup.sql

# Backup Redis
docker-compose exec redis redis-cli SAVE
```

---

## 🔧 Troubleshooting

### Common Issues

#### Backend fails to start

```bash
# Check logs
docker-compose logs backend

# Common causes:
# 1. Database not ready - wait for postgres health check
# 2. Invalid JWT_SECRET - ensure min 32 characters
# 3. Port conflict - change BACKEND_PORT in .env
```

#### Frontend can't connect to backend

```bash
# Verify network connectivity
docker-compose exec frontend ping backend

# Check backend is running
docker-compose ps backend

# Verify API proxy configuration in nginx.conf
```

#### Database connection issues

```bash
# Test database connection
docker-compose exec postgres pg_isready

# Check database logs
docker-compose logs postgres

# Verify credentials in .env match
```

#### Health checks failing

```bash
# Check health endpoint manually
curl http://localhost:3000/health

# Increase start_period in docker-compose.yml
# Check application logs for errors
```

### Reset Everything

```bash
# ⚠️ WARNING: This deletes all data!
docker-compose down -v
docker-compose build --no-cache
docker-compose up -d
```

### Performance Issues

```bash
# Check resource usage
docker stats

# Increase worker count
# Edit .env: BACKEND_WORKERS=4

# Enable Redis caching
# Ensure LUMINABRIDGE__CACHE__URL is set
```

---

## 📊 Monitoring

### Health Check Endpoints

| Endpoint | Description |
|----------|-------------|
| `/health` | Basic health check |
| `/ready` | Readiness check (includes DB) |

### Docker Commands

```bash
# View container stats
docker stats

# Inspect container
docker inspect luminabridge-backend

# View network
docker network inspect luminabridge-network
```

---

## 📝 Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Reference](https://docs.docker.com/compose/compose-file/)
- [LuminaBridge README](../README.md)
- [API Documentation](../docs/)

---

**Need help?** Check the main [README.md](../README.md) or open an issue on GitHub.
