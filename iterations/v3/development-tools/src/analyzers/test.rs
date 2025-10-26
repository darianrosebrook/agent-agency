//! Tests for language analyzers

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzers::{
        RustAnalyzer, TypeScriptAnalyzer, JavaScriptAnalyzer,
        LanguageAnalyzerRegistry, ProgrammingLanguage, LanguageAnalyzer
    };

    #[test]
    fn test_rust_analyzer_basic() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        
        let result = analyzer.analyze(code, "test.rs");
        assert_eq!(result.language, ProgrammingLanguage::Rust);
        assert!(result.complexity_score > 0.0);
    }

    #[test]
    fn test_typescript_analyzer_basic() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}
"#;
        
        let result = analyzer.analyze(code, "test.ts");
        assert_eq!(result.language, ProgrammingLanguage::TypeScript);
        assert!(result.complexity_score > 0.0);
    }

    #[test]
    fn test_javascript_analyzer_basic() {
        let analyzer = JavaScriptAnalyzer::new();
        let code = r#"
function greet(name) {
    return `Hello, ${name}!`;
}
"#;
        
        let result = analyzer.analyze(code, "test.js");
        assert_eq!(result.language, ProgrammingLanguage::JavaScript);
        assert!(result.complexity_score > 0.0);
    }

    #[test]
    fn test_rust_analyzer_violations() {
        let analyzer = RustAnalyzer::new();
        let code = r#"
fn main() {
    unsafe {
        let x = 42;
    }
    panic!("This is a panic!");
}
"#;
        
        let result = analyzer.analyze(code, "test.rs");
        assert!(!result.violations.is_empty());
        
        // Should detect unsafe code and panic
        let violation_codes: Vec<&str> = result.violations.iter().map(|v| v.rule_id.as_str()).collect();
        assert!(violation_codes.contains(&"UNSAFE_CODE"));
        assert!(violation_codes.contains(&"PANIC_USAGE"));
    }

    #[test]
    fn test_typescript_analyzer_violations() {
        let analyzer = TypeScriptAnalyzer::new();
        let code = r#"
function test() {
    let x: any = "hello";
    console.log(x!);
}
"#;
        
        let result = analyzer.analyze(code, "test.ts");
        assert!(!result.warnings.is_empty());
        
        // Should detect any type usage and non-null assertion
        let warning_codes: Vec<&str> = result.warnings.iter().map(|w| w.rule_id.as_str()).collect();
        assert!(warning_codes.contains(&"ANY_TYPE"));
    }

    #[test]
    fn test_javascript_analyzer_violations() {
        let analyzer = JavaScriptAnalyzer::new();
        let code = r#"
function test() {
    var x = 42;
    console.log(x == "42");
}
"#;
        
        let result = analyzer.analyze(code, "test.js");
        assert!(!result.warnings.is_empty());
        
        // Should detect var usage and loose equality
        let warning_codes: Vec<&str> = result.warnings.iter().map(|w| w.rule_id.as_str()).collect();
        assert!(warning_codes.contains(&"VAR_USAGE"));
        assert!(warning_codes.contains(&"LOOSE_EQUALITY"));
    }

    #[test]
    fn test_language_analyzer_registry() {
        let registry = LanguageAnalyzerRegistry::new();
        
        // Test getting analyzers by language
        let rust_analyzer = registry.get_analyzer(&ProgrammingLanguage::Rust);
        assert!(rust_analyzer.is_some());
        
        let ts_analyzer = registry.get_analyzer(&ProgrammingLanguage::TypeScript);
        assert!(ts_analyzer.is_some());
        
        let js_analyzer = registry.get_analyzer(&ProgrammingLanguage::JavaScript);
        assert!(js_analyzer.is_some());
    }

    #[test]
    fn test_language_analyzer_registry_by_extension() {
        let registry = LanguageAnalyzerRegistry::new();
        
        // Test getting analyzers by file extension
        let rust_analyzer = registry.get_analyzer_for_extension("rs");
        assert!(rust_analyzer.is_some());
        assert_eq!(rust_analyzer.unwrap().language(), ProgrammingLanguage::Rust);
        
        let ts_analyzer = registry.get_analyzer_for_extension("ts");
        assert!(ts_analyzer.is_some());
        assert_eq!(ts_analyzer.unwrap().language(), ProgrammingLanguage::TypeScript);
        
        let js_analyzer = registry.get_analyzer_for_extension("js");
        assert!(js_analyzer.is_some());
        assert_eq!(js_analyzer.unwrap().language(), ProgrammingLanguage::JavaScript);
    }

    #[test]
    fn test_change_complexity_calculation() {
        let analyzer = RustAnalyzer::new();
        let diff = r#"
+fn new_function() {
+    println!("New function added");
+}
"#;
        
        let complexity = analyzer.calculate_change_complexity(diff, None);
        assert!(complexity.is_ok());
        assert!(complexity.unwrap() > 0.0);
    }

    #[test]
    fn test_programming_language_detection() {
        assert_eq!(ProgrammingLanguage::from_extension("rs"), ProgrammingLanguage::Rust);
        assert_eq!(ProgrammingLanguage::from_extension("ts"), ProgrammingLanguage::TypeScript);
        assert_eq!(ProgrammingLanguage::from_extension("tsx"), ProgrammingLanguage::TypeScript);
        assert_eq!(ProgrammingLanguage::from_extension("js"), ProgrammingLanguage::JavaScript);
        assert_eq!(ProgrammingLanguage::from_extension("jsx"), ProgrammingLanguage::JavaScript);
        assert_eq!(ProgrammingLanguage::from_extension("py"), ProgrammingLanguage::Python);
        assert_eq!(ProgrammingLanguage::from_extension("unknown"), ProgrammingLanguage::Unknown);
    }
}
