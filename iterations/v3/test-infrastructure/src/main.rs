//! Test Infrastructure Verification
//!
//! This is a minimal test to verify that the test infrastructure is working properly.
//! It tests basic database connectivity and CRUD operations without any complex dependencies.

use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://test_user:test_password@localhost:5433/test_db".to_string());

    println!("🔍 Testing V3 test infrastructure...");
    println!("📍 Database URL: {}", database_url.replace("test_password", "****"));

    // Test 1: Database connection
    println!("Test 1: Database connection...");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    println!("✅ Database connection established");

    // Test 2: Basic query
    println!("Test 2: Basic query execution...");
    let result: (i32,) = sqlx::query_as("SELECT 42 as answer")
        .fetch_one(&pool)
        .await?;

    assert_eq!(result.0, 42);
    println!("✅ Basic query test passed: answer = {}", result.0);

    // Test 3: Test table verification
    println!("Test 3: Test table verification...");
    let tables_result = sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE 'test_%'")
        .fetch_all(&pool)
        .await?;

    println!("✅ Found {} test tables in database", tables_result.len());
    for row in &tables_result {
        let table_name: String = row.get("table_name");
        println!("  - {}", table_name);
    }

    // Test 4: CRUD operations on test table
    println!("Test 4: CRUD operations...");

    // Create test table
    sqlx::query("CREATE TABLE IF NOT EXISTS test_infrastructure (id SERIAL PRIMARY KEY, name TEXT, value INTEGER)")
        .execute(&pool)
        .await?;
    println!("  ✅ Table creation");

    // Insert
    let insert_result = sqlx::query("INSERT INTO test_infrastructure (name, value) VALUES ('infrastructure_test', 999)")
        .execute(&pool)
        .await?;
    assert_eq!(insert_result.rows_affected(), 1);
    println!("  ✅ Insert operation");

    // Select
    let select_result: (i32, String, i32) = sqlx::query_as("SELECT id, name, value FROM test_infrastructure WHERE name = 'infrastructure_test'")
        .fetch_one(&pool)
        .await?;
    assert_eq!(select_result.1, "infrastructure_test");
    assert_eq!(select_result.2, 999);
    println!("  ✅ Select operation: id={}, name={}, value={}", select_result.0, select_result.1, select_result.2);

    // Update
    let update_result = sqlx::query("UPDATE test_infrastructure SET value = 1000 WHERE name = 'infrastructure_test'")
        .execute(&pool)
        .await?;
    assert_eq!(update_result.rows_affected(), 1);
    println!("  ✅ Update operation");

    // Delete
    let delete_result = sqlx::query("DELETE FROM test_infrastructure WHERE name = 'infrastructure_test'")
        .execute(&pool)
        .await?;
    assert_eq!(delete_result.rows_affected(), 1);
    println!("  ✅ Delete operation");

    // Clean up
    sqlx::query("DROP TABLE test_infrastructure")
        .execute(&pool)
        .await?;
    println!("  ✅ Cleanup operation");

    // Test 5: Transaction test
    println!("Test 5: Transaction operations...");
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO test_agent_runs (agent_type, task_description, status) VALUES ('test_infrastructure', 'transaction_test', 'running')")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    println!("  ✅ Transaction committed");

    // Verify transaction worked
    let tx_verify: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM test_agent_runs WHERE agent_type = 'test_infrastructure'")
        .fetch_one(&pool)
        .await?;
    assert!(tx_verify.0 >= 1);
    println!("  ✅ Transaction verification: {} records found", tx_verify.0);

    pool.close().await;
    println!("\n🎉 All test infrastructure tests passed!");
    println!("✅ V3 test infrastructure is operational");
    println!("✅ Database connectivity verified");
    println!("✅ CRUD operations working");
    println!("✅ Transaction support confirmed");

    Ok(())
}






