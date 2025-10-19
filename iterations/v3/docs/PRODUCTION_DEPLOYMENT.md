# Multimodal RAG System - Production Deployment Guide

This guide provides comprehensive instructions for deploying the Multimodal RAG system to production.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Architecture Overview](#architecture-overview)
3. [Deployment Steps](#deployment-steps)
4. [Configuration](#configuration)
5. [Monitoring](#monitoring)
6. [Maintenance](#maintenance)
7. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **CPU**: 8+ cores recommended
- **RAM**: 16GB+ recommended
- **Storage**: 100GB+ SSD recommended
- **Network**: 1Gbps+ bandwidth

### Software Requirements

- Docker 20.10+
- Docker Compose 2.0+
- Git
- curl
- k6 (for load testing)

### Environment Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd agent-agency/iterations/v3
```

2. Copy environment configuration:
```bash
cp env.production.example .env.production
```

3. Update environment variables in `.env.production`:
```bash
# Edit the file with your production values
nano .env.production
```

## Architecture Overview

The production deployment includes the following services:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│     Nginx       │    │   Prometheus    │    │    Grafana      │
│  (Load Balancer)│    │   (Metrics)     │    │  (Dashboards)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Multimodal RAG  │    │   PostgreSQL    │    │     Redis       │
│    Service      │◄──►│  (with pgvector)│    │    (Cache)      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
┌─────────────────┐    ┌─────────────────┐
│  Elasticsearch  │    │     Kibana      │
│   (Logs)        │◄──►│ (Log Analysis)  │
└─────────────────┘    └─────────────────┘
```

## Deployment Steps

### 1. Automated Deployment

Use the provided deployment script:

```bash
# Make the script executable
chmod +x scripts/deploy-production.sh

# Deploy the system
./scripts/deploy-production.sh deploy
```

### 2. Manual Deployment

If you prefer manual deployment:

```bash
# 1. Build and start services
docker-compose -f docker/docker-compose.production.yml up -d

# 2. Wait for services to be ready
docker-compose -f docker/docker-compose.production.yml ps

# 3. Run database migrations
docker-compose -f docker/docker-compose.production.yml exec postgres psql -U multimodal_rag -d multimodal_rag -f /docker-entrypoint-initdb.d/001_enable_pgvector.sql
docker-compose -f docker/docker-compose.production.yml exec postgres psql -U multimodal_rag -d multimodal_rag -f /docker-entrypoint-initdb.d/002_create_vector_tables.sql

# 4. Verify deployment
curl http://localhost:8080/health
```

### 3. Load Testing

Run load tests to verify performance:

```bash
# Install k6 if not already installed
# On macOS: brew install k6
# On Ubuntu: sudo apt-get install k6

# Run load tests
cd load-testing
k6 run k6-multimodal-rag-test.js
```

## Configuration

### Environment Variables

Key environment variables to configure:

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_PASSWORD` | PostgreSQL password | `secure_password_123` |
| `REDIS_PASSWORD` | Redis password | `redis_secure_pass` |
| `JWT_SECRET` | JWT signing secret | `32_character_secret_key` |
| `API_KEY` | API authentication key | `api_key_12345` |
| `GRAFANA_PASSWORD` | Grafana admin password | `grafana_admin_pass` |

### Service Configuration

The main service configuration is in `config/production.yaml`. Key settings:

- **Database connection pooling**: Configured for production load
- **Rate limiting**: 1000 requests/minute with burst of 100
- **Caching**: Redis-based caching with 1-hour TTL
- **Logging**: Structured JSON logging
- **Monitoring**: Prometheus metrics enabled

### Security Configuration

- **Authentication**: JWT-based with configurable expiry
- **CORS**: Configured for production domains
- **Input validation**: Enabled with length limits
- **Rate limiting**: Per-IP rate limiting enabled

## Monitoring

### Accessing Monitoring Dashboards

- **Grafana**: http://localhost:3000 (admin/your_grafana_password)
- **Prometheus**: http://localhost:9090
- **Kibana**: http://localhost:5601
- **Service Metrics**: http://localhost:8081/metrics

### Key Metrics to Monitor

1. **Service Health**
   - HTTP response times (P95 < 2s)
   - Error rates (< 1%)
   - Request throughput

2. **Database Performance**
   - Connection pool usage
   - Query performance
   - Vector search latency

3. **Multimodal Processing**
   - Processing queue size
   - Embedding generation time
   - Cross-modal validation success rate

4. **System Resources**
   - CPU usage (< 80%)
   - Memory usage (< 80%)
   - Disk space (> 10% free)

### Alerting

Alerts are configured in `monitoring/multimodal_rag_rules.yml`:

- **Critical**: Service down, database unavailable, high error rate
- **Warning**: High response time, resource usage, queue backlog

## Maintenance

### Regular Maintenance Tasks

1. **Daily**
   - Check service health
   - Review error logs
   - Monitor resource usage

2. **Weekly**
   - Review performance metrics
   - Update dependencies
   - Clean up old logs

3. **Monthly**
   - Database maintenance
   - Security updates
   - Performance optimization

### Backup and Recovery

#### Creating Backups

```bash
# Manual backup
./scripts/deploy-production.sh backup

# Automated backup (configured in production.yaml)
# Runs daily at 2 AM
```

#### Restoring from Backup

```bash
# Stop services
docker-compose -f docker/docker-compose.production.yml down

# Restore database
docker-compose -f docker/docker-compose.production.yml up -d postgres
docker-compose -f docker/docker-compose.production.yml exec postgres psql -U multimodal_rag -d multimodal_rag < backup/database.sql

# Restore volumes (if needed)
docker run --rm -v multimodal-rag_postgres_data:/data -v $(pwd)/backup:/backup alpine tar xzf /backup/postgres_data.tar.gz -C /data

# Restart services
docker-compose -f docker/docker-compose.production.yml up -d
```

### Updates and Upgrades

1. **Code Updates**
```bash
# Pull latest changes
git pull origin main

# Rebuild and deploy
./scripts/deploy-production.sh deploy
```

2. **Database Migrations**
```bash
# Migrations run automatically during deployment
# Check migration status
docker-compose -f docker/docker-compose.production.yml exec postgres psql -U multimodal_rag -d multimodal_rag -c "SELECT * FROM migration_log ORDER BY applied_at DESC;"
```

## Troubleshooting

### Common Issues

#### 1. Service Won't Start

**Symptoms**: Container exits immediately
**Solutions**:
- Check logs: `docker-compose logs multimodal-rag-service`
- Verify environment variables
- Check port conflicts

#### 2. Database Connection Issues

**Symptoms**: "Connection refused" errors
**Solutions**:
- Verify PostgreSQL is running: `docker-compose ps postgres`
- Check database credentials
- Verify network connectivity

#### 3. High Memory Usage

**Symptoms**: OOM errors, slow performance
**Solutions**:
- Increase container memory limits
- Optimize vector search parameters
- Review caching configuration

#### 4. Slow Vector Search

**Symptoms**: High response times for search queries
**Solutions**:
- Check HNSW index configuration
- Optimize vector dimensions
- Review similarity thresholds

### Log Analysis

#### Service Logs
```bash
# View service logs
docker-compose -f docker/docker-compose.production.yml logs -f multimodal-rag-service

# View specific log levels
docker-compose -f docker/docker-compose.production.yml logs multimodal-rag-service | grep ERROR
```

#### Database Logs
```bash
# View PostgreSQL logs
docker-compose -f docker/docker-compose.production.yml logs postgres
```

#### System Logs
```bash
# View system resource usage
docker stats
```

### Performance Tuning

#### Database Optimization
- Adjust connection pool settings
- Optimize HNSW index parameters
- Configure query timeouts

#### Service Optimization
- Tune JVM settings (if applicable)
- Optimize batch processing sizes
- Configure caching strategies

#### System Optimization
- Adjust Docker resource limits
- Configure swap space
- Optimize network settings

## Support

For additional support:

1. Check the logs first
2. Review monitoring dashboards
3. Consult this documentation
4. Contact the development team

## Security Considerations

### Production Security Checklist

- [ ] Change all default passwords
- [ ] Enable SSL/TLS encryption
- [ ] Configure firewall rules
- [ ] Set up proper backup encryption
- [ ] Enable audit logging
- [ ] Regular security updates
- [ ] Monitor for security alerts
- [ ] Implement access controls

### Network Security

- Use reverse proxy (Nginx) for SSL termination
- Configure proper CORS policies
- Implement rate limiting
- Use secure communication between services

### Data Security

- Encrypt sensitive data at rest
- Use secure key management
- Implement data retention policies
- Regular security audits
