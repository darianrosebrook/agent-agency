# Module Structure Templates

Standardized templates for consistent module structure across Agent Agency V3.

## Available Templates

### `lib.rs.template`

Standard library root file with proper documentation and exports.

**Variables:**
- `{{CRATE_NAME}}`: Name of the crate
- `{{CRATE_DESCRIPTION}}`: One-line description
- `{{FEATURES}}`: Key features
- `{{RESPONSIBILITIES}}`: What the crate is responsible for
- `{{modules}}`: Array of module objects with `name` field
- `{{has_public_exports}}`: Boolean for conditional exports
- `{{exports}}`: Array of export objects with `module` and `type` fields
- `{{has_macros}}`: Boolean for macro re-exports
- `{{crate_name}}`: Lowercase crate name for macro paths

### `mod.rs.template`

Standard module file for submodules.

**Variables:**
- `{{MODULE_NAME}}`: Name of the module
- `{{MODULE_DESCRIPTION}}`: One-line description
- `{{RESPONSIBILITIES}}`: What the module is responsible for
- `{{submodules}}`: Array of submodule objects with `name` field
- `{{has_public_exports}}`: Boolean for conditional exports
- `{{exports}}`: Array of export objects with `name` field

### `types.rs.template`

Standard types module with common derive attributes.

**Variables:**
- `{{MODULE_NAME}}`: Name of the module
- `{{types}}`: Array of type objects
- `{{enums}}`: Array of enum objects
- `{{type_aliases}}`: Array of type alias objects

## Usage

1. Copy the appropriate template to your module
2. Replace variables with actual values
3. Customize as needed while maintaining consistency

## Template Variables Format

Templates use Handlebars-style variables:
- `{{variable}}`: Simple replacement
- `{{#each array}}...{{/each}}`: Iteration over arrays
- `{{#if condition}}...{{/if}}`: Conditional blocks

## Examples

### Using lib.rs.template

```rust
//! agent-data-processing - Unified data processing pipeline
//!
//! Unified data processing pipeline consolidating ingestors/enrichers/indexers/knowledge/file-ops.
//!
//! This crate provides comprehensive data processing capabilities while maintaining domain separation.

pub mod ingestors;
pub mod enrichers;
pub mod indexers;
pub mod knowledge;
pub mod file_ops;

// Re-export main types and functions for easy access
pub use ingestors::DataIngestor;
pub use enrichers::DataEnricher;
pub use indexers::DataIndexer;
pub use knowledge::KnowledgeProcessor;
pub use file_ops::FileOperator;
```

### Using mod.rs.template

```rust
//! Ingestors module
//!
//! Data ingestion capabilities for various sources.
//!
//! This module provides data ingestion from files, APIs, and databases following SOLID principles.

pub mod file_ingestor;
pub mod api_ingestor;
pub mod database_ingestor;

// Re-export main types and functions for easy access
pub use file_ingestor::*;
pub use api_ingestor::*;
pub use database_ingestor::*;
```

## Standards

### Documentation

- Use `//!` for module-level documentation
- Include purpose, responsibilities, and SOLID principles compliance
- Keep descriptions concise but informative

### Exports

- Re-export main types for convenience
- Use wildcard exports (`pub use module::*`) for comprehensive APIs
- Use specific exports for focused APIs

### Module Organization

- Group related functionality together
- Keep modules focused on single responsibilities
- Use clear, descriptive names

## Migration

When migrating existing modules to use templates:

1. Copy template content
2. Update variable placeholders
3. Move existing documentation
4. Adjust exports as needed
5. Test compilation and functionality
