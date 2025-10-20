# API Outage Incident Response Runbook

## Overview

This runbook provides procedures for diagnosing and resolving API service outages in Agent Agency V3 production environments. API outages can affect all client applications and require immediate response.

## Prerequisites

- **Access Level**: Platform Engineer or DevOps Engineer
- **Tools Required**:
  - kubectl configured for production cluster
  - AWS CLI with appropriate permissions
  - Database access (read-only for troubleshooting)
  - Monitoring dashboard access (Grafana)
  - Incident management tool access

## Detection

### Automated Alerts
- **API Health Check Failure**: `/health` endpoint returns 5xx status
- **Error Rate Spike**: API error rate > 5% for 5+ minutes
- **Response Time Degradation**: P95 response time > 2 seconds
- **Pod Crash Loop**: Orchestrator pods restarting repeatedly

### Manual Detection
- User reports of application unavailability
- Monitoring dashboard shows red status
- CI/CD pipeline failures
- Customer support tickets

### Severity Classification

| Severity | Impact | Response Time | Communication |
|----------|--------|---------------|---------------|
| **Critical** | Complete API outage | Immediate (<5 min) | Public status page, all stakeholders |
| **High** | Partial API degradation | <15 minutes | Engineering team, product managers |
| **Medium** | Intermittent failures | <30 minutes | Engineering team |
| **Low** | Minor performance issues | <1 hour | Engineering team |

## Investigation

### Step 1: Initial Assessment (0-2 minutes)

```bash
# Check overall cluster health
kubectl get nodes -o wide
kubectl get pods -n agent-agency --field-selector=status.phase!=Running

# Check API endpoint directly
curl -f https://api.agent-agency.com/health
curl -f https://api.agent-agency.com/v1/status

# Check load balancer status
aws elbv2 describe-target-health --target-group-arn $TARGET_GROUP_ARN
```

### Step 2: Application Layer Investigation (2-5 minutes)

```bash
# Check pod status and recent events
kubectl get pods -n agent-agency -o wide
kubectl describe pod/$(kubectl get pods -n agent-agency -l app=agent-agency-orchestrator -o jsonpath='{.items[0].metadata.name}') -n agent-agency

# Check recent deployments
kubectl rollout history deployment/agent-agency-orchestrator -n agent-agency

# Check application logs
kubectl logs -f deployment/agent-agency-orchestrator -n agent-agency --since=10m | head -100

# Check resource usage
kubectl top pods -n agent-agency
kubectl describe hpa agent-agency-orchestrator -n agent-agency
```

### Step 3: Infrastructure Layer Investigation (5-10 minutes)

```bash
# Check database connectivity
kubectl exec -it $(kubectl get pods -n agent-agency -l app=agent-agency-postgres -o jsonpath='{.items[0].metadata.name}') -n agent-agency -- psql -U agent_agency -d agent_agency -c "SELECT 1;"

# Check Redis connectivity
kubectl exec -it deployment/agent-agency-orchestrator -n agent-agency -- redis-cli -h agent-agency-redis ping

# Check network policies
kubectl get networkpolicies -n agent-agency

# Check service mesh (if applicable)
kubectl get virtualservices -n agent-agency
kubectl get destinationrules -n agent-agency
```

### Step 4: External Dependencies Investigation (10-15 minutes)

```bash
# Check external API status
curl -s https://api.openai.com/v1/models | head -10
curl -s https://api.anthropic.com/v1/messages | head -10

# Check CloudWatch metrics
aws cloudwatch get-metric-statistics \
  --namespace AWS/EC2 \
  --metric-name CPUUtilization \
  --start-time $(date -u -d '10 minutes ago' +%Y-%m-%dT%H:%M:%S) \
  --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
  --period 300 \
  --statistics Average

# Check ELB metrics
aws cloudwatch get-metric-statistics \
  --namespace AWS/ApplicationELB \
  --metric-name RequestCount \
  --start-time $(date -u -d '10 minutes ago' +%Y-%m-%dT%H:%M:%S) \
  --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
  --period 300 \
  --statistics Sum
```

## Resolution

### Scenario 1: Pod Crash Loop

**Symptoms**: Pods restarting continuously, CrashLoopBackOff status

**Resolution**:
```bash
# Check pod logs for crash reason
kubectl logs deployment/agent-agency-orchestrator -n agent-agency --previous

# Check resource limits
kubectl describe pod/$(kubectl get pods -n agent-agency -l app=agent-agency-orchestrator -o jsonpath='{.items[0].metadata.name}') -n agent-agency | grep -A 10 "Containers:"

# Scale down temporarily to investigate
kubectl scale deployment agent-agency-orchestrator --replicas=0 -n agent-agency

# Fix the issue (configuration, image, resources)

# Scale back up
kubectl scale deployment agent-agency-orchestrator --replicas=3 -n agent-agency
```

### Scenario 2: Database Connection Issues

**Symptoms**: Database connection timeouts, "connection refused" errors

**Resolution**:
```bash
# Check database pod status
kubectl get pods -n agent-agency -l app=agent-agency-postgres

# Check database logs
kubectl logs deployment/agent-agency-postgres -n agent-agency --tail=100

# Test database connectivity from application pod
kubectl exec -it deployment/agent-agency-orchestrator -n agent-agency -- nc -zv agent-agency-postgres 5432

# Check connection pool settings
kubectl exec -it deployment/agent-agency-orchestrator -n agent-agency -- env | grep DATABASE

# Restart database if needed
kubectl rollout restart deployment/agent-agency-postgres -n agent-agency
```

### Scenario 3: External API Rate Limiting

**Symptoms**: 429 errors from OpenAI/Anthropic APIs, degraded performance

**Resolution**:
```bash
# Check API key validity and limits
kubectl get secrets -n agent-agency

# Implement exponential backoff
kubectl set env deployment/agent-agency-council RETRY_ENABLED=true -n agent-agency

# Reduce concurrent requests
kubectl scale deployment/agent-agency-council --replicas=1 -n agent-agency

# Monitor API usage
aws cloudwatch get-metric-statistics \
  --namespace AgentAgency \
  --metric-name ExternalAPIRequests \
  --start-time $(date -u -d '1 hour ago' +%Y-%m-%dT%H:%M:%S) \
  --end-time $(date -u +%Y-%m-%dT%H:%M:%S) \
  --period 3600 \
  --statistics Sum
```

### Scenario 4: Resource Exhaustion

**Symptoms**: High CPU/memory usage, OOM kills, slow responses

**Resolution**:
```bash
# Check current resource usage
kubectl top pods -n agent-agency
kubectl top nodes

# Check HPA status
kubectl describe hpa agent-agency-orchestrator -n agent-agency

# Temporarily increase resource limits
kubectl set resources deployment/agent-agency-orchestrator \
  --limits=cpu=4,memory=8Gi \
  --requests=cpu=2,memory=4Gi -n agent-agency

# Scale horizontally
kubectl scale deployment/agent-agency-orchestrator --replicas=5 -n agent-agency

# Check for memory leaks in application
kubectl exec -it deployment/agent-agency-orchestrator -n agent-agency -- ps aux
```

### Scenario 5: Network Issues

**Symptoms**: Intermittent connectivity, timeouts, DNS resolution failures

**Resolution**:
```bash
# Check service discovery
kubectl get services -n agent-agency
kubectl get endpoints -n agent-agency

# Test DNS resolution
kubectl exec -it deployment/agent-agency-orchestrator -n agent-agency -- nslookup agent-agency-postgres

# Check network policies
kubectl get networkpolicies -n agent-agency

# Restart affected services
kubectl rollout restart deployment/agent-agency-orchestrator -n agent-agency
```

## Verification

### Health Checks

```bash
# Application health check
curl -f https://api.agent-agency.com/health

# Dependency checks
curl -f https://api.agent-agency.com/health/database
curl -f https://api.agent-agency.com/health/redis
curl -f https://api.agent-agency.com/health/external-apis

# Load test (if available)
kubectl apply -f deploy/kubernetes/base/load-test.yml
```

### Performance Validation

```bash
# Check response times
curl -w "@curl-format.txt" -o /dev/null -s https://api.agent-agency.com/v1/status

# Monitor for 5 minutes
kubectl logs -f deployment/agent-agency-orchestrator -n agent-agency --since=5m | grep -E "(ERROR|WARN)"

# Check error rates in monitoring
# Access Grafana: http://grafana.agent-agency.com/d/api-performance
```

### Functional Testing

```bash
# Test core API endpoints
curl -X POST https://api.agent-agency.com/v1/tasks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{"description": "Test task for verification"}'

# Verify audit trail is working
curl https://api.agent-agency.com/v1/audit/events?limit=5
```

## Prevention

### Immediate Actions
- **Implement Circuit Breakers**: Add circuit breakers for external API calls
- **Improve Monitoring**: Add more granular health checks and metrics
- **Resource Limits**: Set appropriate resource requests and limits
- **Connection Pooling**: Implement proper database connection pooling

### Long-term Improvements
- **Auto-scaling**: Implement more sophisticated HPA configurations
- **Chaos Engineering**: Regular chaos testing in staging
- **Performance Testing**: Automated performance regression testing
- **Gradual Rollouts**: Implement canary deployments for safer releases

## Escalation

### When to Escalate

**Escalate immediately if:**
- Outage duration > 30 minutes
- Customer impact is critical
- Data loss or corruption suspected
- Security breach indicators present

**Escalate to next level if:**
- Multiple services affected
- Root cause unclear after 15 minutes
- External dependency failures
- Production database issues

### Escalation Contacts

| Level | Contact | Response Time |
|-------|---------|---------------|
| **L1** | Platform Engineer On-Call | <5 minutes |
| **L2** | DevOps Lead | <15 minutes |
| **L3** | Engineering Director | <30 minutes |
| **L4** | CTO/VP Engineering | <60 minutes |

### Communication Plan

**Internal Communication:**
- Slack incident channel: `#incidents`
- Status updates every 10 minutes
- Post-mortem scheduled within 24 hours

**External Communication:**
- Status page: https://status.agent-agency.com
- Customer notifications for >15 minute outages
- Social media updates for >1 hour outages

## Post-Incident

### Immediate Actions
```bash
# Document the incident
kubectl create configmap incident-$(date +%Y%m%d-%H%M%S) \
  --from-literal=timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ) \
  --from-literal=severity=$SEVERITY \
  --from-literal=duration=$DURATION \
  --from-literal=impact=$IMPACT \
  -n agent-agency
```

### Follow-up Tasks
- **Root Cause Analysis**: Identify underlying cause within 24 hours
- **Fix Implementation**: Deploy permanent fix within 72 hours
- **Testing**: Verify fix doesn't break other functionality
- **Documentation**: Update runbooks with new procedures

### Retrospective
- **Timeline Reconstruction**: Document exact sequence of events
- **Impact Assessment**: Quantify customer and business impact
- **Action Items**: Assign ownership and timelines for improvements
- **Prevention Measures**: Implement safeguards to prevent recurrence

## Metrics and KPIs

### Incident Metrics
- **MTTR (Mean Time To Resolution)**: Target < 30 minutes
- **MTTD (Mean Time To Detection)**: Target < 5 minutes
- **False Positive Rate**: Target < 5%
- **Customer Impact Duration**: Track total minutes of downtime

### Process Metrics
- **Runbook Accuracy**: % of incidents resolved using existing runbooks
- **Escalation Rate**: % of incidents requiring escalation
- **Post-mortem Quality**: Score out of 5 for post-mortem comprehensiveness
- **Recurrence Rate**: % of incidents that recur within 30 days

## Related Runbooks

- [Rollback Procedures](rollback-procedures.md)
- [Database Connectivity Issues](incident-database.md)
- [High CPU Usage](incident-high-cpu.md)
- [Blue-Green Deployment](blue-green-deployment.md)
- [Backup and Recovery](backup-recovery.md)

---

**Last Updated**: December 2025
**Version**: 1.0
**Review Cycle**: Quarterly
