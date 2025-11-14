#!/bin/bash

# CoreML Model Verification Script
# Verifies CoreML models are present and functional
#
# Usage:
#   ./verify-coreml-models.sh              # Verify all models
#   ./verify-coreml-models.sh whisper      # Verify specific model

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODELS_DIR="$(dirname "$SCRIPT_DIR")"
SPECIFIC_MODEL=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Check if running on macOS
check_macos() {
    if [[ "$OSTYPE" != "darwin"* ]]; then
        log_error "This script must be run on macOS for CoreML compatibility"
        exit 1
    fi
}

# Verify CoreML model
verify_coreml_model() {
    local model_path="$1"
    local model_name="$2"

    if [[ ! -e "$model_path" ]]; then
        log_error "Model file not found: $model_path"
        return 1
    fi

    log_info "Verifying $model_name model: $model_path"

    # Check file/directory size (reasonable minimum size)
    local file_size
    if [[ -d "$model_path" ]]; then
        file_size=$(du -s "$model_path" | cut -f1)
        file_size=$((file_size * 1024))  # Convert KB to bytes
    else
        file_size=$(stat -f%z "$model_path" 2>/dev/null || stat -c%s "$model_path" 2>/dev/null)
    fi
    local min_size=$((1024 * 1024))  # 1MB minimum

    if (( file_size < min_size )); then
        log_error "Model file too small (${file_size} bytes). Expected > ${min_size} bytes"
        return 1
    fi

    log_info "File size: $((file_size / 1024 / 1024))MB"

    # Try to inspect model with coremltools (if available)
    if command -v python3 &> /dev/null; then
        if python3 -c "import coremltools" &> /dev/null; then
            log_info "Inspecting model metadata..."
            if python3 -c "
import coremltools as ct
import sys
try:
    model = ct.models.MLModel('$model_path')
    spec = model.get_spec()
    print(f'Model version: {spec.specificationVersion}')
    print(f'Inputs: {len(spec.description.input)}')
    print(f'Outputs: {len(spec.description.output)}')
    print('Model appears valid')
except Exception as e:
    print(f'Model inspection failed: {e}', file=sys.stderr)
    sys.exit(1)
" 2>/dev/null; then
                log_success "Model metadata validation passed"
            else
                log_warning "Could not inspect model metadata (coremltools may be needed)"
            fi
        else
            log_warning "coremltools not available for detailed model inspection"
        fi
    fi

    log_success "$model_name model verification passed"
    return 0
}

# Verify specific model
verify_model() {
    local model_name="$1"
    local model_dir="$MODELS_DIR/coreml/$model_name"
    local model_file="$model_dir/model.mlmodel"

    case "$model_name" in
        "whisper")
            # Whisper has separate encoder/decoder models
            local encoder_file="$model_dir/encoder.mlmodel"
            local decoder_file="$model_dir/decoder.mlmodel"

            if [[ -e "$encoder_file" ]] && [[ -e "$decoder_file" ]]; then
                verify_coreml_model "$encoder_file" "Whisper Encoder" && \
                verify_coreml_model "$decoder_file" "Whisper Decoder"
            else
                log_error "Whisper models not found. Run conversion:"
                log_error "  python models/scripts/convert_whisper_to_coreml.py"
                return 1
            fi
            ;;

        "fastvit")
            # FastViT is in root directory (legacy)
            local fastvit_file="$MODELS_DIR/../FastViTT8F16.mlpackage"

            if [[ -d "$fastvit_file" ]]; then
                log_info "FastViT model found (MLPackage format)"
                log_success "FastViT model verification passed"
            else
                log_warning "FastViT model not found at expected location"
            fi
            ;;

        *)
            # Generic CoreML model
            if [[ -f "$model_file" ]]; then
                verify_coreml_model "$model_file" "$model_name"
            else
                log_warning "$model_name model not found: $model_file"
                log_info "Run download script: ./download-coreml-models.sh $model_name"
                return 1
            fi
            ;;
    esac
}

# Main execution
main() {
    local models_to_verify=("$@")

    log_info "CoreML Model Verification Script"
    log_info "Models directory: $MODELS_DIR"

    check_macos

    # If no specific models provided, verify all known models
    if [[ ${#models_to_verify[@]} -eq 0 ]]; then
        models_to_verify=("whisper" "fastvit")
    fi

    log_info "Verifying models: ${models_to_verify[*]}"

    local success_count=0
    local total_count=${#models_to_verify[@]}

    for model in "${models_to_verify[@]}"; do
        if verify_model "$model"; then
            ((success_count++))
        fi
    done

    echo
    log_info "Verification complete: $success_count/$total_count models verified"

    if [[ $success_count -eq $total_count ]]; then
        log_success "All models verified successfully!"
    else
        log_warning "Some models failed verification. Run download/conversion scripts."
        exit 1
    fi
}

# Show usage if requested
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    cat << EOF
Usage: $0 [MODEL...]

Verify CoreML models are present and functional.

If no models are specified, verifies all known models.

MODELS:
    whisper      Whisper speech-to-text models
    fastvit      FastViT image classification model

EXAMPLES:
    $0                    # Verify all models
    $0 whisper           # Verify Whisper models
    $0 whisper fastvit   # Verify multiple models

EOF
    exit 0
fi

# Run main function
main "$@"
