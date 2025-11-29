# V3 API-Ready Test Scenarios

**Date**: 2025-11-29
**Purpose**: Practical test scenarios formatted for V3 API submission

## Quick Test Scenarios (Ready for API)

### Scenario 1: Code Quality Analysis
**API Format**:
```json
{
  "title": "Analyze code quality in data-infrastructure crate",
  "description": "Review the data-infrastructure crate for potential code quality issues, performance bottlenecks, and maintainability concerns. Provide specific recommendations with code examples.",
  "risk_tier": "2",
  "priority": "high",
  "execution_mode": "auto"
}
```

**Expected**: Detailed code analysis with actionable recommendations

### Scenario 2: Algorithm Optimization
**API Format**:
```json
{
  "title": "Optimize sorting algorithm performance",
  "description": "Analyze and optimize a sorting algorithm implementation. Compare time/space complexity, suggest improvements, and implement the most efficient version for large datasets.",
  "risk_tier": "2",
  "priority": "high",
  "execution_mode": "auto"
}
```

**Expected**: Algorithm analysis, optimization implementation, performance comparison

### Scenario 3: System Architecture Design
**API Format**:
```json
{
  "title": "Design scalable microservices architecture",
  "description": "Design a scalable microservices architecture for an e-commerce platform handling 100K+ concurrent users. Include service boundaries, communication patterns, data consistency strategies, and deployment considerations.",
  "risk_tier": "1",
  "priority": "critical",
  "execution_mode": "auto"
}
```

**Expected**: Complete architectural design with trade-off analysis

### Scenario 4: Security Vulnerability Assessment
**API Format**:
```json
{
  "title": "Conduct security assessment of authentication system",
  "description": "Perform a comprehensive security assessment of the current authentication system. Identify vulnerabilities, assess risk levels, and provide remediation recommendations with implementation details.",
  "risk_tier": "1",
  "priority": "critical",
  "execution_mode": "auto"
}
```

**Expected**: Security analysis report with prioritized remediation plan

### Scenario 5: Performance Optimization
**API Format**:
```json
{
  "title": "Optimize database query performance",
  "description": "Analyze slow database queries in the system, identify bottlenecks, and implement optimizations including indexing strategies, query restructuring, and caching mechanisms.",
  "risk_tier": "2",
  "priority": "high",
  "execution_mode": "auto"
}
```

**Expected**: Performance analysis with implemented optimizations

### Scenario 6: API Design Review
**API Format**:
```json
{
  "title": "Design REST API for task management system",
  "description": "Design a comprehensive REST API for a task management system. Include all CRUD operations, proper HTTP status codes, error handling, pagination, filtering, and OpenAPI documentation.",
  "risk_tier": "2",
  "priority": "normal",
  "execution_mode": "auto"
}
```

**Expected**: Complete API design with documentation

### Scenario 7: Error Handling Strategy
**API Format**:
```json
{
  "title": "Design comprehensive error handling strategy",
  "description": "Design a comprehensive error handling strategy for a distributed system. Include error classification, retry mechanisms, circuit breakers, logging standards, and user-facing error messages.",
  "risk_tier": "2",
  "priority": "high",
  "execution_mode": "auto"
}
```

**Expected**: Error handling framework design and implementation

### Scenario 8: Data Pipeline Design
**API Format**:
```json
{
  "title": "Design real-time data processing pipeline",
  "description": "Design a real-time data processing pipeline for IoT sensor data. Include data ingestion, validation, transformation, storage, and real-time analytics capabilities.",
  "risk_tier": "2",
  "priority": "high",
  "execution_mode": "auto"
}
```

**Expected**: Complete data pipeline architecture

### Scenario 9: Testing Strategy
**API Format**:
```json
{
  "title": "Develop comprehensive testing strategy",
  "description": "Develop a comprehensive testing strategy for a web application including unit tests, integration tests, end-to-end tests, performance tests, and CI/CD integration.",
  "risk_tier": "2",
  "priority": "normal",
  "execution_mode": "auto"
}
```

**Expected**: Testing framework design and implementation plan

### Scenario 10: Machine Learning Integration
**API Format**:
```json
{
  "title": "Integrate ML model for user behavior prediction",
  "description": "Integrate a machine learning model to predict user behavior patterns. Include data preparation, model selection, training pipeline, deployment strategy, and monitoring.",
  "risk_tier": "2",
  "priority": "high",
  "execution_mode": "auto"
}
```

**Expected**: ML integration with monitoring and maintenance plan

## Advanced Scenarios (Higher Complexity)

### Scenario 11: Crisis Management Plan
**API Format**:
```json
{
  "title": "Develop crisis management plan for system outage",
  "description": "Develop a comprehensive crisis management plan for handling major system outages. Include incident response procedures, communication protocols, recovery strategies, and post-incident analysis.",
  "risk_tier": "1",
  "priority": "critical",
  "execution_mode": "auto"
}
```

**Expected**: Crisis management framework with actionable procedures

### Scenario 12: Legacy System Migration
**API Format**:
```json
{
  "title": "Plan migration from legacy monolithic system",
  "description": "Plan the migration from a legacy monolithic system to microservices architecture. Include risk assessment, migration phases, rollback strategies, and stakeholder communication plan.",
  "risk_tier": "1",
  "priority": "critical",
  "execution_mode": "auto"
}
```

**Expected**: Migration plan with risk mitigation strategies

## Evaluation Framework

### Automated Metrics
- **Task Completion**: Successfully completes assigned task
- **Execution Time**: Completes within reasonable timeframes
- **Error Handling**: Gracefully handles errors and edge cases
- **Code Quality**: Produces well-structured, maintainable code

### Manual Evaluation
- **Reasoning Quality**: Logical analysis and decision-making
- **Creativity**: Novel approaches to problem-solving
- **Practicality**: Real-world applicability of solutions
- **Documentation**: Clear explanation and rationale

### Scoring Rubric
- **5 - Excellent**: Outstanding performance, creative solutions, comprehensive analysis
- **4 - Good**: Solid performance, good analysis, practical solutions
- **3 - Adequate**: Meets basic requirements, functional solutions
- **2 - Poor**: Incomplete or flawed solutions, poor analysis
- **1 - Unacceptable**: Fails to meet basic requirements

## Test Execution Script

```bash
#!/bin/bash
# V3 Test Scenario Runner

SCENARIOS_DIR="$(dirname "$0")"
API_URL="http://localhost:8080/api/v1/tasks"

echo "=== V3 Test Scenario Runner ==="
echo "API URL: $API_URL"
echo ""

# Function to submit task
submit_task() {
    local title="$1"
    local description="$2"
    local risk_tier="$3"
    local priority="$4"

    echo "Submitting: $title"

    response=$(curl -s -X POST "$API_URL" \
        -H "Content-Type: application/json" \
        -d "{
            \"title\": \"$title\",
            \"description\": \"$description\",
            \"risk_tier\": \"$risk_tier\",
            \"priority\": \"$priority\",
            \"execution_mode\": \"auto\"
        }")

    task_id=$(echo "$response" | jq -r '.task_id // empty')
    if [ -n "$task_id" ]; then
        echo "Task submitted: $task_id"
        echo "$task_id" >> submitted_tasks.txt
    else
        echo "Failed to submit task"
        echo "Response: $response"
    fi
    echo ""
}

# Submit test scenarios
submit_task "Code Quality Analysis" "Review data-infrastructure crate for code quality issues" "2" "high"
submit_task "Algorithm Optimization" "Optimize sorting algorithm for large datasets" "2" "high"
submit_task "Security Assessment" "Assess authentication system security" "1" "critical"

echo "Tasks submitted. Monitor with:"
echo "tail -f /tmp/api-server.log | grep -E '(Task.*completed|Task.*failed|Phase.*complete)'"
```
