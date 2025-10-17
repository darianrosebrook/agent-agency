use crate::types::*;
use anyhow::Result;
use std::path::Path;
use tracing::{debug, error, warn};

/// Language support for detecting and analyzing different programming languages
#[derive(Debug)]
pub struct LanguageSupport {
    /// Language support configuration
    config: DiffEvaluationConfig,
}

impl LanguageSupport {
    /// Create a new language support
    pub fn new(config: DiffEvaluationConfig) -> Result<Self> {
        debug!("Initializing language support");
        Ok(Self { config })
    }

    /// Detect programming language from file path and content
    pub async fn detect_language(
        &self,
        file_path: &str,
        content: &str,
    ) -> Result<ProgrammingLanguage> {
        debug!("Detecting language for file: {}", file_path);

        // Detect language based on file extension
        if let Some(extension) = Path::new(file_path).extension() {
            if let Some(extension_str) = extension.to_str() {
                match extension_str {
                    "rs" => return Ok(ProgrammingLanguage::Rust),
                    "ts" => return Ok(ProgrammingLanguage::TypeScript),
                    "tsx" => return Ok(ProgrammingLanguage::TypeScript),
                    "js" => return Ok(ProgrammingLanguage::JavaScript),
                    "jsx" => return Ok(ProgrammingLanguage::JavaScript),
                    "py" => return Ok(ProgrammingLanguage::Python),
                    "java" => return Ok(ProgrammingLanguage::Java),
                    "cpp" | "cc" | "cxx" => return Ok(ProgrammingLanguage::Cpp),
                    "c" => return Ok(ProgrammingLanguage::C),
                    "go" => return Ok(ProgrammingLanguage::Go),
                    "swift" => return Ok(ProgrammingLanguage::Swift),
                    "kt" => return Ok(ProgrammingLanguage::Kotlin),
                    "scala" => return Ok(ProgrammingLanguage::Scala),
                    "hs" => return Ok(ProgrammingLanguage::Haskell),
                    "ml" => return Ok(ProgrammingLanguage::OCaml),
                    "fs" => return Ok(ProgrammingLanguage::FSharp),
                    "clj" => return Ok(ProgrammingLanguage::Clojure),
                    "ex" => return Ok(ProgrammingLanguage::Elixir),
                    "erl" => return Ok(ProgrammingLanguage::Erlang),
                    "rb" => return Ok(ProgrammingLanguage::Ruby),
                    "php" => return Ok(ProgrammingLanguage::PHP),
                    "pl" => return Ok(ProgrammingLanguage::Perl),
                    "lua" => return Ok(ProgrammingLanguage::Lua),
                    "r" => return Ok(ProgrammingLanguage::R),
                    "jl" => return Ok(ProgrammingLanguage::Julia),
                    "zig" => return Ok(ProgrammingLanguage::Zig),
                    "nim" => return Ok(ProgrammingLanguage::Nim),
                    "dart" => return Ok(ProgrammingLanguage::Dart),
                    _ => {}
                }
            }
        }

        // Fallback to content-based detection
        if content.contains("fn ") && content.contains("->") {
            Ok(ProgrammingLanguage::Rust)
        } else if content.contains("function") && content.contains("=>") {
            Ok(ProgrammingLanguage::JavaScript)
        } else if content.contains("def ") && content.contains(":") {
            Ok(ProgrammingLanguage::Python)
        } else if content.contains("public class") && content.contains("{") {
            Ok(ProgrammingLanguage::Java)
        } else if content.contains("#include") && content.contains("int main") {
            Ok(ProgrammingLanguage::C)
        } else {
            Ok(ProgrammingLanguage::Unknown)
        }
    }
}
