# Configuration Directory

This directory contains all configuration files for the Agent Agency V3 system, organized by purpose.

## Organization

- **`api/`** - API server configurations (TOML, YAML, etc.)
- **`rust/`** - Rust toolchain configurations (clippy, rustfmt, rust-analyzer)
- **`environment/`** - Environment variables and deployment configs

## Existing Config Structure

The `config/` crate contains shared configuration types and validation logic. This directory contains the actual configuration files used by the system.

## Usage

Configuration files in this directory are:
- Version controlled (except secrets)
- Environment-specific where needed
- Documented with comments
- Validated at startup

## Security Note

Never commit secrets or sensitive credentials to this directory. Use environment variables or secure secret management instead.
