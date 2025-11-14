#!/usr/bin/env python3
"""
Convert OpenAI Whisper to CoreML format for Apple Silicon acceleration.

This script converts the PyTorch Whisper model to CoreML format, optimized for:
- ANE acceleration on Apple Silicon
- 16kHz audio input (standard Whisper)
- 30-second chunks (standard Whisper)
- FP16 precision for memory efficiency

Requirements:
    pip install torch torchvision torchaudio openai-whisper coremltools

Usage:
    python convert_whisper_to_coreml.py [--model-size large-v3] [--output-dir models/coreml/whisper]
"""

import argparse
import os
import sys
from pathlib import Path

try:
    import coremltools as ct
    from coremltools.models.neural_network import quantization_utils
    PYTORCH_AVAILABLE = False
    try:
        import torch
        import whisper
        PYTORCH_AVAILABLE = True
    except ImportError:
        print("PyTorch/Whisper not available - will create placeholder models")
        print("For full conversion, install: pip install torch openai-whisper")
except ImportError as e:
    print(f"Missing coremltools: {e}")
    print("Install with: pip install coremltools")
    sys.exit(1)


def load_whisper_model(model_size: str = "large-v3"):
    """Load Whisper model with specified size."""
    print(f"Loading Whisper {model_size} model...")
    model = whisper.load_model(model_size)
    model.eval()
    return model


def create_dummy_inputs():
    """Create dummy inputs for CoreML conversion."""
    # Whisper encoder expects input of shape [batch_size, n_mels, n_ctx]
    # Where n_mels=128 (after preprocessing) and n_ctx depends on audio length
    # For 30 seconds of audio at 16kHz, this becomes ~3000 time steps
    # But after Whisper's preprocessing, it becomes 128 mel bins
    mel_spectrogram = torch.randn(1, 128, 3000)  # Batch size 1, 128 mel bins, 3000 time steps

    return {"input": mel_spectrogram}


def convert_encoder_to_coreml(model, output_dir: Path):
    """Convert Whisper encoder to CoreML."""
    print("Converting Whisper encoder to CoreML...")

    # Create dummy input
    dummy_inputs = create_dummy_inputs()

    # Extract encoder from model
    encoder = model.encoder

    # Try tracing first (more lenient than scripting)
    print("Tracing encoder...")
    with torch.no_grad():
        encoder_traced = torch.jit.trace(encoder, dummy_inputs["input"])

    # Convert to CoreML with optimization
    model_coreml = ct.convert(
        encoder_traced,
        source="pytorch",
        inputs=[ct.TensorType(name="input", shape=dummy_inputs["input"].shape)],
        minimum_deployment_target=ct.target.macOS12,  # Monterey for ANE support
        compute_units=ct.ComputeUnit.ALL,  # Use CPU, GPU, and ANE
        compute_precision=ct.precision.FLOAT16,  # FP16 for memory efficiency
    )

    # Optimize for ANE
    print("Optimizing for ANE acceleration...")
    model_coreml = quantization_utils.quantize_weights(model_coreml, nbits=16)

    # Save encoder
    encoder_path = output_dir / "encoder.mlmodelc"
    model_coreml.save(str(encoder_path))
    print(f"Encoder saved to: {encoder_path}")

    return model_coreml


def convert_decoder_to_coreml(model, output_dir: Path):
    """Convert Whisper decoder to CoreML."""
    print("Converting Whisper decoder to CoreML...")

    # Extract decoder from model
    decoder = model.decoder

    # Create dummy inputs for decoder
    # Decoder expects: tokens, encoder outputs, and kv cache
    batch_size = 1
    n_ctx = 448  # Max tokens for Whisper
    n_state = model.dims.n_audio_state  # 1280 for large-v3
    n_head = model.dims.n_head  # 20 for large-v3

    # Token input (previous tokens)
    tokens = torch.randint(0, model.dims.n_vocab, (batch_size, n_ctx))

    # Encoder output (from encoder)
    encoder_output = torch.randn(batch_size, n_state, 1500)  # 1500 time steps

    # Try tracing first (more lenient than scripting)
    print("Tracing decoder...")
    with torch.no_grad():
        decoder_traced = torch.jit.trace(decoder, (tokens, encoder_output))

    # Convert decoder
    decoder_coreml = ct.convert(
        decoder_traced,
        source="pytorch",
        inputs=[
            ct.TensorType(name="tokens", shape=tokens.shape, dtype=ct.int32),
            ct.TensorType(name="audio_features", shape=encoder_output.shape),
        ],
        minimum_deployment_target=ct.target.macOS12,
        compute_units=ct.ComputeUnit.ALL,
        compute_precision=ct.precision.FLOAT16,
    )

    # Optimize for ANE
    decoder_coreml = quantization_utils.quantize_weights(decoder_coreml, nbits=16)

    # Save decoder
    decoder_path = output_dir / "decoder.mlmodelc"
    decoder_coreml.save(str(decoder_path))
    print(f"Decoder saved to: {decoder_path}")

    return decoder_coreml


def create_placeholder_coreml_model(name: str, input_shape: tuple, output_shape: tuple, output_dir: Path):
    """Create a placeholder CoreML model for testing."""
    print(f"Creating placeholder {name} model...")

    # Create a simple model spec manually
    from coremltools.proto import Model_pb2, FeatureTypes_pb2

    # Create model spec
    spec = Model_pb2.Model()
    spec.specificationVersion = 6  # CoreML 6

    # Set model description
    spec.description.input.add()
    spec.description.input[0].name = "input"
    spec.description.input[0].type.multiArrayType.shape.extend(input_shape)
    spec.description.input[0].type.multiArrayType.dataType = FeatureTypes_pb2.ArrayFeatureType.FLOAT32

    spec.description.output.add()
    spec.description.output[0].name = "output"
    spec.description.output[0].type.multiArrayType.shape.extend(output_shape)
    spec.description.output[0].type.multiArrayType.dataType = FeatureTypes_pb2.ArrayFeatureType.FLOAT32

    # Create a simple neural network
    nn = spec.neuralNetwork

    # Add input layer
    input_layer = nn.layers.add()
    input_layer.name = "input"
    input_layer.input.append("input")
    input_layer.output.append("input")

    # Add identity activation (pass-through)
    activation_layer = nn.layers.add()
    activation_layer.name = "identity"
    activation_layer.input.append("input")
    activation_layer.output.append("output")
    activation_layer.activation.linear.alpha = 1.0
    activation_layer.activation.linear.beta = 0.0

    # Create MLModel
    mlmodel = ct.models.MLModel(spec)

    # Save model
    model_path = output_dir / f"{name}.mlmodel"
    mlmodel.save(str(model_path))
    print(f"Placeholder {name} saved to: {model_path}")

    return mlmodel


def create_audio_preprocessor(output_dir: Path):
    """Create audio preprocessing model for CoreML."""
    print("Creating audio preprocessor...")

    # For now, create a placeholder - actual preprocessing happens in Swift
    print("Audio preprocessing handled in Swift WhisperAudioBridge")
    print("See: coreml-bridge/Sources/WhisperAudio/WhisperAudioBridge.swift")


def main():
    parser = argparse.ArgumentParser(description="Convert Whisper to CoreML")
    parser.add_argument(
        "--model-size",
        default="large-v3",
        choices=["tiny", "base", "small", "medium", "large-v2", "large-v3"],
        help="Whisper model size to convert"
    )
    parser.add_argument(
        "--output-dir",
        default="models/coreml/whisper",
        help="Output directory for CoreML models"
    )

    args = parser.parse_args()

    # Create output directory
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Converting Whisper {args.model_size} to CoreML")
    print(f"Output directory: {output_dir}")

    try:
        if PYTORCH_AVAILABLE:
            print("PyTorch available - performing full conversion...")

            # Load model
            model = load_whisper_model(args.model_size)

            # Convert components
            encoder_model = convert_encoder_to_coreml(model, output_dir)
            decoder_model = convert_decoder_to_coreml(model, output_dir)

            conversion_type = "full"
            source = "openai-whisper"
        else:
            print("PyTorch not available - creating placeholder models for testing...")

            # Create placeholder models with correct shapes
            encoder_model = create_placeholder_coreml_model(
                "encoder",
                input_shape=(1, 80, 3000),  # mel spectrogram
                output_shape=(1, 1280, 1500),  # encoder output
                output_dir=output_dir
            )

            decoder_model = create_placeholder_coreml_model(
                "decoder",
                input_shape=(1, 448),  # tokens + encoder output
                output_shape=(1, 51865),  # vocabulary size
                output_dir=output_dir
            )

            conversion_type = "placeholder"
            source = "generated"

        # Create audio preprocessor note
        create_audio_preprocessor(output_dir)

        # Create metadata file
        metadata = {
            "model_size": args.model_size,
            "conversion_type": conversion_type,
            "source": source,
            "converted_with": "coremltools",
            "precision": "fp16" if PYTORCH_AVAILABLE else "fp32",
            "compute_units": "ALL",
            "target_platform": "macOS12+",
            "components": ["encoder.mlmodelc", "decoder.mlmodelc"],
            "audio_preprocessing": "swift_bridge",
            "notes": [
                "Audio preprocessing handled in Swift for optimal performance",
                "See WhisperAudioBridge.swift for audio pipeline",
                "ANE acceleration enabled for Apple Silicon" if PYTORCH_AVAILABLE else "Placeholder models - replace with real conversion",
                "Supports 30-second audio chunks at 16kHz"
            ]
        }

        import json
        metadata_path = output_dir / "metadata.json"
        with open(metadata_path, 'w') as f:
            json.dump(metadata, f, indent=2)

        print(f"\nConversion complete!")
        print(f"Models saved to: {output_dir}")
        print(f"Metadata: {metadata_path}")
        print(f"Conversion type: {conversion_type}")

        if not PYTORCH_AVAILABLE:
            print("\n⚠️  PLACEHOLDER MODELS CREATED")
            print("To create real models, install PyTorch:")
            print("  pip install torch openai-whisper")
            print("Then re-run this script")

        print("\nNext steps:")
        print("1. Test models with: python models/scripts/test_whisper_coreml.py")
        print("2. Integrate with Rust code in apple-silicon/src/ane/models/whisper_model.rs")
        print("3. Build Swift bridge: cd coreml-bridge && swift build")
        print("4. Run ASR enricher tests")

    except Exception as e:
        print(f"Error during conversion: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
