# Production Environment Variables Template

Copy this template to create your `.env.production` file.

**Security**: Never commit `.env.production` to version control!

---

## How to Use

```bash
cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v2

# Create your production env file
touch .env.production

# Add to .gitignore (if not already)
echo ".env.production" >> .gitignore

# Copy the variables below into .env.production
# Replace placeholder values with your actual credentials
```

---

## Environment Variables

### ARBITER-006: Knowledge Seeker API Keys (PHASE 1 - START HERE!)

```bash
# Google Custom Search API (Primary search provider)
# Setup: https://console.cloud.google.com
# Free tier: 100 queries/day | Paid: $5 per 1,000 queries
GOOGLE_SEARCH_API_KEY="your_google_api_key_here"
GOOGLE_SEARCH_CX="your_custom_search_engine_id_here"

# Bing Web Search API (Fallback search provider)
# Setup: https://portal.azure.com
# Free tier: 1,000 queries/month | Paid: $7 per 1,000 queries
BING_SEARCH_API_KEY="your_bing_api_key_here"
```

### Database Configuration

```bash
# PostgreSQL Connection
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="agent_agency_v2"
DB_USER="postgres"
DB_PASSWORD="your_secure_password_here"

# Database Pool Configuration
DB_MAX_CONNECTIONS="10"
DB_CONNECTION_TIMEOUT_MS="10000"
DB_QUERY_TIMEOUT_MS="30000"
DB_RETRY_ATTEMPTS="3"
DB_RETRY_DELAY_MS="1000"
```

### Security Configuration (ARBITER-013)

```bash
# JWT Configuration
JWT_SECRET="your_256_bit_secret_key_here_minimum_32_characters"
JWT_ISSUER="agent-agency-v2"
JWT_AUDIENCE="api.agent-agency.com"
JWT_EXPIRATION="24h"

# RBAC Configuration
RBAC_ENABLED="true"
TENANT_ISOLATION_ENABLED="true"

# Rate Limiting
RATE_LIMIT_ENABLED="true"
RATE_LIMIT_WINDOW_MS="60000"
RATE_LIMIT_MAX_REQUESTS="100"
```

### Arbiter Orchestrator Configuration

```bash
# Orchestration Settings
MAX_CONCURRENT_TASKS="2000"
TASK_TIMEOUT_MS="300000"
ENABLE_METRICS="true"
ENABLE_TRACING="true"

# Research System Configuration
RESEARCH_ENABLED="true"
RESEARCH_MIN_CONFIDENCE="0.7"
RESEARCH_MAX_QUERIES="5"
RESEARCH_TIMEOUT_MS="30000"
```

### Resilience Configuration

```bash
# Circuit Breaker
CIRCUIT_BREAKER_ENABLED="true"
CIRCUIT_BREAKER_FAILURE_THRESHOLD="5"
CIRCUIT_BREAKER_TIMEOUT_MS="60000"
CIRCUIT_BREAKER_SUCCESS_THRESHOLD="2"

# Retry Policy
RETRY_MAX_ATTEMPTS="3"
RETRY_BASE_DELAY_MS="1000"
RETRY_MAX_DELAY_MS="10000"
```

### Monitoring & Observability

```bash
# Logging
LOG_LEVEL="info"
LOG_FORMAT="json"
LOG_FILE="logs/agent-agency.log"

# Metrics
METRICS_ENABLED="true"
METRICS_PORT="9090"

# Traces
TRACING_ENABLED="true"
TRACING_ENDPOINT="http://localhost:4318"
```

### Performance Tuning

```bash
# Agent Registry (ARBITER-001)
AGENT_REGISTRY_MAX_AGENTS="1000"
AGENT_REGISTRY_STALE_THRESHOLD_MS="86400000"
AGENT_REGISTRY_CLEANUP_INTERVAL_MS="3600000"

# Task Routing (ARBITER-002)
TASK_ROUTING_UCB_EXPLORATION="2.0"
TASK_ROUTING_EPSILON="0.1"
TASK_ROUTING_MIN_SAMPLES="10"

# Knowledge Seeker (ARBITER-006)
KNOWLEDGE_MAX_CONCURRENT_QUERIES="5"
KNOWLEDGE_DEFAULT_TIMEOUT_MS="30000"
KNOWLEDGE_RETRY_ATTEMPTS="2"
KNOWLEDGE_CACHE_ENABLED="true"
KNOWLEDGE_CACHE_TTL_MS="3600000"
```

### Development Settings

```bash
# Production Mode
NODE_ENV="production"
DEBUG="false"
MOCK_PROVIDERS="false"
```

### Backup & Recovery

```bash
# Backup Configuration
BACKUP_ENABLED="true"
BACKUP_INTERVAL_HOURS="24"
BACKUP_RETENTION_DAYS="30"
BACKUP_S3_BUCKET=""
BACKUP_S3_REGION=""
```

---

## Quick Setup Script

```bash
#!/bin/bash
# setup-production-env.sh

# Create .env.production
cat > .env.production << 'EOF'
# ARBITER-006 API Keys (Phase 1 - REQUIRED)
GOOGLE_SEARCH_API_KEY="REPLACE_ME"
GOOGLE_SEARCH_CX="REPLACE_ME"
BING_SEARCH_API_KEY="REPLACE_ME"

# Database
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="agent_agency_v2"
DB_USER="postgres"
DB_PASSWORD="REPLACE_ME"

# JWT Secret (generate with: openssl rand -base64 32)
JWT_SECRET="REPLACE_ME"
JWT_ISSUER="agent-agency-v2"
JWT_AUDIENCE="api.agent-agency.com"
JWT_EXPIRATION="24h"

# Production Settings
NODE_ENV="production"
LOG_LEVEL="info"
ENABLE_METRICS="true"
ENABLE_TRACING="true"
EOF

echo "Created .env.production"
echo "⚠️  Replace REPLACE_ME values with your actual credentials"
echo "See docs/deployment/ARBITER-006-API-SETUP-GUIDE.md for API key setup"
```

---

## Validation Script

```bash
#!/bin/bash
# validate-env.sh

# Check required variables
required_vars=(
  "GOOGLE_SEARCH_API_KEY"
  "GOOGLE_SEARCH_CX"
  "DB_HOST"
  "DB_PASSWORD"
  "JWT_SECRET"
)

missing=()
for var in "${required_vars[@]}"; do
  if [ -z "${!var}" ]; then
    missing+=("$var")
  fi
done

if [ ${#missing[@]} -eq 0 ]; then
  echo "All required environment variables are set"
else
  echo "Missing required variables:"
  printf '   - %s\n' "${missing[@]}"
  exit 1
fi
```

---

## Security Best Practices

### 1. Never Commit Secrets

```bash
# Add to .gitignore
echo ".env.production" >> .gitignore
echo ".env.*.local" >> .gitignore
```

### 2. Use Strong JWT Secrets

```bash
# Generate a strong JWT secret
openssl rand -base64 32

# Or use Node.js
node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
```

### 3. Rotate API Keys Regularly

- Google API keys: Every 90 days
- Bing API keys: Every 90 days
- JWT secrets: Every 180 days
- Database passwords: Every 90 days

### 4. Use Environment-Specific Keys

```bash
# Development
.env.development

# Staging
.env.staging

# Production
.env.production
```

---

## Next Steps

1. Copy this template to `.env.production`
2. Replace all `REPLACE_ME` and placeholder values
3. Follow `ARBITER-006-API-SETUP-GUIDE.md` for API keys
4. Run validation script: `bash validate-env.sh`
5. Test configuration: `npm run test:config`

---

**Status**: Template ready for production configuration  
**Next**: Set up API keys (see ARBITER-006-API-SETUP-GUIDE.md)  
**Priority**: HIGHEST ROI - Start here!

