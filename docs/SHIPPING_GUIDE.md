# 🚀 Shipping Guide for Context Server RS

## Quick Deployment (5 minutes)

### Option 1: Docker Compose (Recommended)
```bash
# Clone and deploy
git clone <your-repo>
cd context-server-rs
docker-compose up -d

# Server ready at stdio/port 3000
```

### Option 2: Direct Binary
```bash
# Build release
cargo build --release

# Run server
./target/release/context-server-rs

# Or install globally
cargo install --path .
context-server-rs
```

## Distribution Methods

### 1. Container Registry
```bash
# Build and push
docker build -t your-registry/context-server-rs:v0.2.0 .
docker push your-registry/context-server-rs:v0.2.0

# Deploy anywhere
docker run -d -p 3000:3000 your-registry/context-server-rs:v0.2.0
```

### 2. GitHub Releases
```bash
# Automated via GitHub Actions
git tag v0.2.0
git push origin v0.2.0

# Downloads available at:
# https://github.com/your-username/context-server-rs/releases
```

### 3. Cargo Registry
```bash
# Publish to crates.io
cargo publish

# Users install with:
cargo install context-server-rs
```

## Cloud Deployment

### AWS ECS/Fargate
```yaml
# Use provided ecs-task-definition.json
aws ecs create-service \
  --cluster production \
  --service-name context-server \
  --task-definition context-server:1
```

### Google Cloud Run
```bash
# Deploy serverless
gcloud run deploy context-server \
  --image gcr.io/your-project/context-server-rs \
  --platform managed \
  --allow-unauthenticated
```

### Azure Container Instances
```bash
# Deploy container
az container create \
  --resource-group production \
  --name context-server \
  --image your-registry/context-server-rs:v0.2.0
```

### DigitalOcean App Platform
```yaml
# Use provided .do/app.yaml
doctl apps create --spec .do/app.yaml
```

## Enterprise Deployment

### Kubernetes
```bash
# Apply manifests
kubectl apply -f deployment/
kubectl apply -f service/
kubectl apply -f ingress/

# Monitor deployment
kubectl rollout status deployment/context-server
```

### Docker Swarm
```bash
# Deploy stack
docker stack deploy -c docker-compose.prod.yml context-server

# Scale services
docker service scale context-server_api=3
```

## Configuration

### Environment Variables
```bash
# Production settings
export RUST_LOG=info
export DATABASE_URL=sqlite:///data/context.db
export SERVER_HOST=0.0.0.0
export SERVER_PORT=3000
export ENABLE_METRICS=true
```

### Configuration File
```yaml
# config/production.yml
server:
  host: "0.0.0.0"
  port: 3000
database:
  url: "sqlite:///data/context.db"
  pool_size: 10
logging:
  level: "info"
  format: "json"
```

## Monitoring Setup

### Prometheus Metrics
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'context-server'
    static_configs:
      - targets: ['context-server:3000']
```

### Grafana Dashboard
```bash
# Import dashboard
grafana-cli plugins install grafana-piechart-panel
# Import dashboards/context-server.json
```

### Log Aggregation
```yaml
# fluentd/fluent.conf
<source>
  @type forward
  port 24224
</source>
<match context-server.**>
  @type elasticsearch
  host elasticsearch
  port 9200
</match>
```

## Security Hardening

### TLS Configuration
```yaml
# nginx.conf
server {
    listen 443 ssl;
    ssl_certificate /etc/ssl/certs/server.crt;
    ssl_certificate_key /etc/ssl/private/server.key;
    
    location / {
        proxy_pass http://context-server:3000;
    }
}
```

### Authentication
```rust
// Add authentication middleware
#[tokio::main]
async fn main() {
    let app = Router::new()
        .layer(AuthLayer::new())
        .route("/", get(handler));
}
```

## Backup Strategy

### Database Backup
```bash
# Automated backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
sqlite3 context.db ".backup backup_$DATE.db"
aws s3 cp backup_$DATE.db s3://backups/context-server/
```

### Volume Backup
```yaml
# docker-compose with backup volumes
volumes:
  data:
    driver: local
    driver_opts:
      type: none
      device: /backup/context-server
      o: bind
```

## Performance Tuning

### Database Optimization
```sql
-- SQLite pragmas for production
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = 10000;
PRAGMA temp_store = memory;
```

### Resource Limits
```yaml
# docker-compose.yml
services:
  context-server:
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '1.0'
        reservations:
          memory: 256M
          cpus: '0.5'
```

## Scaling Strategy

### Horizontal Scaling
```yaml
# Scale replicas
docker-compose up --scale context-server=3

# Load balancer
nginx:
  image: nginx:alpine
  volumes:
    - ./nginx.conf:/etc/nginx/nginx.conf
```

### Database Scaling
```bash
# Migrate to PostgreSQL for high load
docker run -d postgres:15
# Update connection string
export DATABASE_URL=postgresql://user:pass@localhost/context
```

## Health Checks

### Application Health
```bash
# Built-in health endpoint
curl http://localhost:3000/health

# Response
{"status": "healthy", "database": "connected"}
```

### Container Health
```dockerfile
# Dockerfile health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1
```

## Troubleshooting

### Common Issues
```bash
# Check logs
docker logs context-server-rs

# Database connection
sqlite3 context.db ".tables"

# Port binding
netstat -tulpn | grep 3000
```

### Debug Mode
```bash
# Enable debug logging
export RUST_LOG=debug
./context-server-rs

# Trace network issues
export RUST_LOG=rmcp=trace
```

## Update Strategy

### Rolling Updates
```bash
# Build new image
docker build -t context-server:v0.3.0 .

# Update service
docker service update --image context-server:v0.3.0 context-server_api
```

### Blue-Green Deployment
```yaml
# deploy blue environment
docker-compose -f docker-compose.blue.yml up -d

# switch traffic
# update load balancer configuration

# clean up green environment
docker-compose -f docker-compose.green.yml down
```

## Support & Maintenance

### Log Rotation
```bash
# logrotate configuration
/var/log/context-server/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    create 0644 app app
}
```

### Automated Updates
```yaml
# renovate.json for dependency updates
{
  "extends": ["config:base"],
  "packageRules": [
    {
      "updateTypes": ["minor", "patch"],
      "automerge": true
    }
  ]
}
```

---

## 🎯 Ready to Ship!

Your MCP Context Server is production-ready with:
- ✅ Multiple deployment options
- ✅ Comprehensive monitoring
- ✅ Security hardening
- ✅ Scaling strategies
- ✅ Backup procedures
- ✅ Update mechanisms

Choose your deployment method and ship it! 🚀
