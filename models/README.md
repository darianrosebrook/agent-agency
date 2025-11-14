# CoreML Models Directory

This directory contains CoreML models used by the Agent Agency v3 system. These are large binary files that are git-ignored to avoid bloating the repository.

## Directory Structure

```
models/
├── coreml/                    # CoreML models (.mlmodelc format)
│   ├── whisper/              # Whisper speech-to-text models
│   ├── mistral/              # Mistral LLM models
│   ├── yolov3/               # YOLO object detection models
│   └── coreml-anything/      # Text-to-image generation models
├── scripts/                   # Model management scripts
└── README.md                  # This file
```

## Model Acquisition

Models are downloaded and managed via scripts in the `scripts/` directory:

```bash
# Download all available models
./scripts/download-coreml-models.sh

# Download specific model (if available)
./scripts/download-coreml-models.sh whisper

# Convert PyTorch models to CoreML (for models requiring conversion)
python scripts/convert_whisper_to_coreml.py

# Verify model integrity and functionality
./scripts/verify-coreml-models.sh

# Test model performance
python scripts/test_whisper_coreml.py --benchmark
```

## Current Models

### Whisper (Speech-to-Text)
- **Location**: `models/coreml/whisper/`
- **Model**: Whisper Large v3 (converted to CoreML)
- **Size**: ~1.5GB (encoder) + ~4GB (decoder)
- **Use Case**: High-accuracy multilingual transcription
- **Status**: Requires conversion from PyTorch
- **Conversion**: Run `python models/scripts/convert_whisper_to_coreml.py`

### FastViT (Image Classification)
- **Location**: Root directory (legacy)
- **Model**: FastViT T8 F16
- **Size**: 7.5MB
- **Use Case**: Constitutional judge vision analysis
- **Status**: Currently tracked in git (temporary)

## Model Conversion Process

Some models require conversion from PyTorch/HuggingFace format to CoreML:

### Whisper Conversion
```bash
# Install conversion dependencies
pip install torch torchvision torchaudio openai-whisper coremltools

# Convert Whisper model
python scripts/convert_whisper_to_coreml.py --model-size large-v3

# Test the conversion
python scripts/test_whisper_coreml.py
```

### Conversion Requirements
- **Python 3.8+**
- **PyTorch** with MPS support (for Apple Silicon)
- **coremltools** for conversion
- **openai-whisper** for model loading
- **16GB+ RAM** for large model conversion
- **ANE-compatible CoreML tools** (macOS 12.0+)

### Future Models

#### Mistral (LLM)
- **Planned Location**: `models/coreml/mistral/`
- **Model**: Mistral-7B-Instruct-v0.3-CoreML
- **Size**: ~4.2GB
- **Use Case**: Constitutional judge deliberations

#### YOLOv3 (Object Detection)
- **Planned Location**: `models/coreml/yolov3/`
- **Model**: YOLOv3-CoreML
- **Size**: ~240MB
- **Use Case**: UI analysis and security monitoring

#### CoreML-Anything (Text-to-Image)
- **Planned Location**: `models/coreml/coreml-anything/`
- **Model**: CoreML-Anything-V3.1
- **Size**: ~2.1GB
- **Use Case**: Technical diagram generation

## Model Management

### available Practices

1. **Version Control**: Models are versioned by their source repository
2. **Integrity Checks**: SHA256 hashes are verified after download
3. **Space Management**: Models are compressed and only loaded when needed
4. **Backup**: Critical models are backed up separately from git

### Performance Considerations

- **Memory Usage**: Large models may require 4-8GB RAM during loading
- **Storage**: Allocate sufficient disk space (10GB+ recommended)
- **Loading Time**: First-time model loading may take 30-60 seconds
- **ANE Utilization**: Models are optimized for Apple Neural Engine acceleration

### Security

- Models are downloaded from trusted sources only
- SHA256 verification ensures integrity
- No executable code in model files (CoreML format is safe)
- Models are treated as immutable data

## Development Notes

- Models in this directory are git-ignored to prevent repository bloat
- Use `git status --ignored` to see ignored files
- Model files are tracked via SHA256 in documentation
- Updates to models require corresponding code changes

## Troubleshooting

### Model Loading Issues
```bash
# Check model file integrity
./scripts/verify-coreml-models.sh

# Re-download corrupted models
./scripts/download-coreml-models.sh --force

# Check ANE availability
./scripts/check-ane-availability.sh
```

### Performance Issues
```bash
# Monitor ANE utilization
./scripts/monitor-ane-performance.sh

# Profile model loading
./scripts/profile-model-loading.sh whisper-large-v3
```

---

*This directory is git-ignored. Model files are managed separately from source code.*
