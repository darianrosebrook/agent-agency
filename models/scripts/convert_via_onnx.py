#!/usr/bin/env python3
"""
Convert EmbeddingGemma model via ONNX intermediate format

This script converts PyTorch → ONNX → CoreML for better compatibility.
The ONNX intermediate format often resolves CoreML conversion issues.

Author: @darianrosebrook
Date: November 2025
"""

import os
import sys
import argparse
import numpy as np
from pathlib import Path

try:
    import torch
    import coremltools as ct
    from transformers import AutoModel, AutoTokenizer
except ImportError as e:
    print(f"❌ Missing required dependency: {e}")
    print("\n📦 Install dependencies:")
    print("  pip install torch transformers coremltools onnx")
    sys.exit(1)

try:
    import onnx
    import onnxruntime as ort
except ImportError:
    print("❌ Missing ONNX dependencies")
    print("\n📦 Install ONNX dependencies:")
    print("  pip install onnx onnxruntime")
    sys.exit(1)

from convert_embeddinggemma_to_coreml import patch_torch_operations, restore_torch_operations


class EmbeddingWrapper(torch.nn.Module):
    """Wrapper to extract only the embedding tensor from model output"""
    def __init__(self, base_model):
        super().__init__()
        self.base_model = base_model
    
    def forward(self, input_ids, attention_mask=None):
        if attention_mask is not None:
            outputs = self.base_model(input_ids=input_ids, attention_mask=attention_mask)
        else:
            outputs = self.base_model(input_ids=input_ids)
        # Extract last_hidden_state from the output
        if hasattr(outputs, 'last_hidden_state'):
            return outputs.last_hidden_state
        elif isinstance(outputs, tuple):
            return outputs[0]
        else:
            return outputs


def convert_via_onnx(
    model_id: str = "headwAI/embeddinggemma-300m",
    output_dir: Path = None,
    fp16: bool = True,
) -> Path:
    """
    Convert EmbeddingGemma model to CoreML via ONNX intermediate format
    
    Args:
        model_id: HuggingFace model identifier or local path
        output_dir: Directory to save the converted model
        fp16: Whether to use FP16 precision
    
    Returns:
        Path to the converted .mlmodel file
    """
    if output_dir is None:
        output_dir = Path(__file__).parent.parent / "coreml"
    
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print("=" * 70)
    print("EmbeddingGemma → ONNX → CoreML Conversion Pipeline")
    print("=" * 70)
    print(f"\n📥 Model: {model_id}")
    print(f"📁 Output: {output_dir}")
    print(f"🎯 Precision: {'FP16' if fp16 else 'FP32'}")
    print()
    
    # Patch operations before loading
    original_ops = patch_torch_operations()
    
    try:
        # Step 1: Load model
        print("[*] Loading model...")
        model_path = Path(model_id) if Path(model_id).exists() else None
        local_model_id = model_id if model_path is None else str(model_path)
        
        if model_path and model_path.is_dir():
            print(f"    Using local model path: {model_path}")
        else:
            print(f"    Downloading {model_id} from HuggingFace...")
        
        device = torch.device("cpu")
        torch.set_default_device(device)
        
        model = AutoModel.from_pretrained(local_model_id)
        model = model.to(device)
        model.eval()
        
        tokenizer = AutoTokenizer.from_pretrained(local_model_id)
        
        # Create wrapper
        wrapped_model = EmbeddingWrapper(model)
        wrapped_model.eval()
        
        print("    ✅ Model loaded")
        
        # Step 2: Get model config
        config = model.config
        hidden_size = getattr(config, 'hidden_size', getattr(config, 'd_model', 768))
        max_length = getattr(config, 'max_position_embeddings', 2048)
        
        print(f"\n[*] Model architecture:")
        print(f"    Hidden size: {hidden_size}")
        print(f"    Max length: {max_length}")
        
        # Step 3: Create example input for ONNX export
        print("\n[*] Creating example input for ONNX export...")
        sample_text = "This is a sample text for embedding generation."
        encoded = tokenizer(sample_text, return_tensors="pt", padding=False)
        example_input_ids = encoded["input_ids"].to(device)
        
        print(f"    Example input shape: {example_input_ids.shape}")
        
        # Step 4: Export to ONNX
        print("\n[*] Exporting to ONNX format...")
        onnx_path = output_dir / "embeddinggemma.onnx"
        
        try:
            # Export with dynamic axes for variable-length inputs
            torch.onnx.export(
                wrapped_model,
                example_input_ids,
                str(onnx_path),
                input_names=['input_ids'],
                output_names=['embeddings'],
                dynamic_axes={
                    'input_ids': {0: 'batch_size', 1: 'sequence_length'},
                    'embeddings': {0: 'batch_size', 1: 'sequence_length'},
                },
                opset_version=18,  # Use opset 18 to support bitwise operations
                do_constant_folding=True,
                export_params=True,
            )
            print(f"    ✅ ONNX model saved: {onnx_path}")
            
            # Verify ONNX model
            onnx_model = onnx.load(str(onnx_path))
            onnx.checker.check_model(onnx_model)
            print(f"    ✅ ONNX model verified")
            
        except Exception as e:
            print(f"❌ ONNX export failed: {e}")
            import traceback
            traceback.print_exc()
            sys.exit(1)
        
        # Step 5: Test ONNX model
        print("\n[*] Testing ONNX model...")
        try:
            ort_session = ort.InferenceSession(str(onnx_path))
            onnx_input = example_input_ids.numpy().astype(np.int64)
            onnx_outputs = ort_session.run(None, {'input_ids': onnx_input})
            
            print(f"    ✅ ONNX inference successful")
            print(f"    Output shape: {onnx_outputs[0].shape}")
            print(f"    Output dtype: {onnx_outputs[0].dtype}")
            
        except Exception as e:
            print(f"    ⚠️  ONNX inference failed: {e}")
            print(f"    Continuing with CoreML conversion anyway...")
        
        # Step 6: Convert ONNX to CoreML
        print("\n[*] Converting ONNX to CoreML...")
        
        # Load ONNX model
        onnx_model = onnx.load(str(onnx_path))
        
        # Define input specification
        input_spec = [ct.TensorType(
            name="input_ids",
            shape=(1, ct.RangeDim(lower_bound=1, upper_bound=max_length)),
            dtype=np.int32,
        )]
        
        compute_precision = ct.precision.FLOAT16 if fp16 else ct.precision.FLOAT32
        
        try:
            # Try converting with ONNX model object directly
            # CoreMLTools should auto-detect ONNX format
            mlmodel = ct.convert(
                onnx_model,
                source="auto",  # Auto-detect from ONNX model object
                inputs=input_spec,
                minimum_deployment_target=ct.target.macOS13,
                compute_precision=compute_precision,
                compute_units=ct.ComputeUnit.ALL,
                convert_to="neuralnetwork",  # Try Neural Network format first
            )
            print("    ✅ ONNX → CoreML conversion successful (Neural Network format)")
            output_extension = ".mlmodel"
            
        except Exception as e1:
            print(f"    ⚠️  Neural Network format failed: {e1}")
            print(f"    Attempting ML Program format...")
            try:
                mlmodel = ct.convert(
                    onnx_model,
                    source="auto",
                    inputs=input_spec,
                    minimum_deployment_target=ct.target.macOS13,
                    compute_precision=compute_precision,
                    compute_units=ct.ComputeUnit.ALL,
                    convert_to="mlprogram",
                )
                print("    ✅ ONNX → CoreML conversion successful (ML Program format)")
                output_extension = ".mlpackage"
            except Exception as e2:
                print(f"❌ ONNX → CoreML conversion failed: {e2}")
                print(f"\n💡 Trying alternative: Load ONNX model from file path...")
                try:
                    # Alternative: Use file path with explicit source
                    mlmodel = ct.convert(
                        str(onnx_path),
                        source="onnx",  # Explicitly specify ONNX source
                        inputs=input_spec,
                        minimum_deployment_target=ct.target.macOS13,
                        compute_precision=compute_precision,
                        compute_units=ct.ComputeUnit.ALL,
                        convert_to="mlprogram",
                    )
                    print("    ✅ ONNX → CoreML conversion successful (file path method)")
                    output_extension = ".mlpackage"
                except Exception as e3:
                    print(f"❌ All conversion methods failed: {e3}")
                    import traceback
                    traceback.print_exc()
                    sys.exit(1)
        
        # Step 7: Save CoreML model
        print("\n[*] Saving CoreML model...")
        output_path = output_dir / f"embeddinggemma_via_onnx{output_extension}"
        
        try:
            mlmodel.save(str(output_path))
            # Calculate size
            import os
            if output_path.is_dir():
                total_size = sum(
                    os.path.getsize(os.path.join(dirpath, filename))
                    for dirpath, dirnames, filenames in os.walk(output_path)
                    for filename in filenames
                )
            else:
                total_size = output_path.stat().st_size
            model_size = total_size / (1024 * 1024)  # MB
            print(f"    ✅ Model saved: {output_path}")
            print(f"    📦 Size: {model_size:.2f} MB")
            
        except Exception as e:
            print(f"❌ Failed to save model: {e}")
            sys.exit(1)
        
        # Step 8: Test CoreML model
        print("\n[*] Testing CoreML model...")
        try:
            test_model = ct.models.MLModel(str(output_path))
            
            # Test with actual tokenizer output
            test_text = "This is a test sentence."
            test_encoded = tokenizer(test_text, return_tensors="np", padding=False)
            test_input_ids = test_encoded["input_ids"].astype(np.int32)
            
            test_prediction = test_model.predict({"input_ids": test_input_ids})
            output_key = list(test_prediction.keys())[0]
            test_output = test_prediction[output_key]
            
            print(f"    ✅ CoreML inference successful!")
            print(f"    Output shape: {test_output.shape}")
            print(f"    Output dtype: {test_output.dtype}")
            print(f"    Output sample (first 5): {test_output.flatten()[:5]}")
            
        except Exception as e:
            print(f"    ⚠️  CoreML inference failed: {e}")
            print(f"    Model saved but may need further debugging")
        
        print("\n" + "=" * 70)
        print("✅ Conversion Complete!")
        print("=" * 70)
        print(f"\n📦 Output files:")
        print(f"   ONNX: {onnx_path}")
        print(f"   CoreML: {output_path}")
        
        return output_path
        
    finally:
        restore_torch_operations(original_ops)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Convert EmbeddingGemma to CoreML via ONNX")
    parser.add_argument(
        "--model-id",
        type=str,
        default="models/coreml/embeddinggemma-300m-raw",
        help="HuggingFace model identifier or local path"
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default=None,
        help="Output directory (default: models/coreml)"
    )
    parser.add_argument(
        "--fp32",
        action="store_true",
        help="Use FP32 precision instead of FP16"
    )
    
    args = parser.parse_args()
    
    output_dir = Path(args.output_dir) if args.output_dir else None
    
    convert_via_onnx(
        model_id=args.model_id,
        output_dir=output_dir,
        fp16=not args.fp32,
    )

