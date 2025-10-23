//! Test API server configuration and authentication

use std::env;
use tokio;

async fn load_server_config(config_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Simple test of config loading logic
    let mut api_keys = None;

    // Override with environment variables if set
    if let Ok(env_keys) = env::var("AGENT_AGENCY_API_KEYS") {
        let keys: Vec<String> = env_keys.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !keys.is_empty() {
            api_keys = Some(keys);
        }
    }

    // If no config file and no environment variables, fail
    if api_keys.is_none() {
        return Err("No configuration found. Either provide a config file or set AGENT_AGENCY_API_KEYS environment variable.".into());
    }

    println!(" Configuration loaded successfully");
    println!("   API Keys: {}", api_keys.as_ref().unwrap().len());

    Ok(())
}

async fn test_auth_middleware(api_keys: &[String]) {
    // Simulate header extraction
    let test_headers = [
        ("authorization", "Bearer test-key-123"),
        ("x-api-key", "test-key-123"),
        ("authorization", "Bearer invalid-key"),
        ("", ""), // No auth
    ];

    for (header_name, header_value) in test_headers {
        let api_key = if header_name == "authorization" {
            header_value.strip_prefix("Bearer ").unwrap_or("")
        } else if header_name == "x-api-key" {
            header_value
        } else {
            ""
        };

        let is_valid = !api_key.is_empty() && api_keys.contains(&api_key.to_string());
        println!("Header '{}: {}' -> Valid: {}", header_name, header_value, is_valid);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Testing API Server Configuration");

    // Test 1: No config should fail
    println!("\n Test 1: No configuration (should fail)");
    env::remove_var("AGENT_AGENCY_API_KEYS");
    match load_server_config("nonexistent.toml").await {
        Ok(_) => println!(" Unexpected success"),
        Err(e) => println!(" Expected failure: {}", e),
    }

    // Test 2: Environment variable config should work
    println!("\n Test 2: Environment variable configuration");
    env::set_var("AGENT_AGENCY_API_KEYS", "test-key-123,dev-key-456");
    match load_server_config("nonexistent.toml").await {
        Ok(_) => println!(" Environment config loaded successfully"),
        Err(e) => println!(" Unexpected failure: {}", e),
    }

    // Test 3: Authentication middleware
    println!("\n Test 3: Authentication middleware");
    let api_keys = vec!["test-key-123".to_string(), "dev-key-456".to_string()];
    test_auth_middleware(&api_keys).await;

    println!("\n All tests completed!");
    Ok(())
}
