# Common Patterns

Shared patterns, traits, and utilities for Agent Agency V3 to reduce duplication and promote consistency across the codebase.

## Features

- **Common Traits**: Health check, lifecycle, configuration, metrics, and validation traits
- **Shared Types**: Common data structures like `ValidationResult`, `OperationResult`, `HealthStatus`
- **Validation Utilities**: Email, UUID, URL validation and configuration validation
- **Macros**: Builder patterns, error types, component traits, and ID generation
- **Templates**: Standardized module structure templates

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
common-patterns.workspace = true
```

## Common Patterns

### Health Checkable Components

```rust
use common_patterns::traits::HealthCheckable;

#[async_trait::async_trait]
impl HealthCheckable for MyComponent {
    async fn health_check(&self) -> anyhow::Result<HealthStatus> {
        // Implement health check logic
        Ok(HealthStatus::Healthy)
    }

    fn component_name(&self) -> &str {
        "my_component"
    }
}
```

### Validation

```rust
use common_patterns::validation::Validators;

let result = Validators::validate_email("user@example.com");
if result.is_valid {
    println!("Valid email!");
}
```

### Macros

```rust
use common_patterns::define_id_type;

// Creates a newtyped ID with UUID backing
define_id_type!(UserId);

// Now you have UserId(uuid::Uuid) with convenience methods
let user_id = UserId::new();
```

## Module Structure

```
common-patterns/
├── src/
│   ├── lib.rs           # Main exports
│   ├── traits.rs        # Common trait definitions
│   ├── types.rs         # Shared type definitions
│   ├── validation.rs    # Validation utilities
│   └── macros.rs        # Procedural macros
└── README.md
```

## Templates

Standardized templates are available in `templates/` for consistent module structure:

- `lib.rs.template`: Standard library root
- `mod.rs.template`: Standard module file
- `types.rs.template`: Standard types module

## Contributing

When adding new common patterns:

1. Ensure they solve duplication across at least 2 modules
2. Add comprehensive documentation
3. Include usage examples
4. Update this README
