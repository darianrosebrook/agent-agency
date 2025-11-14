#!/usr/bin/env python3
"""
Whisper Decoder to CoreML conversion script.
Converts the Whisper decoder component to CoreML format.
"""

import os
import sys
import torch
import coremltools as ct
from pathlib import Path
import whisper

def create_decoder_conversion():
    """Create a decoder conversion with proper input shapes."""
    print("Loading Whisper model...")
    
    # Load Whisper model
    model = whisper.load_model("base")  # Start with base model for simplicity
    
    print("Creating dummy inputs for decoder...")
    # Decoder expects: tokens, encoder outputs
    batch_size = 1
    n_ctx = 448  # Max tokens for Whisper base
    n_state = model.dims.n_audio_state  # 512 for base model
    
    # Token input (previous tokens)
    tokens = torch.randint(0, model.dims.n_vocab, (batch_size, n_ctx))
    
    # Encoder output (from encoder) - shape depends on audio length
    # For 30 seconds of audio, this becomes ~1500 time steps
    encoder_output = torch.randn(batch_size, n_state, 1500)
    
    print("Extracting decoder...")
    decoder = model.decoder
    
    print("Converting decoder to TorchScript...")
    # Use tracing for decoder
    decoder.eval()
    with torch.no_grad():
        traced_decoder = torch.jit.trace(decoder, (tokens, encoder_output))
    
    print("Converting to CoreML...")
    # Convert with proper input specifications
    try:
        coreml_model = ct.convert(
            traced_decoder,
            inputs=[
                ct.TensorType(name="tokens", shape=tokens.shape, dtype=ct.int32),
                ct.TensorType(name="encoder_output", shape=encoder_output.shape)
            ],
            minimum_deployment_target=ct.target.macOS13,  # Use macOS 13 for better compatibility
            compute_units=ct.ComputeUnit.CPU_AND_GPU,  # Avoid ANE for now
        )
        
        # Save the model
        output_path = Path("models/coreml/whisper")
        output_path.mkdir(parents=True, exist_ok=True)
        
        model_path = output_path / "decoder_base.mlpackage"
        coreml_model.save(str(model_path))
        
        print(f"✅ Successfully converted decoder to {model_path}")
        print(f"Model size: {model_path.stat().st_size / (1024*1024):.1f} MB")
        
        return True
        
    except Exception as e:
        print(f"❌ Conversion failed: {e}")
        return False

def test_decoder_conversion():
    """Test the converted decoder model."""
    try:
        import coremltools as ct
        
        model_path = Path("models/coreml/whisper/decoder_base.mlpackage")
        if not model_path.exists():
            print("❌ No decoder model to test")
            return False
            
        print("Loading CoreML decoder model...")
        model = ct.models.MLModel(str(model_path))
        
        print("Decoder model metadata:")
        print(f"  - Inputs: {model.input_description}")
        print(f"  - Output: {model.output_description}")
        
        print("✅ Decoder model loaded successfully")
        return True
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
        return False

if __name__ == "__main__":
    print("=== Whisper Decoder CoreML Conversion ===")
    
    # Set up environment
    os.environ["LD_LIBRARY_PATH"] = "/Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu/lib:" + os.environ.get("LD_LIBRARY_PATH", "")
    os.environ["TORCH_LIBRARY_PATH"] = "/Users/darianrosebrook/Desktop/Projects/agent-agency/libtorch-cpu"
    os.environ["KMP_DUPLICATE_LIB_OK"] = "TRUE"
    
    success = create_decoder_conversion()
    
    if success:
        print("\n=== Testing Decoder Conversion ===")
        test_decoder_conversion()
    else:
        print("\n❌ Decoder conversion failed")
        sys.exit(1)
