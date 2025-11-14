#!/usr/bin/env python3
"""
Simple Whisper to CoreML conversion script.
Focuses on getting encoder working first with minimal complexity.
"""

import os
import sys
import torch
import coremltools as ct
from pathlib import Path
import whisper

def create_simple_encoder_conversion():
    """Create a simple encoder conversion with minimal dependencies."""
    print("Loading Whisper model...")
    
    # Load Whisper model
    model = whisper.load_model("base")  # Start with base model for simplicity
    
    print("Creating dummy input...")
    # Create dummy input - Whisper expects mel spectrograms
    # Shape: [batch_size, n_mels, n_ctx]
    # For base model: n_mels=80, n_ctx=3000 (30 seconds)
    dummy_input = torch.randn(1, 80, 3000)
    
    print("Extracting encoder...")
    encoder = model.encoder
    
    print("Converting encoder to TorchScript...")
    # Use tracing instead of scripting for simplicity
    encoder.eval()
    with torch.no_grad():
        traced_encoder = torch.jit.trace(encoder, dummy_input)
    
    print("Converting to CoreML...")
    # Convert with minimal options
    try:
        coreml_model = ct.convert(
            traced_encoder,
            inputs=[ct.TensorType(name="input", shape=dummy_input.shape)],
            minimum_deployment_target=ct.target.macOS13,  # Use macOS 13 for better compatibility
            compute_units=ct.ComputeUnit.CPU_AND_GPU,  # Avoid ANE for now
        )
        
        # Save the model
        output_path = Path("models/coreml/whisper")
        output_path.mkdir(parents=True, exist_ok=True)
        
        model_path = output_path / "encoder_base.mlpackage"
        coreml_model.save(str(model_path))
        
        print(f"✅ Successfully converted encoder to {model_path}")
        print(f"Model size: {model_path.stat().st_size / (1024*1024):.1f} MB")
        
        return True
        
    except Exception as e:
        print(f"❌ Conversion failed: {e}")
        return False

def test_conversion():
    """Test the converted model."""
    try:
        import coremltools as ct
        
        model_path = Path("models/coreml/whisper/encoder_base.mlpackage")
        if not model_path.exists():
            print("❌ No model to test")
            return False
            
        print("Loading CoreML model...")
        model = ct.models.MLModel(str(model_path))
        
        print("Model metadata:")
        print(f"  - Input: {model.input_description}")
        print(f"  - Output: {model.output_description}")
        
        print("✅ Model loaded successfully")
        return True
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        return False

if __name__ == "__main__":
    print("=== Simple Whisper CoreML Conversion ===")
    
    # Set up environment
    os.environ["LD_LIBRARY_PATH"] = "/Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu/lib:" + os.environ.get("LD_LIBRARY_PATH", "")
    os.environ["TORCH_LIBRARY_PATH"] = "/Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu"
    
    success = create_simple_encoder_conversion()
    
    if success:
        print("\n=== Testing Conversion ===")
        test_conversion()
    else:
        print("\n❌ Conversion failed")
        sys.exit(1)
