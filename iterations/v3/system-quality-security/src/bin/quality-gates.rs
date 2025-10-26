use agent_agency_quality_gates::{run_quality_gates_with_config, QualityGateConfig, config::Severity};
use clap::{Arg, Command};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("quality-gates")
        .version("0.1.0")
        .author("Agent Agency")
        .about("Automated quality gates for Agent Agency codebase")
        .arg(
            Arg::new("path")
                .help("Path to check (defaults to current directory)")
                .index(1)
                .default_value(".")
        )
        .arg(
            Arg::new("max-lines")
                .long("max-lines")
                .help("Maximum lines per file")
                .default_value("1000")
        )
        .arg(
            Arg::new("format")
                .long("format")
                .help("Output format")
                .value_parser(["text", "json"])
                .default_value("text")
        )
        .get_matches();

    let path = Path::new(matches.get_one::<String>("path").unwrap());
    let max_lines: usize = matches.get_one::<String>("max-lines").unwrap().parse()?;
    let format = matches.get_one::<String>("format").unwrap();

    // Create custom config
    let mut config = QualityGateConfig::default();
    config.max_lines_per_file = max_lines;

    println!("🔍 Running quality gates on: {}", path.display());
    println!("📏 Max lines per file: {}", max_lines);

    let results = run_quality_gates_with_config(path, config).await?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        "text" => {
            println!("\n📊 Quality Gate Results:");
            println!("📁 Files checked: {}", results.total_files_checked);
            println!("⏱️  Execution time: {}ms", results.execution_time_ms);
            println!("✅ Passed: {}", results.passed);
            println!("🚫 Errors: {}", results.error_count());
            println!("⚠️  Warnings: {}", results.warning_count());
            println!("ℹ️  Info: {}", results.info_count());

            if !results.violations.is_empty() {
                println!("\n🚨 Violations:");
                for violation in &results.violations {
                    let severity_icon = match violation.severity {
                        agent_agency_quality_gates::Severity::Error => "🚫",
                        agent_agency_quality_gates::Severity::Warning => "⚠️",
                        agent_agency_quality_gates::Severity::Info => "ℹ️",
                    };

                    println!("{} {} in {}: {}", severity_icon, violation.rule, violation.file, violation.message);

                    if let Some(suggestion) = &violation.suggestion {
                        println!("  💡 {}", suggestion);
                    }

                    if let Some(line) = violation.line {
                        println!("  📍 Line: {}", line);
                    }
                }
            }

            if results.passed {
                println!("\n🎉 All quality gates passed!");
            } else {
                println!("\n❌ Quality gates failed!");
                std::process::exit(1);
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
