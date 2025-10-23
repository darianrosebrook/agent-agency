# Agent Agency V3 - Production Deployment Guide

## Overview

This guide covers the complete production deployment of Agent Agency V3, a production-ready autonomous AI development platform. The system includes constitutional AI governance, autonomous execution, and comprehensive safety controls.

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Dashboard │    │   REST API      │    │   CLI Tools     │
│   (Next.js)     │    │   (Axum)        │    │   (Rust)        │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                     │                      │
          └─────────────────────┼──────────────────────┘
                                │
                   ┌────────────▼────────────┐
                   │   Orchestration Layer   │
                   │   - Arbiter Council     │
                   │   - Task Scheduling     │
                   │   - Resource Mgmt      │
                   └────────────┬────────────┘
                                │
                   ┌────────────▼────────────┐
                   │   Execution Layer      │
                   │   - Self-Prompting     │
                   │   - File Operations    │
                   │   - Quality Gates      │
                   └────────────┬────────────┘
                                │
                   ┌────────────▼────────────┐
                   │   Data Layer           │
                   │   - PostgreSQL         │
                   │   - Redis Cache        │
                   │   - File Storage       │
                   └─────────────────────────┘
```

## Prerequisites

### System Requirements
- **CPU**: 8+ cores (16+ recommended)
- **RAM**: 32GB+ (64GB recommended)
- **Storage**: 500GB+ SSD
- **Network**: 1Gbps+ bandwidth

### Software Dependencies
- **Docker**: 24.0+
- **Docker Compose**: 2.20+
- **Kubernetes**: 1.27+ (for production cluster)
- **PostgreSQL**: 15+
- **Redis**: 7+
- **Nginx**: 1.24+ (for load balancing)

### Development Dependencies
- **Rust**: 1.75+
- **Node.js**: 18+
- **pnpm**: 8+
- **CAWS CLI**: Latest version

## Deployment Options

### Option 1: Docker Compose (Development/Staging)

Perfect for development, staging, and small production deployments.

#### Quick Start

```bash
# Clone the repository
git clone https://github.com/your-org/agent-agency.git
cd agent-agency/iterations/v3

# Start the system
docker-compose -f docker/docker-compose.production.yml up -d

# Check health
curl http://localhost:3000/health
```

#### Configuration

Create `.env.production`:

```bash
# Database
DATABASE_URL=postgresql://agent_agency:password@postgres:5432/agent_agency
REDIS_URL=redis://redis:6379

# Security
JWT_SECRET=your-super-secure-jwt-secret-here
API_KEY=your-api-key-here

# External Services
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Arbiter Configuration
ARBITER_COUNCIL_SIZE=3
ARBITER_DEBATE_ROUNDS=2

# Execution Limits
MAX_CONCURRENT_TASKS=5
TASK_TIMEOUT_SECONDS=600
```

### Option 2: Kubernetes (Production)

For enterprise-scale production deployments.

#### Helm Chart Deployment

```bash
# Add the Helm repository
helm repo add agent-agency https://charts.agent-agency.dev
helm repo update

# Install with custom values
helm install agent-agency agent-agency/agent-agency \
  --namespace agent-agency \
  --create-namespace \
  --values values-production.yaml
```

#### Custom Values Example

```yaml
# values-production.yaml
global:
  env: production

database:
  postgresql:
    enabled: true
    auth:
      postgresPassword: "secure-password"
      username: "agent_agency"
      password: "secure-password"
      database: "agent_agency"

redis:
  enabled: true
  auth:
    password: "secure-redis-password"

arbiter:
  councilSize: 5
  debateRounds: 3
  confidenceThreshold: 0.85

execution:
  maxConcurrentTasks: 20
  taskTimeoutSeconds: 1800
  enableRollback: true

ingress:
  enabled: true
  className: "nginx"
  hosts:
    - host: agent-agency.your-domain.com
      paths:
        - path: /
          pathType: Exact
```

## Service Configuration

### API Service (Rust/Axum)

```yaml
# api-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agent-agency-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: agent-agency-api
  template:
    metadata:
      labels:
        app: agent-agency-api
    spec:
      containers:
      - name: api
        image: agent-agency/api:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: agent-agency-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: agent-agency-secrets
              key: redis-url
        resources:
          requests:
            cpu: 1000m
            memory: 2Gi
          limits:
            cpu: 2000m
            memory: 4Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
```

### Web Dashboard (Next.js)

```yaml
# dashboard-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agent-agency-dashboard
spec:
  replicas: 2
  selector:
    matchLabels:
      app: agent-agency-dashboard
  template:
    metadata:
      labels:
        app: agent-agency-dashboard
    spec:
      containers:
      - name: dashboard
        image: agent-agency/dashboard:latest
        ports:
        - containerPort: 3001
        env:
        - name: NEXT_PUBLIC_API_URL
          value: "https://api.agent-agency.your-domain.com"
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 1000m
            memory: 2Gi
```

### Worker Pool (Rust)

```yaml
# worker-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agent-agency-worker
spec:
  replicas: 5
  selector:
    matchLabels:
      app: agent-agency-worker
  template:
    metadata:
      labels:
        app: agent-agency-worker
    spec:
      containers:
      - name: worker
        image: agent-agency/worker:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: agent-agency-secrets
              key: database-url
        - name: WORKER_POOL_SIZE
          value: "10"
        resources:
          requests:
            cpu: 2000m
            memory: 4Gi
          limits:
            cpu: 4000m
            memory: 8Gi
```

## Database Setup

### PostgreSQL Schema

```sql
-- Create database and user
CREATE DATABASE agent_agency;
CREATE USER agent_agency WITH ENCRYPTED PASSWORD 'secure-password';
GRANT ALL PRIVILEGES ON DATABASE agent_agency TO agent_agency;

-- Enable extensions
\c agent_agency;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
```

### Initial Migrations

```bash
# Run migrations
cd iterations/v3
sqlx migrate run
```

## Monitoring & Observability

### Prometheus Metrics

The system exposes metrics at `/metrics` endpoint:

```yaml
# prometheus-config.yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'agent-agency-api'
    static_configs:
      - targets: ['api:3000']
    metrics_path: '/metrics'

  - job_name: 'agent-agency-worker'
    static_configs:
      - targets: ['worker:9090']
    metrics_path: '/metrics'
```

### Grafana Dashboards

Key metrics to monitor:
- **Task Success Rate**: `rate(agent_agency_tasks_completed_total[5m]) / rate(agent_agency_tasks_started_total[5m])`
- **Arbiter Confidence**: `histogram_quantile(0.95, rate(agent_agency_arbiter_confidence_bucket[5m]))`
- **Execution Time**: `histogram_quantile(0.95, rate(agent_agency_task_duration_seconds_bucket[5m]))`
- **Error Rate**: `rate(agent_agency_errors_total[5m])`

### Logging

```yaml
# fluent-bit-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluent-bit-config
data:
  fluent-bit.conf: |
    [INPUT]
        Name              tail
        Path              /var/log/containers/*agent-agency*.log
        Parser            docker
        Tag               agent-agency.*
        Refresh_Interval  5

    [OUTPUT]
        Name  elasticsearch
        Host  elasticsearch
        Port  9200
        Index agent-agency
```

## Security Configuration

### Network Policies

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: agent-agency-network-policy
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: agent-agency
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 3000
    - protocol: TCP
      port: 3001
  egress:
  - to:
    - podSelector:
        matchLabels:
          app.kubernetes.io/name: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - podSelector:
        matchLabels:
          app.kubernetes.io/name: redis
    ports:
    - protocol: TCP
      port: 6379
```

### Secrets Management

```yaml
# secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: agent-agency-secrets
type: Opaque
data:
  database-url: <base64-encoded-connection-string>
  redis-url: <base64-encoded-redis-url>
  jwt-secret: <base64-encoded-jwt-secret>
  openai-api-key: <base64-encoded-api-key>
  anthropic-api-key: <base64-encoded-api-key>
```

## Scaling & Performance Tuning

### Horizontal Pod Autoscaling

```yaml
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: agent-agency-api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: agent-agency-api
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Resource Optimization

```yaml
# resource-tuning
# API Service
cpu: 1000m-2000m
memory: 2Gi-4Gi

# Workers
cpu: 2000m-4000m
memory: 4Gi-8Gi

# Dashboard
cpu: 500m-1000m
memory: 1Gi-2Gi
```

## Backup & Disaster Recovery

### Database Backups

```yaml
# backup-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: agent-agency-backup
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:15
            command:
            - pg_dump
            - -h
            - postgres
            - -U
            - agent_agency
            - -d
            - agent_agency
            - >
            - /backup/agent-agency-$(date +%Y%m%d-%H%M%S).sql
            env:
            - name: PGPASSWORD
              valueFrom:
                secretKeyRef:
                  name: agent-agency-secrets
                  key: database-password
            volumeMounts:
            - name: backup-volume
              mountPath: /backup
          volumes:
          - name: backup-volume
            persistentVolumeClaim:
              claimName: backup-pvc
          restartPolicy: OnFailure
```

### Disaster Recovery

1. **Database Failover**: PostgreSQL streaming replication
2. **Application Failover**: Multi-region Kubernetes clusters
3. **Data Recovery**: Point-in-time recovery with WAL archives
4. **Service Mesh**: Istio for intelligent traffic routing

## Testing Production Deployment

### Health Checks

```bash
# API Health
curl -f https://api.agent-agency.your-domain.com/health

# Dashboard Health
curl -f https://dashboard.agent-agency.your-domain.com/api/health

# Database Connectivity
kubectl exec -it postgres-pod -- psql -U agent_agency -d agent_agency -c "SELECT 1"
```

### Load Testing

```bash
# Install k6
brew install k6

# Run load test
k6 run --vus 10 --duration 5m load-test.js
```

### Integration Testing

```bash
# Run E2E test suite
./scripts/run-e2e-tests.sh

# Check test results
cat test-results/e2e-report-*.txt
```

## Maintenance & Operations

### Regular Tasks

1. **Daily**: Monitor system health and performance metrics
2. **Weekly**: Review arbiter decisions and system accuracy
3. **Monthly**: Update dependencies and security patches
4. **Quarterly**: Performance optimization and scaling review

### Troubleshooting

#### Common Issues

1. **High Arbiter Confidence but Task Failures**
   - Check claim extraction accuracy
   - Review arbiter council configuration
   - Validate evidence collection

2. **Slow Task Execution**
   - Monitor worker resource utilization
   - Check database query performance
   - Review external API rate limits

3. **Memory Leaks**
   - Monitor Rust application memory usage
   - Check for unbounded data structures
   - Review file operation cleanup

#### Debug Commands

```bash
# Check pod logs
kubectl logs -f deployment/agent-agency-api

# Check database connections
kubectl exec -it postgres-pod -- psql -U agent_agency -d agent_agency -c "SELECT * FROM pg_stat_activity"

# Check Redis stats
kubectl exec -it redis-pod -- redis-cli info

# Run diagnostics
kubectl exec -it agent-agency-api-pod -- ./diagnostics
```

## Compliance & Security

### SOC 2 Type II Requirements

- [ ] Access Controls: RBAC implementation
- [ ] Audit Logging: Complete event tracking
- [ ] Data Encryption: At rest and in transit
- [ ] Change Management: Controlled deployments
- [ ] Incident Response: 24/7 monitoring and alerting

### GDPR Compliance

- [ ] Data Minimization: Only collect necessary data
- [ ] Consent Management: User permission handling
- [ ] Right to Deletion: Data removal procedures
- [ ] Data Portability: Export user data
- [ ] Breach Notification: Automated alerting

## Support & Documentation

### User Documentation
- **API Reference**: Complete OpenAPI specification
- **CLI Guide**: Command-line usage and examples
- **Dashboard Guide**: Web interface walkthrough
- **Integration Guide**: Third-party integration examples

### Operational Runbooks
- **Incident Response**: Step-by-step incident handling
- **Capacity Planning**: Scaling procedures and thresholds
- **Backup Recovery**: Disaster recovery procedures
- **Security Incidents**: Breach response and forensics

---

## Ready for Production! 🚀

This deployment guide provides everything needed to deploy Agent Agency V3 in production environments. The system is designed for enterprise-scale operations with comprehensive monitoring, security, and reliability features.

For questions or support, contact the development team or refer to the operational runbooks.
