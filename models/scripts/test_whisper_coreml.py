#!/usr/bin/env python3
"""
Test CoreML Whisper models for functionality and performance.

This script validates:
1. Model loading and initialization
2. Inference with dummy data
3. Performance benchmarks
4. ANE utilization (on Apple Silicon)

Usage:
    python test_whisper_coreml.py [--model-dir models/coreml/whisper]
"""

import argparse
import time
import numpy as np
from pathlib import Path

try:
    import coremltools as ct
except ImportError:
    print("coremltools not installed. Install with: pip install coremltools")
    exit(1)


def load_coreml_model(model_path: Path):
    """Load CoreML model and return model object."""
    print(f"Loading model: {model_path}")
    model = ct.models.MLModel(str(model_path))
    return model


def create_test_input(shape, dtype=np.float32):
    """Create test input tensor."""
    if dtype == np.int32:
        return np.random.randint(0, 1000, shape, dtype=dtype)
    else:
        return np.random.randn(*shape).astype(dtype)


def test_encoder_inference(encoder_path: Path):
    """Test encoder model inference."""
    print("\n=== Testing Encoder Model ===")

    try:
        encoder = load_coreml_model(encoder_path)

        # Create test mel spectrogram (1, 80, 3000)
        mel_input = create_test_input((1, 80, 3000))

        # Test inference
        start_time = time.time()
        predictions = encoder.predict({"input": mel_input})
        inference_time = time.time() - start_time

        print(f"Inference time: {inference_time:.3f}s")
        print(f"Output shapes: {[v.shape for v in predictions.values()]}")

        return True, inference_time

    except Exception as e:
        print(f"Encoder test failed: {e}")
        return False, 0.0


def test_decoder_inference(decoder_path: Path):
    """Test decoder model inference."""
    print("\n=== Testing Decoder Model ===")

    try:
        decoder = load_coreml_model(decoder_path)

        # Create test inputs - adjust for placeholder models
        if "placeholder" in str(decoder_path):
            tokens = create_test_input((1, 448), dtype=np.int32)  # Match placeholder input shape
            audio_features = None  # Placeholder decoder only takes tokens
        else:
            tokens = create_test_input((1, 448), dtype=np.int32)  # Batch size 1, 448 tokens
            audio_features = create_test_input((1, 1280, 1500))  # Encoder output

        # Test inference
        start_time = time.time()
        if audio_features is not None:
            predictions = decoder.predict({
                "tokens": tokens,
                "audio_features": audio_features
            })
        else:
            predictions = decoder.predict({"tokens": tokens})
        inference_time = time.time() - start_time

        print(f"Inference time: {inference_time:.3f}s")
        print(f"Output shapes: {[v.shape for v in predictions.values()]}")

        return True, inference_time

    except Exception as e:
        print(f"Decoder test failed: {e}")
        return False, 0.0


def benchmark_model(model_path: Path, model_name: str, num_runs: int = 5):
    """Benchmark model performance."""
    print(f"\n=== Benchmarking {model_name} ===")

    try:
        model = load_coreml_model(model_path)

        # Create appropriate test input based on model type
        if "encoder" in str(model_path):
            test_input = {"input": create_test_input((1, 80, 3000))}
        elif "decoder" in str(model_path):
            if "placeholder" in str(model_path):
                test_input = {"tokens": create_test_input((1, 448), dtype=np.int32)}
            else:
                test_input = {
                    "tokens": create_test_input((1, 448), dtype=np.int32),
                    "audio_features": create_test_input((1, 1280, 1500))
                }
        else:
            print(f"Unknown model type for {model_path}")
            return

        # Warm up
        _ = model.predict(test_input)

        # Benchmark
        times = []
        for i in range(num_runs):
            start_time = time.time()
            _ = model.predict(test_input)
            times.append(time.time() - start_time)

        avg_time = np.mean(times)
        std_time = np.std(times)
        min_time = np.min(times)
        max_time = np.max(times)

        print("Benchmark results:")
        print(f"Average: {avg_time:.3f}s")
        print(f"Std Dev: {std_time:.3f}s")
        print(f"Min: {min_time:.3f}s")
        print(f"Max: {max_time:.3f}s")
    except Exception as e:
        print(f"Benchmark failed: {e}")


def check_ane_utilization():
    """Check ANE utilization (macOS only)."""
    print("\n=== Checking ANE Utilization ===")

    try:
        import subprocess
        result = subprocess.run(
            ["powermetrics", "--samplers", "ane", "-n", "1", "-i", "1000"],
            capture_output=True,
            text=True,
            timeout=5
        )

        if "ANE" in result.stdout:
            print("ANE utilization detected")
            # Parse ANE usage from output
            lines = result.stdout.split('\n')
            for line in lines:
                if "ANE" in line and "%" in line:
                    print(f"ANE usage: {line.strip()}")
        else:
            print("ANE utilization not detected (may not be active)")

    except Exception as e:
        print(f"Could not check ANE utilization: {e}")
        print("Note: ANE monitoring requires admin privileges")


def main():
    parser = argparse.ArgumentParser(description="Test CoreML Whisper models")
    parser.add_argument(
        "--model-dir",
        default="models/coreml/whisper",
        help="Directory containing CoreML models"
    )
    parser.add_argument(
        "--benchmark",
        action="store_true",
        help="Run performance benchmarks"
    )
    parser.add_argument(
        "--check-ane",
        action="store_true",
        help="Check ANE utilization"
    )

    args = parser.parse_args()

    model_dir = Path(args.model_dir)
    if not model_dir.exists():
        print(f"Model directory not found: {model_dir}")
        print("Run conversion first: python convert_whisper_to_coreml.py")
        exit(1)

    print(f"Testing CoreML Whisper models in: {model_dir}")

    # Load metadata if available
    metadata_path = model_dir / "metadata.json"
    if metadata_path.exists():
        import json
        with open(metadata_path) as f:
            metadata = json.load(f)
        print(f"Model: Whisper {metadata.get('model_size', 'unknown')}")
        print(f"Converted with: {metadata.get('converted_with', 'unknown')}")
        print(f"Precision: {metadata.get('precision', 'unknown')}")

    # Test encoder
    encoder_path = model_dir / "encoder.mlmodel"
    encoder_success = False
    if encoder_path.exists():
        encoder_success, encoder_time = test_encoder_inference(encoder_path)
    else:
        print("Encoder model not found")

    # Test decoder
    decoder_path = model_dir / "decoder.mlmodel"
    decoder_success = False
    if decoder_path.exists():
        decoder_success, decoder_time = test_decoder_inference(decoder_path)
    else:
        print("Decoder model not found")

    # Summary
    print("\n=== Test Summary ===")
    print(f"Encoder: {'✓' if encoder_success else '✗'}")
    print(f"Decoder: {'✓' if decoder_success else '✗'}")

    if encoder_success and decoder_success:
        print("✅ All models loaded and functional")
    else:
        print("❌ Some models failed - check conversion process")
        exit(1)

    # Benchmarks
    if args.benchmark:
        if encoder_success:
            benchmark_model(encoder_path, "Encoder")
        if decoder_success:
            benchmark_model(decoder_path, "Decoder")

    # ANE check
    if args.check_ane:
        check_ane_utilization()

    print("\n=== Next Steps ===")
    print("1. Integrate with Rust code: apple-silicon/src/ane/models/whisper_model.rs")
    print("2. Build Swift bridge: cd coreml-bridge && swift build")
    print("3. Test ASR enricher: cargo test --package enrichers")
    print("4. Run video ingestor tests")


if __name__ == "__main__":
    main()
