# Agent Agency V3 - Production Deployment Guide

**Status**: ✅ Production Ready | **Last Updated**: October 23, 2025

## Overview

This guide covers the complete production deployment of Agent Agency V3, including infrastructure setup, security hardening, monitoring, and operational procedures.

## Prerequisites

### Infrastructure Requirements

- **AWS Account** with appropriate permissions
- **Route 53** hosted zone for DNS
- **ACM** for SSL certificates
- **EKS Cluster** (v1.24+) with at least 3 nodes
- **RDS PostgreSQL** (v15+) or Aurora PostgreSQL
- **ElastiCache Redis** or self-hosted Redis
- **S3 Bucket** for backups and artifacts

### Tooling Requirements

- **AWS CLI v2.7+**
- **kubectl v1.24+**
- **kustomize v4.5+**
- **helm v3.9+**
- **terraform v1.3+**
- **docker v20.10+**

### Network Requirements

- **VPC** with private subnets across 3 AZs
- **NAT Gateway** for outbound internet access
- **Security Groups** configured for service communication
- **Load Balancer** (ALB/NLB) for external traffic

## Infrastructure Setup

### 1. Terraform Infrastructure

```bash
cd deploy/terraform/aws

# Initialize Terraform
terraform init

# Plan the infrastructure
terraform plan -var-file=production.tfvars

# Apply the infrastructure
terraform apply -var-file=production.tfvars
```

**Key Components Created:**
- EKS Cluster with managed node groups
- RDS PostgreSQL database
- ElastiCache Redis cluster
- S3 buckets for backups and artifacts
- VPC with security groups
- IAM roles for service accounts

### 2. Kubernetes Cluster Setup

```bash
# Update kubeconfig
aws eks update-kubeconfig --region us-east-1 --name agent-agency-production

# Verify cluster access
kubectl cluster-info
kubectl get nodes

# Install required controllers
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.11.0/cert-manager.yaml
kubectl apply -f https://github.com/external-secrets/external-secrets/releases/download/v0.8.1/external-secrets.yaml
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.7.0/deploy/static/provider/aws/deploy.yaml
```

## Secrets Management

### External Secrets Operator Setup

```bash
# Create AWS Secrets Manager backend
kubectl apply -f deploy/kubernetes/base/monitoring/external-secrets.yml

# Verify ESO installation
kubectl get pods -n external-secrets-system

# Create secrets in AWS Secrets Manager
aws secretsmanager create-secret \
  --name prod/agent-agency/database \
  --secret-string '{"password":"your-db-password"}'

aws secretsmanager create-secret \
  --name prod/agent-agency/auth \
  --secret-string '{"jwt-secret":"your-jwt-secret","encryption-key":"your-encryption-key"}'

aws secretsmanager create-secret \
  --name prod/agent-agency/ai \
  --secret-string '{"openai-api-key":"your-openai-key","anthropic-api-key":"your-anthropic-key"}'
```

### Certificate Management

```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.11.0/cert-manager.yaml

# Create ClusterIssuer for Let's Encrypt
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@yourdomain.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

## Application Deployment

### Automated Deployment

```bash
# Run the production deployment script
./scripts/deploy-production.sh \
  --namespace=agent-agency-prod \
  --cluster=agent-agency-production \
  --region=us-east-1
```

### Manual Deployment Steps

```bash
# 1. Build and push images
docker build -f deploy/docker/Dockerfile.orchestrator -t your-registry/agent-agency-orchestrator:latest .
docker build -f deploy/docker/Dockerfile.council -t your-registry/agent-agency-council:latest .

docker push your-registry/agent-agency-orchestrator:latest
docker push your-registry/agent-agency-council:latest

# 2. Deploy to Kubernetes
cd deploy/kubernetes/aws
kustomize edit set image agent-agency/orchestrator=your-registry/agent-agency-orchestrator:latest
kustomize edit set image agent-agency/council=your-registry/agent-agency-council:latest

kubectl apply -k overlays/production/

# 3. Verify deployment
kubectl get pods -n agent-agency-prod
kubectl get ingress -n agent-agency-prod
```

## Monitoring Setup

### Prometheus & Grafana

```bash
# Deploy monitoring stack
kubectl apply -f deploy/kubernetes/base/monitoring/prometheus-deployment.yml
kubectl apply -f deploy/kubernetes/base/monitoring/grafana-deployment.yml

# Access Grafana
kubectl port-forward svc/grafana 3000:3000 -n agent-agency-prod

# Default credentials: admin/admin (CHANGE IMMEDIATELY)
```

### Key Metrics to Monitor

#### Application Metrics
- **Task Completion Rate**: `rate(agent_agency_tasks_completed_total[5m])`
- **Council Evaluation Latency**: `histogram_quantile(0.95, rate(agent_agency_council_evaluation_duration_bucket[5m]))`
- **Active Tasks**: `agent_agency_active_tasks`
- **Error Rate**: `rate(agent_agency_errors_total[5m])`

#### System Metrics
- **CPU Utilization**: `rate(container_cpu_usage_seconds_total[5m])`
- **Memory Usage**: `container_memory_usage_bytes`
- **Disk I/O**: `rate(container_fs_reads_bytes_total[5m])`
- **Network I/O**: `rate(container_network_receive_bytes_total[5m])`

#### SLO Tracking
- **Council Evaluation Success**: `agent_agency_council_evaluations_success_total / agent_agency_council_evaluations_total`
- **API Response Time**: `histogram_quantile(0.95, rate(agent_agency_http_request_duration_seconds_bucket[5m]))`

## Security Configuration

### Network Policies

```bash
# Apply network security policies
kubectl apply -f deploy/kubernetes/base/network-policies.yml

# Verify policies
kubectl get networkpolicies -n agent-agency-prod
```

### Security Headers & TLS

The ingress configuration includes:
- **SSL/TLS termination** with Let's Encrypt certificates
- **Security headers** (HSTS, CSP, X-Frame-Options)
- **Rate limiting** (100 requests/minute)
- **CORS configuration** for authorized domains

### RBAC Configuration

```bash
# Apply RBAC policies
kubectl apply -f deploy/kubernetes/base/serviceaccount.yml

# Verify service accounts
kubectl get serviceaccounts -n agent-agency-prod
kubectl get clusterrolebindings | grep agent-agency
```

## Scaling Configuration

### Horizontal Pod Autoscaling

```bash
# Apply HPA configuration
kubectl apply -f deploy/kubernetes/base/hpa.yml

# Monitor scaling events
kubectl get hpa -n agent-agency-prod -w
```

### Vertical Scaling

```bash
# Update resource requests/limits
kubectl patch deployment agent-agency-orchestrator -n agent-agency-prod --type='json' \
  -p='[{"op": "replace", "path": "/spec/template/spec/containers/0/resources/requests/cpu", "value": "2"}]'

# Scale node groups (via AWS)
aws eks update-nodegroup-config \
  --cluster-name agent-agency-production \
  --nodegroup-name agent-agency-nodes \
  --scaling-config minSize=3,maxSize=50,desiredSize=10
```

## Backup & Recovery

### Automated Backups

```bash
# Deploy backup jobs
kubectl apply -f deploy/kubernetes/base/backup-recovery.yml

# Monitor backup jobs
kubectl get cronjobs -n agent-agency-prod
kubectl get jobs -n agent-agency-prod
```

### Manual Backup

```bash
# Database backup
kubectl exec -it postgres-0 -n agent-agency-prod -- pg_dump -U agent_agency agent_agency > backup.sql

# Redis backup
kubectl exec -it redis-0 -n agent-agency-prod -- redis-cli SAVE

# Upload to S3
aws s3 cp backup.sql s3://agent-agency-backups/database/manual-$(date +%Y%m%d-%H%M%S).sql
```

### Disaster Recovery

```bash
# Trigger disaster recovery
kubectl create job disaster-recovery --from=cronjob/disaster-recovery -n agent-agency-prod

# Monitor recovery progress
kubectl logs -f job/disaster-recovery -n agent-agency-prod
```

## Operational Procedures

### Health Checks

```bash
# Application health
curl -f https://api.agent-agency.yourdomain.com/health

# Council health
curl -f https://council.agent-agency.yourdomain.com/health

# Database connectivity
kubectl exec deployment/agent-agency-orchestrator -n agent-agency-prod -- pg_isready -h postgres

# Redis connectivity
kubectl exec deployment/agent-agency-orchestrator -n agent-agency-prod -- redis-cli -h redis ping
```

### Log Management

```bash
# View application logs
kubectl logs -f deployment/agent-agency-orchestrator -n agent-agency-prod

# View council logs
kubectl logs -f deployment/agent-agency-council -n agent-agency-prod

# Search logs with Loki (if configured)
kubectl port-forward svc/loki 3100:3100 -n monitoring
# Visit: http://localhost:3100
```

### Troubleshooting Common Issues

#### Pods Not Starting
```bash
# Check pod status
kubectl get pods -n agent-agency-prod
kubectl describe pod <pod-name> -n agent-agency-prod

# Check resource constraints
kubectl get events -n agent-agency-prod --sort-by=.metadata.creationTimestamp

# Check logs
kubectl logs <pod-name> -n agent-agency-prod --previous
```

#### Database Connection Issues
```bash
# Test database connectivity
kubectl exec deployment/agent-agency-orchestrator -n agent-agency-prod -- \
  psql -h postgres -U agent_agency -d agent_agency -c "SELECT 1"

# Check database logs
kubectl logs -f statefulset/postgres -n agent-agency-prod
```

#### High Memory Usage
```bash
# Check memory usage
kubectl top pods -n agent-agency-prod

# Check multi-tenant memory dashboard
kubectl port-forward svc/grafana 3000:3000 -n agent-agency-prod
# Navigate to Agent Agency → Memory Dashboard
```

## Performance Optimization

### Database Optimization

```bash
# Enable connection pooling
kubectl apply -f deploy/kubernetes/base/pgbouncer.yml

# Configure PostgreSQL parameters
kubectl exec -it postgres-0 -n agent-agency-prod -- psql -U agent_agency -c "
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET work_mem = '4MB';
SELECT pg_reload_conf();
"
```

### Caching Strategy

```bash
# Configure Redis for different cache types
kubectl exec -it redis-0 -n agent-agency-prod -- redis-cli CONFIG SET maxmemory 512mb
kubectl exec -it redis-0 -n agent-agency-prod -- redis-cli CONFIG SET maxmemory-policy allkeys-lru
```

### Model Optimization

```bash
# Enable model quantization (if supported)
kubectl set env deployment/agent-agency-council \
  ENABLE_QUANTIZATION=true \
  QUANTIZATION_LEVEL=int8 -n agent-agency-prod

# Configure model caching
kubectl set env deployment/agent-agency-council \
  MODEL_CACHE_SIZE=2GB \
  MODEL_CACHE_TTL=3600 -n agent-agency-prod
```

## Cost Optimization

### Resource Rightsizing

```bash
# Analyze resource usage
kubectl top pods -n agent-agency-prod --containers

# Right-size containers
kubectl patch deployment agent-agency-orchestrator -n agent-agency-prod --type='json' \
  -p='[{"op": "replace", "path": "/spec/template/spec/containers/0/resources/requests", "value": {"cpu": "500m", "memory": "1Gi"}}]'
```

### Spot Instance Usage

```bash
# Configure spot instances for non-critical workloads
kubectl apply -f deploy/kubernetes/base/spot-instances.yml

# Monitor spot instance interruptions
kubectl get events -n agent-agency-prod | grep spot
```

### Storage Optimization

```bash
# Use appropriate storage classes
kubectl patch pvc prometheus-storage -n agent-agency-prod \
  -p '{"spec": {"storageClassName": "gp3"}}'

# Configure backup retention
kubectl patch cronjob database-backup -n agent-agency-prod \
  --type='json' -p='[{"op": "replace", "path": "/spec/jobTemplate/spec/template/spec/containers/0/env", "value": [{"name": "RETENTION_DAYS", "value": "30"}]}]'
```

## Compliance & Security

### Security Audits

```bash
# Run security scans
kubectl apply -f deploy/kubernetes/base/security-scan.yml

# View security reports
kubectl get jobs -n agent-agency-prod
kubectl logs job/security-scan -n agent-agency-prod
```

### Compliance Monitoring

```bash
# Enable audit logging
kubectl set env deployment/agent-agency-orchestrator \
  AUDIT_LOG_ENABLED=true \
  AUDIT_LOG_LEVEL=detailed -n agent-agency-prod

# Configure compliance monitoring
kubectl apply -f deploy/kubernetes/base/compliance-monitor.yml
```

## Maintenance Procedures

### Regular Updates

```bash
# Update dependencies
kubectl set image deployment/agent-agency-orchestrator \
  orchestrator=your-registry/agent-agency-orchestrator:v3.1.0 -n agent-agency-prod

# Rolling update
kubectl rollout status deployment/agent-agency-orchestrator -n agent-agency-prod
```

### Certificate Rotation

```bash
# Renew certificates manually
kubectl delete certificate agent-agency-tls -n agent-agency-prod
kubectl apply -f deploy/kubernetes/base/ingress.yml

# Monitor certificate status
kubectl get certificates -n agent-agency-prod
```

### Database Maintenance

```bash
# Run VACUUM ANALYZE
kubectl exec -it postgres-0 -n agent-agency-prod -- psql -U agent_agency -d agent_agency -c "VACUUM ANALYZE;"

# Reindex tables
kubectl exec -it postgres-0 -n agent-agency-prod -- psql -U agent_agency -d agent_agency -c "REINDEX DATABASE agent_agency;"
```

## Monitoring & Alerting

### Alert Configuration

```yaml
# PrometheusRule for critical alerts
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: agent-agency-alerts
  namespace: agent-agency-prod
spec:
  groups:
  - name: agent-agency
    rules:
    - alert: HighErrorRate
      expr: rate(agent_agency_errors_total[5m]) > 0.1
      for: 5m
      labels:
        severity: critical
      annotations:
        summary: "High error rate detected"
        description: "Error rate is {{ $value }} errors per second"
```

### Alert Channels

```bash
# Configure Slack notifications
kubectl apply -f deploy/kubernetes/base/alertmanager.yml

# Test alerts
kubectl port-forward svc/alertmanager 9093:9093 -n agent-agency-prod
curl -X POST http://localhost:9093/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '[{"labels": {"alertname": "TestAlert"}, "annotations": {"summary": "Test alert"}}]'
```

## Rollback Procedures

### Automated Rollback

```bash
# Trigger rollback via CI/CD
# The deployment script handles automatic rollback on failures

# Manual rollback to previous version
kubectl rollout undo deployment/agent-agency-orchestrator -n agent-agency-prod
kubectl rollout undo deployment/agent-agency-council -n agent-agency-prod
```

### Blue-Green Rollback

```bash
# Switch traffic back to previous environment
kubectl patch svc agent-agency-api -n agent-agency-prod \
  --type='json' -p='[{"op": "replace", "path": "/spec/selector/version", "value": "blue"}]'

# Scale down failed environment
kubectl scale deployment agent-agency-orchestrator-green --replicas=0 -n agent-agency-prod
```

## Support & Escalation

### Support Contacts

- **Platform Team**: platform@yourcompany.com
- **Security Team**: security@yourcompany.com
- **Database Team**: database@yourcompany.com
- **DevOps On-Call**: PagerDuty integration

### Escalation Matrix

| Severity | Response Time | Communication |
|----------|---------------|---------------|
| **Critical** | 15 minutes | Phone call + Slack |
| **High** | 1 hour | Slack + email |
| **Medium** | 4 hours | Slack |
| **Low** | 24 hours | GitHub issue |

### Emergency Contacts

- **Primary**: +1-555-0100 (24/7)
- **Secondary**: +1-555-0101 (Business hours)
- **Security Incident**: security-incident@yourcompany.com

## Appendix

### Environment Variables Reference

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Yes | - |
| `REDIS_URL` | Redis connection string | Yes | - |
| `JWT_SECRET` | JWT signing secret | Yes | - |
| `ENCRYPTION_KEY` | Data encryption key | Yes | - |
| `OPENAI_API_KEY` | OpenAI API key | No | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | No | - |

### Resource Limits

| Component | CPU Request | CPU Limit | Memory Request | Memory Limit |
|-----------|-------------|-----------|----------------|--------------|
| Orchestrator | 500m | 2 | 1Gi | 4Gi |
| Council | 1 | 4 | 2Gi | 8Gi |
| PostgreSQL | 500m | 2 | 1Gi | 4Gi |
| Redis | 200m | 1 | 256Mi | 1Gi |
| Prometheus | 500m | 2 | 1Gi | 4Gi |
| Grafana | 100m | 500m | 256Mi | 1Gi |

### Network Ports

| Service | Port | Protocol | Description |
|---------|------|----------|-------------|
| Orchestrator API | 8080 | HTTP | Main API endpoint |
| Council API | 8081 | HTTP | AI decision API |
| PostgreSQL | 5432 | TCP | Database connections |
| Redis | 6379 | TCP | Cache connections |
| Prometheus | 9090 | HTTP | Metrics endpoint |
| Grafana | 3000 | HTTP | Dashboard access |
| Jaeger | 16686 | HTTP | Tracing UI |

### Backup Schedule

| Component | Frequency | Retention | Location |
|-----------|-----------|-----------|----------|
| Database | Daily 2 AM | 30 days | S3 |
| Redis | Every 4 hours | 7 days | S3 |
| etcd | Every 6 hours | 14 days | S3 |
| Logs | Daily | 90 days | S3 |
| Metrics | Continuous | 1 year | S3 |

This deployment guide provides a comprehensive framework for operating Agent Agency V3 in production. Regular updates and improvements should be made based on operational experience and changing requirements.
