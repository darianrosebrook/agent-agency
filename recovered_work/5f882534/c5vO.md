# Agent Agency V3 - Production Deployment Scenarios

## Overview

This directory contains comprehensive production deployment scenarios for Agent Agency V3, providing enterprise-grade deployment patterns across multiple cloud providers and infrastructure types.

## Deployment Scenarios

### 🚀 Quick Start (Development)
- **Docker Compose**: Single-node development environment
- **Local Kubernetes**: Minikube/MicroK8s setup for testing

### ☁️ Cloud Production Deployments
- **AWS EKS**: Production deployment on Amazon EKS
- **Google GKE**: Production deployment on Google GKE
- **Azure AKS**: Production deployment on Azure AKS
- **Multi-Cloud**: Hybrid deployment across cloud providers

### 🏗️ Infrastructure Patterns
- **High Availability**: Multi-zone, multi-region deployments
- **Auto-Scaling**: Horizontal Pod Autoscaling configurations
- **Disaster Recovery**: Backup and failover strategies
- **Security Hardening**: Production security configurations

## Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │────│   API Gateway    │────│  Orchestrator   │
│   (External)    │    │   (Kong/Traefik)│    │   (Rust Core)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         └────────────────────────┼────────────────────────┘
                                  │
                    ┌─────────────────┐    ┌─────────────────┐
                    │   Council       │────│   Judges        │
                    │   (AI Models)   │    │   (Specialized) │
                    └─────────────────┘    └─────────────────┘
                             │
                    ┌─────────────────┐    ┌─────────────────┐
                    │   Execution     │────│   Artifacts     │
                    │   (Workers)     │    │   (Storage)     │
                    └─────────────────┘    └─────────────────┘
```

## Components

### Core Services
- **Orchestrator**: Main coordination service (Rust)
- **Council**: AI decision-making service (Python/Node.js)
- **Execution Workers**: Task execution containers
- **Artifact Storage**: File and data storage

### Supporting Services
- **PostgreSQL**: Primary database
- **Redis**: Caching and session storage
- **Elasticsearch**: Search and analytics
- **Monitoring Stack**: Prometheus, Grafana, Jaeger

### Security Components
- **OAuth/OIDC**: Authentication service
- **Vault**: Secrets management
- **Cert-Manager**: TLS certificate management
- **Network Policies**: Service mesh security

## Quick Start

### Local Development

```bash
# Start development environment
docker-compose -f deploy/docker-compose/dev.yml up -d

# View logs
docker-compose -f deploy/docker-compose/dev.yml logs -f

# Stop environment
docker-compose -f deploy/docker-compose/dev.yml down
```

### Production Deployment

```bash
# Deploy to Kubernetes (AWS EKS example)
cd deploy/kubernetes/aws
terraform init
terraform plan
terraform apply

# Deploy application
kubectl apply -k overlays/production/
```

## Configuration

### Environment Variables

```bash
# Database
DATABASE_URL=postgresql://user:pass@host:5432/db
REDIS_URL=redis://host:6379

# Security
JWT_SECRET=your-secret-key
OAUTH_CLIENT_ID=your-client-id
ENCRYPTION_KEY=your-encryption-key

# AI Services
OPENAI_API_KEY=your-openai-key
ANTHROPIC_API_KEY=your-anthropic-key

# Monitoring
PROMETHEUS_ENDPOINT=http://prometheus:9090
JAEGER_ENDPOINT=http://jaeger:14268/api/traces
```

### Feature Flags

```yaml
# deploy/kubernetes/base/configmap.yml
apiVersion: v1
kind: ConfigMap
metadata:
  name: agent-agency-config
data:
  # Core features
  ENABLE_AUDIT_TRAIL: "true"
  ENABLE_CIRCUIT_BREAKERS: "true"
  ENABLE_GRACEFUL_DEGRADATION: "true"

  # AI capabilities
  ENABLE_CLAUDE_INTEGRATION: "true"
  ENABLE_GPT4_INTEGRATION: "true"
  ENABLE_OLLAMA_INTEGRATION: "true"

  # Security
  ENABLE_OAUTH: "true"
  ENABLE_MFA: "true"
  ENABLE_AUDIT_LOGGING: "true"

  # Performance
  MAX_CONCURRENT_TASKS: "50"
  REQUEST_TIMEOUT_SECONDS: "300"
  CIRCUIT_BREAKER_THRESHOLD: "10"
```

## Monitoring & Observability

### Metrics Dashboard

Access Grafana at: `http://localhost:3000` (admin/admin)

Key dashboards:
- **Agent Performance**: Task completion rates, response times
- **System Health**: CPU, memory, disk usage
- **Council Analytics**: Judge performance, decision quality
- **Error Tracking**: Error rates, recovery success

### Logging

```bash
# View application logs
kubectl logs -f deployment/agent-agency-orchestrator

# View audit trail
kubectl logs -f deployment/agent-agency-audit

# Search logs with Loki
kubectl port-forward svc/loki 3100:3100
# Visit: http://localhost:3100
```

### Tracing

```bash
# View distributed traces
kubectl port-forward svc/jaeger 16686:16686
# Visit: http://localhost:16686
```

## Scaling

### Horizontal Scaling

```yaml
# deploy/kubernetes/aws/hpa.yml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: agent-agency-orchestrator
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: agent-agency-orchestrator
  minReplicas: 3
  maxReplicas: 50
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

### Vertical Scaling

```yaml
# Update resource requests/limits
kubectl patch deployment agent-agency-orchestrator \
  --type='json' \
  -p='[{"op": "replace", "path": "/spec/template/spec/containers/0/resources/requests/cpu", "value": "2"}]'
```

## Backup & Recovery

### Database Backup

```bash
# Manual backup
kubectl exec -it postgres-0 -- pg_dump agent_agency > backup.sql

# Automated backup (via CronJob)
kubectl apply -f deploy/kubernetes/base/backup-job.yml
```

### Disaster Recovery

```bash
# Failover to backup region
kubectl apply -k overlays/disaster-recovery/

# Restore from backup
kubectl apply -f deploy/kubernetes/aws/restore-job.yml
```

## Security

### Network Security

```yaml
# Network policies
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: agent-agency-network-policy
spec:
  podSelector:
    matchLabels:
      app: agent-agency
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: api-gateway
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
```

### Secrets Management

```yaml
# External secrets operator
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: agent-agency-secrets
spec:
  secretStoreRef:
    kind: SecretStore
    name: aws-secretsmanager
  target:
    name: agent-agency-secret
    creationPolicy: Owner
  data:
  - secretKey: openai-api-key
    remoteRef:
      key: prod/agent-agency/openai-key
```

## Troubleshooting

### Common Issues

**Pods not starting:**
```bash
# Check pod status
kubectl get pods -n agent-agency

# View pod logs
kubectl logs -f pod/pod-name -n agent-agency

# Check resource constraints
kubectl describe pod pod-name -n agent-agency
```

**Database connection issues:**
```bash
# Test database connectivity
kubectl exec -it postgres-0 -- psql -U agent_agency -d agent_agency

# Check network policies
kubectl get networkpolicies -n agent-agency
```

**Performance issues:**
```bash
# Check resource usage
kubectl top pods -n agent-agency

# View performance metrics
kubectl port-forward svc/prometheus 9090:9090
# Visit: http://localhost:9090
```

## Cost Optimization

### Resource Optimization

```yaml
# Right-size containers
resources:
  requests:
    cpu: 500m
    memory: 1Gi
  limits:
    cpu: 2
    memory: 4Gi

# Use spot instances for non-critical workloads
nodeSelector:
  lifecycle: spot
```

### Storage Optimization

```yaml
# Use appropriate storage classes
persistentVolumeClaim:
  storageClassName: gp3  # AWS
  # storageClassName: standard-rwo  # GCP
  # storageClassName: managed-premium  # Azure
```

## Contributing

### Adding New Deployment Scenarios

1. Create new directory under `deploy/`
2. Add Terraform configurations for infrastructure
3. Add Kubernetes manifests
4. Update CI/CD pipelines
5. Add documentation and runbooks

### Testing Deployments

```bash
# Test locally
docker-compose -f deploy/docker-compose/test.yml up -d

# Test on Kubernetes
kubectl apply -k overlays/test/
```

## Support

- **Documentation**: See individual scenario READMEs
- **Issues**: GitHub Issues with `deployment` label
- **Discussions**: GitHub Discussions for deployment questions

## License

This deployment configuration is part of Agent Agency V3.
