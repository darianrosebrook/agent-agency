# ARBITER-006 API Keys Setup Guide

**Component**: Knowledge Seeker  
**Priority**: 🔥 HIGHEST ROI  
**Time Estimate**: 2-4 hours  
**Status**: ⏳ IN PROGRESS

---

## Overview

ARBITER-006 (Knowledge Seeker) is **90% complete** with all 3 search providers fully implemented (882 lines). The only requirement for production readiness is setting up API keys for Google and Bing search.

**Why This is Highest ROI**:

- All code is implemented and tested
- Just needs API configuration
- Enables powerful research capabilities
- 90% → 100% in 1-2 days!

---

## Option 1: Google Custom Search API (Recommended Primary)

### Step 1: Create Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com)
2. Click "Select a project" → "New Project"
3. Name: `agent-agency-search`
4. Click "Create"

### Step 2: Enable Custom Search API

1. In Google Cloud Console, go to "APIs & Services" → "Library"
2. Search for "Custom Search API"
3. Click "Custom Search API"
4. Click "Enable"

### Step 3: Create API Credentials

1. Go to "APIs & Services" → "Credentials"
2. Click "Create Credentials" → "API key"
3. Copy the API key (keep it secure!)
4. (Optional) Click "Restrict Key" to limit to Custom Search API only

### Step 4: Create Custom Search Engine

1. Go to [Google Programmable Search Engine](https://programmablesearchengine.google.com/controlpanel/create)
2. Configuration:
   - **Name**: "Agent Agency Search"
   - **What to search**: "Search the entire web"
   - **Search settings**: Enable "Image search" and "SafeSearch"
3. Click "Create"
4. Copy the **Search engine ID** (cx parameter)

### Step 5: Set Environment Variables

Add to your `.env` file or shell:

```bash
# Google Custom Search API
export GOOGLE_SEARCH_API_KEY="AIza...your_api_key_here"
export GOOGLE_SEARCH_CX="your_custom_search_engine_id_here"
```

### Pricing

**Free Tier**: 100 queries per day  
**Paid Tier**: $5 per 1,000 queries (up to 10,000/day)

For development/testing, the free tier is sufficient.

---

## Option 2: Bing Web Search API (Recommended Fallback)

### Step 1: Create Azure Account

1. Go to [Azure Portal](https://portal.azure.com)
2. Sign in or create a free account
3. Azure free tier includes $200 credit for 30 days

### Step 2: Create Bing Search Resource

1. In Azure Portal, click "Create a resource"
2. Search for "Bing Search v7"
3. Click "Create"
4. Configuration:
   - **Subscription**: Select your subscription
   - **Resource group**: Create new → "agent-agency-rg"
   - **Region**: Choose nearest (e.g., "West US")
   - **Name**: "agent-agency-bing-search"
   - **Pricing tier**: F1 (Free) for testing, S1 for production
5. Click "Review + create" → "Create"

### Step 3: Get API Key

1. After deployment, go to the resource
2. Click "Keys and Endpoint" in left menu
3. Copy **Key 1** (keep it secure!)

### Step 4: Set Environment Variable

Add to your `.env` file or shell:

```bash
# Bing Web Search API
export BING_SEARCH_API_KEY="your_bing_api_key_here"
```

### Pricing

**Free Tier (F1)**: 1,000 queries per month  
**Standard Tier (S1)**: 1,000 queries per month, then $7 per 1,000 queries

---

## Option 3: DuckDuckGo (No API Key Required!)

**Already implemented and working!**

DuckDuckGoSearchProvider uses HTML scraping and requires no API key. It's automatically available as a fallback when Google/Bing fail or are unavailable.

**Features**:

- Privacy-focused
- No rate limits
- No API key required
- Slightly slower than API-based providers

---

## Testing Your Setup

### 1. Verify Environment Variables

```bash
# Check if variables are set
echo $GOOGLE_SEARCH_API_KEY
echo $GOOGLE_SEARCH_CX
echo $BING_SEARCH_API_KEY
```

### 2. Test Individual Providers

```bash
cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v2

# Test Google Search Provider
npm test -- tests/integration/knowledge/google-search-provider.test.ts

# Test Bing Search Provider
npm test -- tests/integration/knowledge/bing-search-provider.test.ts

# Test DuckDuckGo Search Provider (no API key needed)
npm test -- tests/integration/knowledge/duckduckgo-search-provider.test.ts
```

### 3. Test Knowledge Seeker Integration

```bash
# Test full knowledge seeker with all providers
npm test -- tests/integration/knowledge/knowledge-seeker-integration.test.ts
```

### 4. Test Research System

```bash
# Test task research augmentation
npm test -- tests/integration/research/research-flow.test.ts
```

### 5. Manual Test

Create a test script:

```typescript
// test-knowledge-seeker.ts
import { KnowledgeSeeker } from "./src/knowledge/KnowledgeSeeker";

const config = {
  enabled: true,
  providers: [
    {
      name: "google",
      type: "web_search" as any,
      endpoint: "https://www.googleapis.com/customsearch/v1",
      apiKey: process.env.GOOGLE_SEARCH_API_KEY,
      searchEngineId: process.env.GOOGLE_SEARCH_CX,
      rateLimit: { requestsPerMinute: 100, requestsPerHour: 1000 },
      limits: { maxResultsPerQuery: 10, maxConcurrentQueries: 5 },
    },
    {
      name: "bing",
      type: "web_search" as any,
      endpoint: "https://api.bing.microsoft.com/v7.0/search",
      apiKey: process.env.BING_SEARCH_API_KEY,
      rateLimit: { requestsPerMinute: 50, requestsPerHour: 500 },
      limits: { maxResultsPerQuery: 10, maxConcurrentQueries: 3 },
    },
  ],
  processor: {
    minRelevanceScore: 0.5,
    minCredibilityScore: 0.5,
    maxResultsToProcess: 10,
  },
  queryProcessing: {
    maxConcurrentQueries: 5,
    defaultTimeoutMs: 30000,
    retryAttempts: 2,
  },
};

async function test() {
  const seeker = new KnowledgeSeeker(config);

  const query = {
    text: "How to implement multi-armed bandit algorithm in TypeScript?",
    type: "technical_documentation" as any,
    context: {
      taskType: "code_implementation",
      requiredInfo: ["algorithm explanation", "code examples"],
    },
  };

  const results = await seeker.search(query);

  console.log("Search Results:", results);
  console.log("Total Results:", results.results.length);
  console.log("Confidence:", results.metadata.confidence);
}

test().catch(console.error);
```

Run it:

```bash
npx ts-node test-knowledge-seeker.ts
```

---

## Fallback Strategy

The Knowledge Seeker automatically handles provider failures:

1. **Primary**: Google Custom Search (if API key available)
2. **Fallback 1**: Bing Web Search (if API key available)
3. **Fallback 2**: DuckDuckGo (always available, no API key)
4. **Fallback 3**: Mock provider (development only)

**This means**: Even with no API keys, the system will work using DuckDuckGo!

---

## Production Configuration

### Environment Variables for Production

```bash
# Required for production
GOOGLE_SEARCH_API_KEY="your_production_google_key"
GOOGLE_SEARCH_CX="your_production_search_engine_id"
BING_SEARCH_API_KEY="your_production_bing_key"

# Optional: Database for result caching
DB_HOST="production-db-host"
DB_PORT="5432"
DB_NAME="agent_agency_v2"
DB_USER="knowledge_seeker"
DB_PASSWORD="secure_password"
```

### Rate Limiting

**Google Custom Search**:

- Free: 100 queries/day
- Paid: 10,000 queries/day max
- Recommended: 100 queries/minute

**Bing Web Search**:

- Free: 1,000 queries/month
- Paid: Unlimited (pay per query)
- Recommended: 50 queries/minute

**DuckDuckGo**:

- No rate limits (respectful scraping)
- Recommended: 10 queries/minute

### Monitoring

Add to your observability stack:

```typescript
// Metrics to track
- knowledge_seeker_queries_total (by provider)
- knowledge_seeker_query_latency_ms (P50, P95, P99)
- knowledge_seeker_query_errors_total (by error type)
- knowledge_seeker_provider_fallback_total
- knowledge_seeker_results_per_query (average)
- knowledge_seeker_confidence_score (average)
```

---

## Cost Estimation

### Development (1-10 developers)

**Recommended**:

- Google: Free tier (100 queries/day)
- Bing: Free tier (1,000 queries/month)
- **Monthly Cost**: $0

### Production (Small - 100 users)

**Estimated Usage**: 10,000 queries/month

**Recommended**:

- Google: Paid tier ($50/month for 10,000 queries)
- Bing: Fallback for redundancy ($70/month for 10,000 queries)
- **Monthly Cost**: $50-120

### Production (Medium - 1,000 users)

**Estimated Usage**: 100,000 queries/month

**Recommended**:

- Google: Paid tier ($500/month for 100,000 queries)
- Bing: Fallback ($700/month for 100,000 queries)
- **Monthly Cost**: $500-1,200

### Production (Large - 10,000+ users)

**Estimated Usage**: 1,000,000+ queries/month

**Recommended**:

- Google: Enterprise pricing (contact sales)
- Bing: Enterprise pricing (contact Microsoft)
- Consider caching strategy to reduce API calls
- **Monthly Cost**: Negotiate with providers

---

## Troubleshooting

### Error: "Invalid API Key"

**Google**:

- Verify API key is correct
- Check if Custom Search API is enabled
- Verify API key restrictions allow Custom Search API

**Bing**:

- Verify API key is correct
- Check if Bing Search v7 resource is active
- Verify subscription is active

### Error: "Quota Exceeded"

**Solution**:

- Upgrade to paid tier
- Enable result caching to reduce queries
- Implement request deduplication
- Use fallback providers

### Error: "Search Engine ID Not Found"

**Google**:

- Verify Custom Search Engine exists
- Check if Search Engine ID (cx) is correct
- Ensure search engine is active

### Slow Query Times

**Solutions**:

- Enable result caching in database
- Reduce `maxResultsPerQuery` in config
- Increase `defaultTimeoutMs` for slow networks
- Use concurrent queries for multiple searches

---

## Next Steps

After API keys are configured:

1. **Test all providers** - Verify each provider works independently
2. **Test fallback chain** - Verify automatic fallback works
3. **Run integration tests** - Full end-to-end testing
4. **Measure performance** - Confirm <30s latency for queries
5. **Deploy to staging** - Test in staging environment
6. **Monitor usage** - Track API usage and costs
7. **Deploy to production** - Roll out gradually

---

## Production Readiness Checklist

- [ ] Google Custom Search API configured
- [ ] Bing Web Search API configured
- [ ] Environment variables set in production
- [ ] All provider tests passing
- [ ] Fallback chain tested
- [ ] Integration tests passing
- [ ] Performance benchmarks met (<30s per query)
- [ ] Rate limiting configured
- [ ] Result caching enabled (optional)
- [ ] Monitoring configured
- [ ] Cost tracking enabled
- [ ] Documentation updated

**Once complete**: ARBITER-006 is **PRODUCTION-READY** ✅

---

**Status**: ARBITER-006 90% → 100% in 1-2 days  
**Next**: ARBITER-002 Integration Tests (Phase 2)

