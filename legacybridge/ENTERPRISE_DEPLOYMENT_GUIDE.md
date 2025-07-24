# LegacyBridge Enterprise Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying LegacyBridge Enterprise Edition to support 1000+ concurrent users with high availability, multi-tenancy, and enterprise-grade security.

## Table of Contents

- [Architecture Overview](#architecture-overview)
  - [Multi-Tenant Architecture](#multi-tenant-architecture)
  - [Component Overview](#component-overview)
- [Prerequisites](#prerequisites)
  - [Hardware Requirements](#hardware-requirements)
  - [Software Requirements](#software-requirements)
  - [Network Requirements](#network-requirements)
- [Infrastructure Setup](#infrastructure-setup)
  - [Cloud Provider Setup](#cloud-provider-setup)
  - [On-Premise Setup](#on-premise-setup)
- [Database Configuration](#database-configuration)
  - [PostgreSQL Setup](#postgresql-setup)
  - [Redis Setup](#redis-setup)
- [Application Deployment](#application-deployment)
  - [Docker Deployment](#docker-deployment)
  - [Kubernetes Deployment](#kubernetes-deployment)
- [Load Balancer Configuration](#load-balancer-configuration)
  - [NGINX Configuration](#nginx-configuration)
  - [Cloud Load Balancers](#cloud-load-balancers)
- [Security Configuration](#security-configuration)
  - [SSL/TLS Setup](#ssltls-setup)
  - [Firewall Rules](#firewall-rules)
  - [Authentication Setup](#authentication-setup)
- [Monitoring and Observability](#monitoring-and-observability)
  - [Prometheus Setup](#prometheus-setup)
  - [Grafana Dashboards](#grafana-dashboards)
  - [Log Aggregation](#log-aggregation)
- [Backup and Disaster Recovery](#backup-and-disaster-recovery)
  - [Backup Strategy](#backup-strategy)
  - [Recovery Procedures](#recovery-procedures)
- [Performance Tuning](#performance-tuning)
  - [Application Tuning](#application-tuning)
  - [Database Optimization](#database-optimization)
- [Troubleshooting](#troubleshooting)
  - [Common Issues](#common-issues)
  - [Diagnostic Tools](#diagnostic-tools)

## Architecture Overview

### Multi-Tenant Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                        Load Balancer                         │
│                     (NGINX/AWS ALB/GCP LB)                  │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────┴───────────────┐
        │                               │
┌───────▼────────┐             ┌───────▼────────┐
│   Web Server   │             │   Web Server   │
│   (Node.js)    │   ......    │   (Node.js)    │
│   Instance 1   │             │   Instance N   │
└───────┬────────┘             └───────┬────────┘
        │                               │
        └───────────────┬───────────────┘
                        │
        ┌───────────────┴───────────────┐
        │                               │
┌───────▼────────┐             ┌───────▼────────┐
│  Redis Cluster │             │   PostgreSQL    │
│  (Sessions &   │             │    Cluster      │
│   Queue)       │             │  (Primary +     │
└────────────────┘             │   Replicas)    │
                               └─────────────────┘
```

### Component Overview

1. **Load Balancer**: Distributes traffic across application instances
2. **Application Servers**: Stateless Node.js instances running LegacyBridge
3. **PostgreSQL Cluster**: Primary database with read replicas
4. **Redis Cluster**: Session storage and job queue management
5. **File Storage**: S3-compatible storage for documents
6. **CDN**: Content delivery for static assets

## Prerequisites

### Hardware Requirements

#### Minimum Requirements (100-500 users)
- **Application Servers**: 3 instances, 4 vCPU, 8GB RAM each
- **Database Server**: 8 vCPU, 32GB RAM, 500GB SSD
- **Redis Server**: 4 vCPU, 16GB RAM
- **Load Balancer**: 2 vCPU, 4GB RAM

#### Recommended Requirements (500-1000+ users)
- **Application Servers**: 5-10 instances, 8 vCPU, 16GB RAM each
- **Database Cluster**: Primary + 2 replicas, 16 vCPU, 64GB RAM, 1TB SSD each
- **Redis Cluster**: 3 nodes, 8 vCPU, 32GB RAM each
- **Load Balancer**: Managed service (AWS ALB, GCP LB, etc.)

### Software Requirements
- **OS**: Ubuntu 22.04 LTS or RHEL 8+
- **Node.js**: 18.x or 20.x LTS
- **PostgreSQL**: 15.x
- **Redis**: 7.x
- **Docker**: 24.x (for containerized deployment)
- **Kubernetes**: 1.28+ (for orchestrated deployment)

## Infrastructure Setup

### 1. Network Configuration

```bash
# Create VPC (AWS example)
aws ec2 create-vpc --cidr-block 10.0.0.0/16

# Create subnets
aws ec2 create-subnet --vpc-id vpc-xxx --cidr-block 10.0.1.0/24 --availability-zone us-east-1a
aws ec2 create-subnet --vpc-id vpc-xxx --cidr-block 10.0.2.0/24 --availability-zone us-east-1b

# Create security groups
aws ec2 create-security-group --group-name legacybridge-app --description "Application servers"
aws ec2 create-security-group --group-name legacybridge-db --description "Database servers"
```

### 2. SSL/TLS Certificates

```bash
# Using Let's Encrypt
sudo certbot certonly --standalone -d legacybridge.com -d *.legacybridge.com

# Or using AWS Certificate Manager
aws acm request-certificate \
  --domain-name legacybridge.com \
  --subject-alternative-names "*.legacybridge.com" \
  --validation-method DNS
```

## Database Configuration

### 1. PostgreSQL Setup

```bash
# Install PostgreSQL 15
sudo sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -
sudo apt-get update
sudo apt-get install postgresql-15 postgresql-contrib-15

# Configure PostgreSQL for production
sudo nano /etc/postgresql/15/main/postgresql.conf
```

**postgresql.conf optimizations:**
```conf
# Connection settings
max_connections = 1000
shared_buffers = 16GB  # 25% of RAM
effective_cache_size = 48GB  # 75% of RAM

# Write performance
checkpoint_segments = 32
checkpoint_completion_target = 0.9
wal_buffers = 16MB

# Query performance
work_mem = 32MB
maintenance_work_mem = 2GB
random_page_cost = 1.1  # For SSD storage

# Logging
log_min_duration_statement = 1000  # Log slow queries
log_checkpoints = on
log_connections = on
log_disconnections = on
```

### 2. Create Database and Schema

```bash
# Connect to PostgreSQL
sudo -u postgres psql

# Create database and user
CREATE DATABASE legacybridge;
CREATE USER legacybridge_app WITH ENCRYPTED PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE legacybridge TO legacybridge_app;

# Apply enterprise schema
\c legacybridge
\i /path/to/src/enterprise/database/schema.sql
```

### 3. Setup Replication

**On Primary:**
```bash
# Configure primary for replication
echo "host replication replicator 10.0.0.0/16 md5" >> /etc/postgresql/15/main/pg_hba.conf

# Create replication user
CREATE USER replicator WITH REPLICATION ENCRYPTED PASSWORD 'repl_password';

# Restart PostgreSQL
sudo systemctl restart postgresql
```

**On Replica:**
```bash
# Stop PostgreSQL
sudo systemctl stop postgresql

# Clear data directory
sudo rm -rf /var/lib/postgresql/15/main/*

# Create base backup
sudo -u postgres pg_basebackup -h primary_ip -D /var/lib/postgresql/15/main -U replicator -v -P -W

# Configure replica
echo "standby_mode = 'on'
primary_conninfo = 'host=primary_ip port=5432 user=replicator password=repl_password'
trigger_file = '/tmp/postgresql.trigger'" > /var/lib/postgresql/15/main/recovery.conf

# Start PostgreSQL
sudo systemctl start postgresql
```

## Application Deployment

### 1. Build Application

```bash
# Clone repository
git clone https://github.com/your-org/legacybridge.git
cd legacybridge

# Install dependencies
npm install

# Build for production
npm run build

# Build Docker image
docker build -t legacybridge:latest .
```

### 2. Docker Deployment

**Dockerfile:**
```dockerfile
FROM node:20-alpine

WORKDIR /app

# Copy package files
COPY package*.json ./
RUN npm ci --only=production

# Copy application
COPY . .

# Build application
RUN npm run build

# Create non-root user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nodejs -u 1001
USER nodejs

EXPOSE 3000

CMD ["npm", "start"]
```

### 3. Kubernetes Deployment

```bash
# Create namespace
kubectl create namespace legacybridge

# Apply configurations
kubectl apply -f src/enterprise/config/kubernetes-deployment.yaml

# Check deployment status
kubectl get pods -n legacybridge
kubectl get services -n legacybridge
```

### 4. Environment Configuration

Create `.env.production`:
```env
# Application
NODE_ENV=production
APP_NAME=LegacyBridge Enterprise
PORT=3000

# Database
DATABASE_URL=postgresql://legacybridge_app:password@postgres-cluster:5432/legacybridge
DATABASE_POOL_MIN=10
DATABASE_POOL_MAX=100

# Redis
REDIS_URL=redis://redis-cluster:6379
REDIS_PASSWORD=redis_password

# Authentication
JWT_SECRET=your-super-secret-jwt-key-change-this
JWT_EXPIRES_IN=1h
REFRESH_TOKEN_EXPIRES_IN=7d

# File Storage
STORAGE_TYPE=s3
AWS_BUCKET_NAME=legacybridge-files
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key

# Security
ENCRYPTION_KEY=your-32-character-encryption-key
CORS_ORIGINS=https://legacybridge.com,https://*.legacybridge.com

# Rate Limiting
RATE_LIMIT_WINDOW_MS=60000
RATE_LIMIT_MAX_REQUESTS=100

# Monitoring
ENABLE_METRICS=true
METRICS_PORT=9090
LOG_LEVEL=info
```

## Load Balancer Configuration

### NGINX Configuration

```nginx
upstream legacybridge {
    least_conn;
    server app1.internal:3000 max_fails=3 fail_timeout=30s;
    server app2.internal:3000 max_fails=3 fail_timeout=30s;
    server app3.internal:3000 max_fails=3 fail_timeout=30s;
    keepalive 32;
}

server {
    listen 80;
    server_name legacybridge.com *.legacybridge.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name legacybridge.com *.legacybridge.com;

    ssl_certificate /etc/letsencrypt/live/legacybridge.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/legacybridge.com/privkey.pem;
    
    # SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;
    
    # Timeouts
    proxy_connect_timeout 30s;
    proxy_send_timeout 300s;
    proxy_read_timeout 300s;
    
    # File upload size
    client_max_body_size 1000M;
    
    location / {
        proxy_pass http://legacybridge;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
    
    location /health {
        access_log off;
        proxy_pass http://legacybridge/health;
    }
}
```

## Security Configuration

### 1. Firewall Rules

```bash
# Application servers
sudo ufw allow from 10.0.0.0/16 to any port 3000
sudo ufw allow from 10.0.0.0/16 to any port 9090

# Database servers
sudo ufw allow from 10.0.1.0/24 to any port 5432

# Redis servers
sudo ufw allow from 10.0.1.0/24 to any port 6379
```

### 2. SSL/TLS Configuration

```bash
# Generate strong DH parameters
openssl dhparam -out /etc/ssl/certs/dhparam.pem 4096

# Configure SSL in application
export NODE_OPTIONS="--tls-min-v1.2"
```

### 3. Security Best Practices

1. **Enable MFA for all admin users**
2. **Implement IP whitelisting for admin access**
3. **Regular security audits and penetration testing**
4. **Encrypted backups with key rotation**
5. **Network segmentation between tiers**
6. **Regular dependency updates**

## Monitoring and Observability

### 1. Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'legacybridge'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - legacybridge
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: legacybridge
```

### 2. Grafana Dashboards

Import the following dashboards:
- Application metrics: `dashboards/legacybridge-app.json`
- Database metrics: `dashboards/postgres-metrics.json`
- Redis metrics: `dashboards/redis-metrics.json`

### 3. Alerting Rules

```yaml
# alerts.yml
groups:
  - name: legacybridge
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High error rate detected
          
      - alert: DatabaseConnectionPoolExhausted
        expr: pg_stat_database_numbackends / pg_settings_max_connections > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: Database connection pool nearly exhausted
```

## Backup and Disaster Recovery

### 1. Database Backups

```bash
# Daily backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/postgres"
DB_NAME="legacybridge"

# Create backup
pg_dump -h localhost -U legacybridge_app -d $DB_NAME | gzip > $BACKUP_DIR/backup_$DATE.sql.gz

# Upload to S3
aws s3 cp $BACKUP_DIR/backup_$DATE.sql.gz s3://legacybridge-backups/postgres/

# Clean old backups (keep 30 days)
find $BACKUP_DIR -name "*.sql.gz" -mtime +30 -delete
```

### 2. File Storage Backup

```bash
# Sync to backup bucket
aws s3 sync s3://legacybridge-files s3://legacybridge-backups/files/ --delete
```

### 3. Disaster Recovery Plan

1. **RTO (Recovery Time Objective)**: 4 hours
2. **RPO (Recovery Point Objective)**: 1 hour

**Recovery Steps:**
1. Provision new infrastructure from terraform/CloudFormation templates
2. Restore database from latest backup
3. Restore file storage from S3 backup
4. Update DNS to point to new infrastructure
5. Verify system functionality

## Performance Tuning

### 1. Application Tuning

```javascript
// Cluster mode for multi-core utilization
const cluster = require('cluster');
const numCPUs = require('os').cpus().length;

if (cluster.isMaster) {
  for (let i = 0; i < numCPUs; i++) {
    cluster.fork();
  }
} else {
  // Start application
  app.listen(3000);
}
```

### 2. Database Tuning

```sql
-- Create indexes for common queries
CREATE INDEX idx_conversion_jobs_org_status ON conversion_jobs(organization_id, status);
CREATE INDEX idx_audit_logs_org_timestamp ON audit_logs(organization_id, timestamp DESC);

-- Analyze tables regularly
ANALYZE conversion_jobs;
ANALYZE audit_logs;
```

### 3. Redis Tuning

```conf
# redis.conf
maxmemory 16gb
maxmemory-policy allkeys-lru
save ""  # Disable persistence for cache
```

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Check for memory leaks: `node --inspect app.js`
   - Monitor with: `pm2 monit`
   - Solution: Increase memory limits or optimize code

2. **Slow Database Queries**
   - Enable slow query log
   - Run `EXPLAIN ANALYZE` on slow queries
   - Add appropriate indexes

3. **Connection Pool Exhaustion**
   - Increase pool size in configuration
   - Check for connection leaks
   - Implement connection timeout

4. **File Upload Failures**
   - Check file size limits in NGINX and application
   - Verify S3 permissions
   - Check available disk space

### Health Checks

```bash
# Application health
curl https://legacybridge.com/health

# Database health
psql -h localhost -U legacybridge_app -c "SELECT 1"

# Redis health
redis-cli ping

# Full system check
./scripts/health-check.sh
```

## Support

For enterprise support:
- Email: enterprise@legacybridge.com
- Phone: +1-800-LEGACY-1
- Slack: legacybridge-enterprise.slack.com

For urgent issues, use the escalation process defined in your SLA.