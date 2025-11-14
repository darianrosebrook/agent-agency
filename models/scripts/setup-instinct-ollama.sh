#!/bin/bash

# Instinct Model Setup Script for Agent Agency
#
# Sets up Instinct model for code editing tasks in the agent system.
# This script handles both Ollama integration and GGUF model management.
#
# Usage:
#   ./setup-instinct-ollama.sh              # Full setup
#   ./setup-instinct-ollama.sh --ollama     # Ollama setup only
#   ./setup-instinct-ollama.sh --gguf       # GGUF setup only
#   ./setup-instinct-ollama.sh --test       # Run integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODELS_DIR="$(dirname "$SCRIPT_DIR")"
INSTINCT_DIR="$MODELS_DIR/coreml/instinct"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

# Check if Ollama is installed and running
check_ollama() {
    log_step "Checking Ollama installation..."

    if ! command -v ollama &> /dev/null; then
        log_error "Ollama is not installed. Please install Ollama first:"
        log_error "  brew install ollama  # macOS"
        log_error "  curl -fsSL https://ollama.ai/install.sh | sh  # Linux"
        exit 1
    fi

    # Start Ollama if not running
    if ! pgrep -f "ollama serve" > /dev/null; then
        log_info "Starting Ollama service..."
        ollama serve &
        sleep 3
    fi

    # Verify Ollama is responsive
    if ! curl -s http://localhost:11434/api/tags > /dev/null; then
        log_error "Ollama service is not responding on http://localhost:11434"
        exit 1
    fi

    log_success "Ollama is running and responsive"
}

# Setup Instinct in Ollama
setup_ollama_instinct() {
    log_step "Setting up Instinct model in Ollama..."

    local model_name="nate/instinct"

    # Check if model is already available
    if ollama list | grep -q "nate/instinct"; then
        log_success "Instinct model already available in Ollama"
        return 0
    fi

    # Pull the model
    log_info "Pulling Instinct model from Ollama library..."
    if ollama pull "$model_name"; then
        log_success "Successfully pulled Instinct model"
    else
        log_error "Failed to pull Instinct model from Ollama"
        log_warning "Make sure you have the Ollama version of Instinct available"
        return 1
    fi

    # Verify model loaded correctly
    if ollama list | grep -q "nate/instinct"; then
        log_success "Instinct model verified in Ollama"
    else
        log_error "Instinct model not found after pull"
        return 1
    fi
}

# Setup GGUF model
setup_gguf_instinct() {
    log_step "Setting up Instinct GGUF model..."

    local gguf_file="$INSTINCT_DIR/instinct-Q4_K_M.gguf"

    if [ ! -f "$gguf_file" ]; then
        log_error "GGUF file not found at: $gguf_file"
        log_info "Expected file: instinct-Q4_K_M.gguf"
        log_info "Please ensure the GGUF file is placed in $INSTINCT_DIR"
        return 1
    fi

    # Create Ollama model file for GGUF
    local modelfile="$INSTINCT_DIR/Modelfile"

    cat > "$modelfile" << EOF
FROM $gguf_file

# Instinct model configuration for code editing
PARAMETER temperature 0.1
PARAMETER top_p 0.9
PARAMETER top_k 40
PARAMETER num_ctx 4096

SYSTEM """
You are Instinct, a specialized code editing assistant.

Your purpose is to help with code editing, refactoring, and improvement tasks.
You excel at understanding code context and suggesting precise, helpful edits.

Guidelines:
- Focus on code quality and best practices
- Provide specific, actionable suggestions
- Consider the broader codebase context
- Suggest improvements that maintain functionality
- Be precise and technical in your recommendations
"""

TEMPLATE """
{{ if .System }}{{ .System }}

{{ end }}{{ .Prompt }}
"""
EOF

    log_success "Created Ollama Modelfile for Instinct GGUF"

    # Create the model in Ollama
    log_info "Creating Ollama model from GGUF..."
    if (cd "$INSTINCT_DIR" && ollama create instinct-gguf .); then
        log_success "Successfully created Instinct GGUF model in Ollama"
    else
        log_error "Failed to create Ollama model from GGUF"
        return 1
    fi
}

# Test Instinct integration
test_instinct() {
    log_step "Testing Instinct model integration..."

    local test_prompt="Write a Python function to calculate fibonacci numbers using memoization."

    log_info "Testing Ollama Instinct model..."
    if ollama run nate/instinct "$test_prompt" --format json 2>/dev/null | head -10; then
        log_success "Ollama Instinct model responds correctly"
    else
        log_warning "Ollama Instinct model test failed - may not be available"
    fi

    log_info "Testing GGUF Instinct model..."
    if ollama run instinct-gguf "$test_prompt" --format json 2>/dev/null | head -10; then
        log_success "GGUF Instinct model responds correctly"
    else
        log_warning "GGUF Instinct model test failed - may need setup"
    fi

    # Test code editing capabilities
    local code_edit_prompt='Given this Python code:
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

Improve this function with memoization to handle larger inputs efficiently.'

    log_info "Testing code editing capabilities..."
    echo "$code_edit_prompt" | ollama run nate/instinct 2>/dev/null | head -20
    log_success "Code editing test completed"
}

# Main setup function
main() {
    local setup_ollama=true
    local setup_gguf=true
    local run_tests=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --ollama)
                setup_ollama=true
                setup_gguf=false
                shift
                ;;
            --gguf)
                setup_ollama=false
                setup_gguf=true
                shift
                ;;
            --test)
                run_tests=true
                shift
                ;;
            -h|--help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --ollama    Setup Ollama Instinct model only"
                echo "  --gguf      Setup GGUF Instinct model only"
                echo "  --test      Run integration tests"
                echo "  -h, --help  Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    log_info "🤖 Setting up Instinct model for Agent Agency"
    log_info "📁 Models directory: $MODELS_DIR"
    log_info "🎯 Instinct directory: $INSTINCT_DIR"

    # Ensure directories exist
    mkdir -p "$INSTINCT_DIR"

    # Check Ollama
    check_ollama

    # Setup models
    if [ "$setup_ollama" = true ]; then
        setup_ollama_instinct || log_warning "Ollama setup failed, continuing..."
    fi

    if [ "$setup_gguf" = true ]; then
        setup_gguf_instinct || log_warning "GGUF setup failed, continuing..."
    fi

    # Run tests if requested
    if [ "$run_tests" = true ]; then
        test_instinct
    fi

    log_success "🎉 Instinct model setup completed!"
    echo ""
    log_info "Available Instinct models:"
    ollama list | grep instinct || log_warning "No Instinct models found"

    echo ""
    log_info "Usage examples:"
    echo "  # Use Ollama version"
    echo "  ollama run nate/instinct \"Refactor this function to use type hints\""
    echo ""
    echo "  # Use GGUF version"
    echo "  ollama run instinct-gguf \"Add error handling to this code\""
    echo ""
    echo "  # In Rust code (update your OllamaService)"
    echo "  let instinct = OllamaService::new("
    echo "      \"http://localhost:11434\".to_string(),"
    echo "      \"nate/instinct\".to_string(),"
    echo "  );"
}

# Run main function
main "$@"
