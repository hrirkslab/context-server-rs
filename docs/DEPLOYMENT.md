# Production Deployment Guide

## Prerequisites

- Docker and Docker Compose
- OR Rust 1.76+ for native compilation

## Quick Start with Docker

1. **Clone and deploy:**
   ```bash
   git clone https://github.com/hrirkslab/context-server-rs.git
   cd context-server-rs
   docker-compose up -d
   ```

2. **Check health:**
   ```bash
   docker-compose logs context-server
   curl -X POST -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"ping"}' \
     http://localhost:3000
   ```

## Native Deployment

1. **Build for production:**
   ```bash
   cargo build --release
   ```

2. **Run with systemd service:**
   ```bash
   sudo cp target/release/context-server-rs /usr/local/bin/
   sudo cp deploy/context-server.service /etc/systemd/system/
   sudo systemctl enable context-server
   sudo systemctl start context-server
   ```

## Environment Configuration

Create `.env` file:
```env
RUST_LOG=info
DATABASE_PATH=/app/config/context.db
MCP_HOST=0.0.0.0
MCP_PORT=3000
```

## Security Considerations

- Run as non-root user
- Use TLS in production
- Configure firewall rules
- Regular security updates
- Database backups

## Monitoring

- Health check endpoint: `/health`
- Metrics endpoint: `/metrics`
- Logs via systemd journal or Docker logs

## Scaling

- Use read replicas for SQLite
- Consider PostgreSQL for high load
- Load balance multiple instances
- Container orchestration with Kubernetes

## Backup Strategy

```bash
# Backup database
cp /app/config/context.db /backup/context-$(date +%Y%m%d).db

# Automated backup script
0 2 * * * /usr/local/bin/backup-context-db.sh
```
