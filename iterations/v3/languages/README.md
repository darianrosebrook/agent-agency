# Language-Specific Components

This directory contains components implemented in languages other than Rust, organized by programming language.

## Organization

- **`swift/`** - Swift implementations for Apple platform integration
- **`python/`** - Python utilities and ML model management
- **`javascript/`** - JavaScript/TypeScript components (see `apps/` for full applications)

## Swift Components (`swift/`)

- **`asr-bridge/`** - Automatic Speech Recognition integration
- **`bridges/`** - General Apple platform bridges
- **`coreml-bridge/`** - Core ML model integration
- **`vision-bridge/`** - Computer vision components
- **`model_downloader/`** - Model download utilities

## Python Components (`python/`)

- **Workspace analysis scripts** - `check_workspace_status.py`, `check_all_crates.py`
- **Evidence processing** - `fix_evidence_relevance.py`, `fix_all_evidence_relevance.py`
- **`pytorch-wrapper/`** - PyTorch integration and utilities

## Purpose

These language-specific components serve specialized purposes:

- **Swift**: Native Apple platform integration (Core ML, Vision, Speech)
- **Python**: ML model management, data processing, analysis tools
- **JavaScript**: Web interfaces and tooling (full apps in `apps/`)

## Build Integration

These components are integrated into the build system through:

- Swift components compiled as frameworks/libraries
- Python scripts executed via build scripts
- JavaScript components bundled with web applications

## Development Notes

- Each language directory maintains its own tooling and dependencies
- Cross-language integration happens at the Rust boundary
- Language-specific tooling (Swift Package Manager, pip, npm) used appropriately
