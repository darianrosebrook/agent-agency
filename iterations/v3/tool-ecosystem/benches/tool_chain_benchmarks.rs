//! Performance benchmarks for Tool Chain components
//!
//! Uses Criterion for accurate latency measurements and regression detection.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use petgraph::Graph;
use std::sync::Arc;
use tokio::runtime::Runtime;

use tool_chain_planner::{
    ToolChainPlanner, ToolChain, ToolNode, ToolEdge,
    PlanningContext, PlanningConstraints, ChainMetadata,
    ToolRegistry, SchemaRegistry,
};
use tool_bandits::LinUCBPolicy;
use executor::ChainExecutor;

/// Benchmark tool chain creation
fn bench_tool_chain_creation(c: &mut Criterion) {
    c.bench_function("tool_chain_creation", |b| {
        b.iter(|| {
            let mut chain = ToolChain {
                dag: Graph::new(),
                roots: Vec::new(),
                sinks: Vec::new(),
                estimated_cost: 0.0,
                estimated_time_ms: 0,
                confidence: 0.0,
                plan_hash: 0,
                metadata: ChainMetadata {
                    name: "bench_chain".to_string(),
                    description: "Benchmark chain".to_string(),
                    created_at: chrono::Utc::now(),
                    version: "1.0.0".to_string(),
                    author: "bench".to_string(),
                },
            };

            // Add benchmark nodes
            for i in 0..10 {
                let node = ToolNode {
                    tool_id: format!("tool_{}", i),
                    inputs: vec![],
                    outputs: vec![],
                    params: serde_json::Value::Null,
                    fallback: None,
                    sla_ms: 1000,
                    cost_hint: 0.01,
                    retry_policy: Default::default(),
                };
                chain.dag.add_node(node);
            }

            black_box(chain);
        });
    });
}

/// Benchmark DAG topological sorting
fn bench_dag_topological_sort(c: &mut Criterion) {
    c.bench_function("dag_topological_sort", |b| {
        b.iter(|| {
            let mut chain = ToolChain {
                dag: Graph::new(),
                roots: Vec::new(),
                sinks: Vec::new(),
                estimated_cost: 0.0,
                estimated_time_ms: 0,
                confidence: 0.0,
                plan_hash: 0,
                metadata: ChainMetadata {
                    name: "topo_chain".to_string(),
                    description: "Topological sort benchmark".to_string(),
                    created_at: chrono::Utc::now(),
                    version: "1.0.0".to_string(),
                    author: "bench".to_string(),
                },
            };

            // Create a diamond pattern DAG
            let node_a = chain.dag.add_node(ToolNode {
                tool_id: "input".to_string(),
                inputs: vec![],
                outputs: vec![],
                params: serde_json::Value::Null,
                fallback: None,
                sla_ms: 1000,
                cost_hint: 0.01,
                retry_policy: Default::default(),
            });

            let node_b = chain.dag.add_node(ToolNode {
                tool_id: "processor1".to_string(),
                inputs: vec![],
                outputs: vec![],
                params: serde_json::Value::Null,
                fallback: None,
                sla_ms: 1000,
                cost_hint: 0.01,
                retry_policy: Default::default(),
            });

            let node_c = chain.dag.add_node(ToolNode {
                tool_id: "processor2".to_string(),
                inputs: vec![],
                outputs: vec![],
                params: serde_json::Value::Null,
                fallback: None,
                sla_ms: 1000,
                cost_hint: 0.01,
                retry_policy: Default::default(),
            });

            let node_d = chain.dag.add_node(ToolNode {
                tool_id: "combiner".to_string(),
                inputs: vec![],
                outputs: vec![],
                params: serde_json::Value::Null,
                fallback: None,
                sla_ms: 1000,
                cost_hint: 0.01,
                retry_policy: Default::default(),
            });

            // Add edges
            chain.dag.add_edge(node_a, node_b, ToolEdge {
                from_port: "output".to_string(),
                to_port: "input".to_string(),
                codec: None,
            });
            chain.dag.add_edge(node_a, node_c, ToolEdge {
                from_port: "output".to_string(),
                to_port: "input".to_string(),
                codec: None,
            });
            chain.dag.add_edge(node_b, node_d, ToolEdge {
                from_port: "output".to_string(),
                to_port: "input1".to_string(),
                codec: None,
            });
            chain.dag.add_edge(node_c, node_d, ToolEdge {
                from_port: "output".to_string(),
                to_port: "input2".to_string(),
                codec: None,
            });

            // Benchmark topological iteration
            use petgraph::visit::Topo;
            let mut topo = Topo::new(&chain.dag);
            while let Some(_) = topo.next(&chain.dag) {
                black_box(());
            }
        });
    });
}

/// Benchmark bandit policy selection
fn bench_bandit_selection(c: &mut Criterion) {
    let mut policy = LinUCBPolicy::new(0.1, vec!["tool1".to_string(), "tool2".to_string(), "tool3".to_string()]);
    let context = ToolContextFeatures {
        task_type: "coding".to_string(),
        prompt_len: 1000,
        retrieval_k: 5,
        is_code_task: true,
        expected_latency_ms: 2000,
        cost_budget_cents: 100,
        risk_tier: 2,
    };
    let constraints = ToolConstraints {
        max_latency_ms: 3000,
        max_cost_cents: 150,
        require_caws: false,
    };
    let tools = vec!["tool1".to_string(), "tool2".to_string(), "tool3".to_string()];

    c.bench_function("bandit_tool_selection", |b| {
        b.iter(|| {
            let (selected_tool, propensity, confidence) = policy.select_tool(
                black_box(&context),
                black_box(&tools),
                black_box(&constraints)
            );
            black_box((selected_tool, propensity, confidence));
        });
    });
}

/// Benchmark bandit learning updates
fn bench_bandit_update(c: &mut Criterion) {
    let mut policy = LinUCBPolicy::new(0.1, vec!["tool1".to_string(), "tool2".to_string()]);
    let context = ToolContextFeatures {
        task_type: "analysis".to_string(),
        prompt_len: 500,
        retrieval_k: 3,
        is_code_task: false,
        expected_latency_ms: 1000,
        cost_budget_cents: 50,
        risk_tier: 1,
    };

    c.bench_function("bandit_learning_update", |b| {
        b.iter(|| {
            policy.update(
                black_box(&context),
                black_box(&"tool1".to_string()),
                black_box(0.8) // reward
            );
        });
    });
}

/// Benchmark schema validation
fn bench_schema_validation(c: &mut Criterion) {
    let registry = JsonSchemaRegistry::new();
    // Add test schemas
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "query": {"type": "string"},
            "limit": {"type": "integer", "minimum": 1, "maximum": 100}
        },
        "required": ["query"]
    });
    registry.register_schema("test.Query".to_string(), schema);

    let valid_data = serde_json::json!({"query": "test query", "limit": 10});
    let invalid_data = serde_json::json!({"limit": 10}); // missing required field

    c.bench_function("schema_validation_valid", |b| {
        b.iter(|| {
            let result = registry.validate("test.Query", black_box(&valid_data));
            black_box(result);
        });
    });

    c.bench_function("schema_validation_invalid", |b| {
        b.iter(|| {
            let result = registry.validate("test.Query", black_box(&invalid_data));
            black_box(result);
        });
    });
}

/// Benchmark schema conversion
fn bench_schema_conversion(c: &mut Criterion) {
    let registry = JsonSchemaRegistry::new();
    // Add converter for string to url
    registry.register_converter("string->url".to_string(), Box::new(|value| async {
        if let serde_json::Value::String(s) = value {
            Ok(serde_json::Value::String(format!("https://{}", s)))
        } else {
            Err(anyhow::anyhow!("Not a string"))
        }
    }));

    let input = serde_json::Value::String("example.com".to_string());

    c.bench_function("schema_conversion", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(registry.convert("string->url", black_box(input.clone())));
            black_box(result);
        });
    });
}

/// Benchmark plan hashing
fn bench_plan_hashing(c: &mut Criterion) {
    let mut chain = ToolChain {
        dag: Graph::new(),
        roots: Vec::new(),
        sinks: Vec::new(),
        estimated_cost: 1.5,
        estimated_time_ms: 2500,
        confidence: 0.85,
        plan_hash: 0,
        metadata: ChainMetadata {
            name: "hash_test_chain".to_string(),
            description: "Plan hashing benchmark".to_string(),
            created_at: chrono::Utc::now(),
            version: "1.0.0".to_string(),
            author: "bench".to_string(),
        },
    };

    // Add nodes for realistic hash
    for i in 0..5 {
        let node = ToolNode {
            tool_id: format!("hash_tool_{}", i),
            inputs: vec![],
            outputs: vec![],
            params: serde_json::Value::Null,
            fallback: None,
            sla_ms: 1000,
            cost_hint: 0.01,
            retry_policy: Default::default(),
        };
        chain.dag.add_node(node);
    }

    c.bench_function("plan_hash_computation", |b| {
        b.iter(|| {
            let hash = ToolChainPlanner::compute_plan_hash(black_box(chain.clone()));
            black_box(hash);
        });
    });
}

/// Benchmark concurrent chain execution
fn bench_concurrent_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_chain_execution", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Create executor with concurrency limit
                let executor = ChainExecutor::new(5); // 5 concurrent tools

                // Create mock chain and dependencies
                let chain = create_mock_chain();
                let registry = Arc::new(MockSchemaRegistry);
                let toolbox = Arc::new(MockToolbox);

                let cancel_token = tokio_util::sync::CancellationToken::new();

                let result = executor.execute(
                    black_box(&chain),
                    black_box(&*registry),
                    black_box(&*toolbox),
                    black_box(cancel_token),
                ).await;

                black_box(result);
            });
        });
    });
}

// Mock implementations for benchmarking
struct MockSchemaRegistry;
impl SchemaRegistry for MockSchemaRegistry {
    fn validate(&self, _key: &str, _value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    fn convert(&self, _from: &str, _to: &str, value: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(value)
    }
}

struct MockToolbox;
impl ToolRuntime for MockToolbox {
    async fn call(&self, _tool_id: &str, _params: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Simulate some processing time
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(serde_json::Value::String("mock result".to_string()))
    }
}

fn create_mock_chain() -> ToolChain {
    let mut chain = ToolChain {
        dag: Graph::new(),
        roots: Vec::new(),
        sinks: Vec::new(),
        estimated_cost: 0.1,
        estimated_time_ms: 100,
        confidence: 0.8,
        plan_hash: 12345,
        metadata: ChainMetadata {
            name: "mock_chain".to_string(),
            description: "Mock chain for benchmarking".to_string(),
            created_at: chrono::Utc::now(),
            version: "1.0.0".to_string(),
            author: "bench".to_string(),
        },
    };

    // Add a few mock nodes
    let node1 = chain.dag.add_node(ToolNode {
        tool_id: "mock_tool_1".to_string(),
        inputs: vec![],
        outputs: vec![],
        params: serde_json::Value::Null,
        fallback: None,
        sla_ms: 1000,
        cost_hint: 0.01,
        retry_policy: Default::default(),
    });

    let node2 = chain.dag.add_node(ToolNode {
        tool_id: "mock_tool_2".to_string(),
        inputs: vec![],
        outputs: vec![],
        params: serde_json::Value::Null,
        fallback: None,
        sla_ms: 1000,
        cost_hint: 0.01,
        retry_policy: Default::default(),
    });

    chain.dag.add_edge(node1, node2, ToolEdge {
        from_port: "output".to_string(),
        to_port: "input".to_string(),
        codec: None,
    });

    chain.roots.push(node1);
    chain.sinks.push(node2);

    chain
}

criterion_group!(
    benches,
    bench_tool_chain_creation,
    bench_dag_topological_sort,
    bench_bandit_selection,
    bench_bandit_update,
    bench_schema_validation,
    bench_schema_conversion,
    bench_plan_hash_computation,
    bench_concurrent_execution,
);
criterion_main!(benches);
