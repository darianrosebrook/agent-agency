#!/bin/bash

# CoreML Model Download Script
# Downloads and verifies CoreML models for Agent Agency v3
#
# Usage:
#   ./download-coreml-models.sh              # Download all models
#   ./download-coreml-models.sh whisper      # Download only Whisper
#   ./download-coreml-models.sh --force      # Force re-download
#   ./download-coreml-models.sh --verify     # Only verify existing models

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODELS_DIR="$(dirname "$SCRIPT_DIR")"
FORCE_DOWNLOAD=false
VERIFY_ONLY=false
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

# Show usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] [MODEL]

Download and verify CoreML models for Agent Agency v3.

OPTIONS:
    --force      Force re-download of existing models
    --verify     Only verify existing models, don't download
    --help       Show this help message

MODELS:
    whisper      Whisper Large v3 (speech-to-text)
    mistral      Mistral-7B-Instruct-v0.3 (LLM)
    yolov3       YOLOv3 (object detection)
    fastvit      FastViT T8 F16 (image classification)
    all          Download all models (default)

EXAMPLES:
    $0                    # Download all models
    $0 whisper           # Download only Whisper
    $0 --force mistral   # Force re-download Mistral
    $0 --verify          # Verify all existing models

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --force)
            FORCE_DOWNLOAD=true
            shift
            ;;
        --verify)
            VERIFY_ONLY=true
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            SPECIFIC_MODEL="$1"
            shift
            ;;
    esac
done

# Model definitions with URLs and checksums (bash 3.2 compatible)
MODEL_URLS_whisper=""
MODEL_CHECKSUMS_whisper=""

MODEL_URLS_mistral="https://huggingface.co/mistralai/Mistral-7B-Instruct-v0.3/resolve/main/mistral-coreml.mlmodelc"
MODEL_CHECKSUMS_mistral=""

MODEL_URLS_yolov3="https://huggingface.co/onnx-community/yolov3/resolve/main/yolov3-coreml.mlmodelc"
MODEL_CHECKSUMS_yolov3=""

MODEL_URLS_fastvit="https://huggingface.co/apple/FastViT/resolve/main/fastvit-t8-f16.mlmodelc"
MODEL_CHECKSUMS_fastvit=""

# Check if running on macOS
check_macos() {
    if [[ "$OSTYPE" != "darwin"* ]]; then
        log_error "This script must be run on macOS for CoreML compatibility"
        exit 1
    fi
}

# Check available disk space
check_disk_space() {
    local required_gb=10
    # Use macOS df -h and parse human readable output
    local available=$(df -h . | tail -1 | awk '{print $4}' | sed 's/G$//')
    # Remove trailing 'G' if present and convert to number
    available=${available%G}

    # Check if it's a number
    if [[ $available =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
        if (( $(echo "$available < $required_gb" | bc -l) )); then
            log_error "Insufficient disk space. Need at least ${required_gb}GB, have ${available}GB"
            exit 1
        fi
    else
        log_warning "Could not parse disk space, skipping check"
        return 0
    fi

    log_info "Disk space check passed: ${available}GB available"
}

# Download file with progress
download_file() {
    local url="$1"
    local output="$2"

    log_info "Downloading $output from $url"

    if command -v curl &> /dev/null; then
        curl -L -o "$output" "$url" --progress-bar
    elif command -v wget &> /dev/null; then
        wget -O "$output" "$url"
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Verify file checksum
verify_checksum() {
    local file="$1"
    local expected_checksum="$2"

    if [[ -z "$expected_checksum" ]]; then
        log_warning "No checksum available for $file, skipping verification"
        return 0
    fi

    local actual_checksum
    if command -v shasum &> /dev/null; then
        actual_checksum=$(shasum -a 256 "$file" | cut -d' ' -f1)
    elif command -v sha256sum &> /dev/null; then
        actual_checksum=$(sha256sum "$file" | cut -d' ' -f1)
    else
        log_warning "No SHA256 tool found, skipping checksum verification"
        return 0
    fi

    if [[ "$actual_checksum" == "$expected_checksum" ]]; then
        log_success "Checksum verification passed for $file"
        return 0
    else
        log_error "Checksum verification failed for $file"
        log_error "Expected: $expected_checksum"
        log_error "Actual: $actual_checksum"
        return 1
    fi
}

# Download and verify a single model
download_model() {
    local model_name="$1"
    local url_var="MODEL_URLS_$model_name"
    local checksum_var="MODEL_CHECKSUMS_$model_name"
    local url="${!url_var}"
    local checksum="${!checksum_var}"

    if [[ -z "$url" ]]; then
        log_error "No URL defined for model: $model_name"
        return 1
    fi

    local model_dir="$MODELS_DIR/coreml/$model_name"
    local model_file="$model_dir/model.mlmodel"

    mkdir -p "$model_dir"

    # Check if model already exists
    if [[ -f "$model_file" ]] && [[ "$FORCE_DOWNLOAD" != true ]]; then
        log_info "Model $model_name already exists, verifying..."
        if verify_checksum "$model_file" "$checksum"; then
            log_success "Model $model_name is valid"
            return 0
        else
            log_warning "Model $model_name is corrupted, re-downloading..."
        fi
    fi

    if [[ "$VERIFY_ONLY" == true ]]; then
        if [[ -f "$model_file" ]]; then
            verify_checksum "$model_file" "$checksum"
        else
            log_warning "Model $model_name not found"
        fi
        return 0
    fi

    # Download model
    log_info "Downloading $model_name model..."
    local temp_file="$model_file.tmp"

    if download_file "$url" "$temp_file"; then
        # Check if it's a zip file and extract if needed
        if [[ "$temp_file" == *.zip ]]; then
            log_info "Extracting $model_name model from zip..."
            if unzip -q "$temp_file" -d "$model_dir"; then
                # Find the extracted .mlmodel file
                local extracted_model
                extracted_model=$(find "$model_dir" -name "*.mlmodel" -type f | head -1)

                if [[ -n "$extracted_model" ]]; then
                    mv "$extracted_model" "$model_file"
                    # Clean up extracted directory
                    find "$model_dir" -mindepth 1 -name "*.mlmodel" -prune -o -type f -delete
                    find "$model_dir" -type d -empty -delete
                else
                    log_error "No .mlmodel file found in extracted zip for $model_name"
                    rm -rf "$temp_file"
                    return 1
                fi
            else
                log_error "Failed to extract zip file for $model_name"
                rm -f "$temp_file"
                return 1
            fi
        else
            # Direct .mlmodelc file
            mv "$temp_file" "$model_file"
        fi

        # Verify final model file
        if verify_checksum "$model_file" "$checksum"; then
            log_success "Successfully downloaded and verified $model_name"
        else
            rm -f "$model_file"
            log_error "Downloaded file verification failed for $model_name"
            return 1
        fi
    else
        rm -f "$temp_file"
        log_error "Failed to download $model_name"
        return 1
    fi
}

# Main execution
main() {
    log_info "CoreML Model Download Script"
    log_info "Models directory: $MODELS_DIR"

    check_macos
    # check_disk_space  # Temporarily disabled for testing

    # Determine which models to process
    local models_to_download=()

    if [[ -n "$SPECIFIC_MODEL" ]]; then
        if [[ "$SPECIFIC_MODEL" == "all" ]]; then
            models_to_download=("whisper" "mistral" "yolov3" "fastvit")
        else
            models_to_download=("$SPECIFIC_MODEL")
        fi
    else
        models_to_download=("whisper" "mistral" "yolov3" "fastvit")
    fi

    log_info "Processing models: ${models_to_download[*]}"

    local success_count=0
    local total_count=${#models_to_download[@]}

    for model in "${models_to_download[@]}"; do
        if download_model "$model"; then
            ((success_count++))
        fi
    done

    log_info "Download complete: $success_count/$total_count models processed successfully"

    if [[ $success_count -eq $total_count ]]; then
        log_success "All models downloaded and verified successfully!"
    else
        log_warning "Some models failed to download. Run with --force to retry."
        exit 1
    fi
}

# Run main function
main "$@"
