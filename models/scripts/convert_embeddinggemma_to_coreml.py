#!/usr/bin/env python3
"""
Convert EmbeddingGemma model from HuggingFace to CoreML format

This script downloads the embeddinggemma-2b-002 model (or similar) from HuggingFace
and converts it to CoreML format for ANE-accelerated inference on Apple Silicon.

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
    from coremltools.models.neural_network import quantization_utils
except ImportError as e:
    print(f"❌ Missing required dependency: {e}")
    print("\n📦 Install dependencies:")
    print("  pip install torch transformers coremltools")
    sys.exit(1)

# Import model surgery utilities
try:
    from coreml_surgery import BitwiseOperationReplacer
    SURGERY_AVAILABLE = True
except ImportError:
    SURGERY_AVAILABLE = False
    print("⚠️  Warning: coreml_surgery module not found. Bitwise operation replacement disabled.")


def patch_torch_operations():
    """
    Globally patch torch operations to replace CoreML-incompatible operations.
    This must be called before model loading to ensure operations are replaced
    during tracing.
    """
    if not SURGERY_AVAILABLE:
        return None
    
    original_ops = {}
    
    # Store original operations
    original_ops['bitwise_or'] = torch.bitwise_or
    original_ops['bitwise_and'] = torch.bitwise_and
    original_ops['tensor_or'] = torch.Tensor.__or__
    original_ops['tensor_and'] = torch.Tensor.__and__
    original_ops['tensor_new_ones'] = torch.Tensor.new_ones
    
    def logical_or_replacement(a, b):
        """Replace bitwise_or with logical_or for CoreML compatibility"""
        if isinstance(a, torch.Tensor) and isinstance(b, torch.Tensor):
            if a.dtype == torch.bool and b.dtype == torch.bool:
                return torch.logical_or(a, b)
            # Convert to bool, apply logical_or, convert back to original dtype
            result = torch.logical_or(a.bool(), b.bool())
            return result.to(a.dtype)
        return original_ops['bitwise_or'](a, b)
    
    def logical_and_replacement(a, b):
        """Replace bitwise_and with logical_and for CoreML compatibility"""
        if isinstance(a, torch.Tensor) and isinstance(b, torch.Tensor):
            if a.dtype == torch.bool and b.dtype == torch.bool:
                return torch.logical_and(a, b)
            # Convert to bool, apply logical_and, convert back to original dtype
            result = torch.logical_and(a.bool(), b.bool())
            return result.to(a.dtype)
        return original_ops['bitwise_and'](a, b)
    
    def tensor_or_replacement(self, other):
        """Replace tensor __or__ operator"""
        if isinstance(other, torch.Tensor):
            if self.dtype == torch.bool and other.dtype == torch.bool:
                return torch.logical_or(self, other)
            result = torch.logical_or(self.bool(), other.bool())
            return result.to(self.dtype)
        return original_ops['tensor_or'](self, other)
    
    def tensor_and_replacement(self, other):
        """Replace tensor __and__ operator"""
        if isinstance(other, torch.Tensor):
            if self.dtype == torch.bool and other.dtype == torch.bool:
                return torch.logical_and(self, other)
            result = torch.logical_and(self.bool(), other.bool())
            return result.to(self.dtype)
        return original_ops['tensor_and'](self, other)
    
    def tensor_new_ones_replacement(self, *args, **kwargs):
        """Replace tensor.new_ones() with torch.full() for CoreML compatibility"""
        # new_ones creates a tensor with same dtype and device, filled with 1
        # Extract size and kwargs - handle both positional and keyword args
        if args:
            size = args[0]
            dtype = kwargs.get('dtype', self.dtype)
            device = kwargs.get('device', self.device)
            requires_grad = kwargs.get('requires_grad', False)
        else:
            size = kwargs.get('size', self.shape)
            dtype = kwargs.get('dtype', self.dtype)
            device = kwargs.get('device', self.device)
            requires_grad = kwargs.get('requires_grad', False)
        
        # Use torch.full with explicit dtype to ensure correct dtype preservation
        # For integer dtypes, use integer value 1; for float, use 1.0
        fill_value = 1 if dtype in (torch.int32, torch.int64, torch.int16, torch.int8, torch.uint8) else 1.0
        result = torch.full(size, fill_value, dtype=dtype, device=device, requires_grad=requires_grad)
        return result
    
    # Apply patches
    torch.bitwise_or = logical_or_replacement
    torch.bitwise_and = logical_and_replacement
    torch.Tensor.__or__ = tensor_or_replacement
    torch.Tensor.__and__ = tensor_and_replacement
    torch.Tensor.new_ones = tensor_new_ones_replacement
    
    return original_ops


def restore_torch_operations(original_ops):
    """Restore original torch operations"""
    if original_ops:
        torch.bitwise_or = original_ops['bitwise_or']
        torch.bitwise_and = original_ops['bitwise_and']
        torch.Tensor.__or__ = original_ops['tensor_or']
        torch.Tensor.__and__ = original_ops['tensor_and']
        torch.Tensor.new_ones = original_ops['tensor_new_ones']


def convert_embeddinggemma_to_coreml(
    model_id: str = "headwAI/embeddinggemma-300m",
    output_dir: Path = None,
    quantize: bool = True,
    fp16: bool = True,
) -> Path:
    """
    Convert EmbeddingGemma model to CoreML format
    
    Args:
        model_id: HuggingFace model identifier (e.g., "google/gemma-2-2b" or "google/gemma-2-2b-it")
        output_dir: Directory to save the converted model
        quantize: Whether to apply INT8 quantization (reduces size, may affect accuracy)
        fp16: Whether to use FP16 precision (recommended for ANE acceleration)
    
    Returns:
        Path to the converted .mlmodel file
    """
    # Patch torch operations BEFORE loading model
    original_ops = patch_torch_operations()
    
    try:
        return _convert_embeddinggemma_to_coreml_internal(model_id, output_dir, quantize, fp16)
    finally:
        # Restore original operations
        restore_torch_operations(original_ops)


def _convert_embeddinggemma_to_coreml_internal(
    model_id: str = "headwAI/embeddinggemma-300m",
    output_dir: Path = None,
    quantize: bool = True,
    fp16: bool = True,
) -> Path:
    if output_dir is None:
        output_dir = Path(__file__).parent.parent / "coreml"
    
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print("=" * 70)
    print("EmbeddingGemma → CoreML Conversion Pipeline")
    print("=" * 70)
    print(f"\n📥 Model: {model_id}")
    print(f"📁 Output: {output_dir}")
    print(f"🎯 Precision: {'FP16' if fp16 else 'FP32'}")
    print(f"⚖️  Quantization: {'INT8' if quantize else 'None'}")
    print()
    
    # Step 1: Load model and tokenizer from HuggingFace or local path
    print("[*] Loading model from HuggingFace or local path...")
    
    # Check if model_id is a local path
    model_path = Path(model_id) if Path(model_id).exists() else None
    local_model_id = model_id
    
    if model_path and model_path.is_dir():
        print(f"    Using local model path: {model_path}")
        local_model_id = str(model_path)
    else:
        print(f"    Downloading {model_id} from HuggingFace...")
    
    try:
        # For embedding models, try SentenceTransformer first (best for embedding models)
        try:
            from sentence_transformers import SentenceTransformer
            print(f"    Trying SentenceTransformer (optimal for embedding models)...")
            # SentenceTransformer handles the full pipeline including tokenization
            sentence_model = SentenceTransformer(local_model_id)
            model = sentence_model[0].auto_model  # Extract the underlying AutoModel
            tokenizer = sentence_model.tokenizer
            print(f"    ✅ Model loaded via SentenceTransformer")
        except ImportError:
            print(f"    SentenceTransformer not available, using transformers...")
            raise
        except Exception as e1:
            print(f"    ⚠️  SentenceTransformer failed: {e1}")
            print(f"    Falling back to transformers library...")
            
            # Load tokenizer first
            tokenizer = AutoTokenizer.from_pretrained(local_model_id)
            print(f"    ✅ Tokenizer loaded")
            
            # Load model - embedding models may use different base classes
            # Try AutoModel first (works for most embedding models)
            try:
                model = AutoModel.from_pretrained(local_model_id)
                print(f"    ✅ Model loaded via AutoModel")
            except Exception as e2:
                print(f"    ⚠️  AutoModel failed: {e2}")
                print(f"    Trying AutoModelForCausalLM...")
                from transformers import AutoModelForCausalLM
                model = AutoModelForCausalLM.from_pretrained(local_model_id)
                print(f"    ✅ Model loaded via AutoModelForCausalLM")
        
        model.eval()
        print(f"    ✅ Model set to evaluation mode")
        
        # Force CPU device to avoid MPS tracing issues
        device = torch.device("cpu")
        model = model.to(device)
        print(f"    ✅ Model moved to CPU device")
        
    except Exception as e:
        print(f"❌ Failed to load model: {e}")
        print("\n💡 Troubleshooting:")
        print(f"   1. For local models: Ensure path exists and contains config.json")
        print(f"   2. Install sentence-transformers: pip install sentence-transformers")
        print(f"   3. Download manually: python -c 'from transformers import AutoModel; AutoModel.from_pretrained(\"{model_id}\").save_pretrained(\"local_model\")'")
        sys.exit(1)
    
    # Step 2: Determine model input/output shapes
    print("\n[*] Analyzing model architecture...")
    
    # Get model config to determine dimensions
    config = model.config
    hidden_size = getattr(config, 'hidden_size', getattr(config, 'd_model', 768))
    vocab_size = getattr(config, 'vocab_size', 256000)
    max_length = getattr(config, 'max_position_embeddings', 512)
    
    print(f"    Hidden size: {hidden_size}")
    print(f"    Vocab size: {vocab_size}")
    print(f"    Max length: {max_length}")
    
    # Step 3: Create example input for tracing
    print("\n[*] Creating example input for model tracing...")
    
    # Initialize variables
    has_attention_mask = False
    example_attention_mask = None
    
    # Use actual tokenizer to create proper input format
    try:
        sample_text = "This is a sample text for embedding generation."
        encoded = tokenizer(
            sample_text,
            return_tensors="pt",
            padding=True,
            truncation=True,
            max_length=max_length
        )
        example_input_ids = encoded["input_ids"]
        
        # Check if attention_mask is needed
        has_attention_mask = "attention_mask" in encoded
        if has_attention_mask:
            example_attention_mask = encoded["attention_mask"]
            print(f"    Example input_ids shape: {example_input_ids.shape}")
            print(f"    Example attention_mask shape: {example_attention_mask.shape}")
            print(f"    Using input_ids + attention_mask for tracing")
        else:
            print(f"    Example input_ids shape: {example_input_ids.shape}")
            print(f"    Using input_ids only for tracing")
    except Exception as e:
        print(f"    ⚠️  Tokenizer encoding failed: {e}")
        print(f"    Using random input_ids as fallback...")
        # Fallback: create random token IDs
        example_input_ids = torch.randint(0, min(vocab_size, 50000), (1, min(max_length, 128)))
        example_attention_mask = None
        has_attention_mask = False
        print(f"    Example input shape: {example_input_ids.shape}")
    
    # Step 4: Trace the model
    print("\n[*] Tracing model with TorchScript...")
    
    # Force CPU device to avoid MPS issues
    device = torch.device("cpu")
    model = model.to(device)
    example_input_ids = example_input_ids.to(device)
    if example_attention_mask is not None:
        example_attention_mask = example_attention_mask.to(device)
    print(f"    Using device: {device}")
    
    # Wrap model to return only the embedding tensor (not the full output object)
    class EmbeddingWrapper(torch.nn.Module):
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
    
    wrapped_model = EmbeddingWrapper(model)
    wrapped_model.eval()
    
    # For embedding models, we need to handle the forward pass correctly
    # Operations are already patched globally, so tracing will capture the replacements
    traced_with_attention = False
    if SURGERY_AVAILABLE:
        print("    🔧 Bitwise operation replacement enabled (patched before model loading)")
    
    try:
        with torch.no_grad():
            # Try with attention_mask if available
            if has_attention_mask and example_attention_mask is not None:
                try:
                    # Try forward pass with attention_mask
                    _ = wrapped_model(example_input_ids, example_attention_mask)
                    traced_model = torch.jit.trace(
                        wrapped_model,
                        (example_input_ids, example_attention_mask)
                    )
                    traced_with_attention = True
                    print("    ✅ Model traced successfully (with attention_mask)")
                except Exception as e_mask:
                    print(f"    ⚠️  Tracing with attention_mask failed: {e_mask}")
                    print(f"    Trying without attention_mask...")
                    # Fallback to input_ids only
                    traced_model = torch.jit.trace(wrapped_model, example_input_ids)
                    traced_with_attention = False
                    print("    ✅ Model traced successfully (input_ids only)")
            else:
                # Try input_ids only
                traced_model = torch.jit.trace(wrapped_model, example_input_ids)
                traced_with_attention = False
                print("    ✅ Model traced successfully")
    except Exception as e:
        print(f"❌ Tracing failed: {e}")
        print("\n💡 Try using torch.jit.script() instead of trace()")
        try:
            with torch.no_grad():
                traced_model = torch.jit.script(model)
            print("    ✅ Model scripted successfully")
        except Exception as e2:
            print(f"❌ Scripting also failed: {e2}")
            print("\n💡 Alternative: Convert SentenceTransformer pipeline directly")
            print("   Consider using sentence-transformers export functionality")
            sys.exit(1)
    
    # Step 5: Convert to CoreML
    print("\n[*] Converting to CoreML format...")
    
    # Define input specification based on what was actually traced
    # Use INT32 for input_ids (token IDs are integers)
    if traced_with_attention:
        input_spec = [
            ct.TensorType(
                name="input_ids",
                shape=(1, ct.RangeDim(lower_bound=1, upper_bound=max_length)),
                dtype=np.int32,
            ),
            ct.TensorType(
                name="attention_mask",
                shape=(1, ct.RangeDim(lower_bound=1, upper_bound=max_length)),
                dtype=np.int32,
            ),
        ]
    else:
        input_spec = [ct.TensorType(
            name="input_ids",
            shape=(1, ct.RangeDim(lower_bound=1, upper_bound=max_length)),
            dtype=np.int32,
        )]
    
    try:
        # Convert with FP16 if requested
        compute_precision = ct.precision.FLOAT16 if fp16 else ct.precision.FLOAT32
        
        # Try conversion with Neural Network format first (more compatible, fewer runtime issues)
        output_extension = ".mlmodel"
        try:
            mlmodel = ct.convert(
                traced_model,
                inputs=input_spec,
                outputs=None,  # Auto-detect outputs
                minimum_deployment_target=ct.target.macOS13,  # macOS 13+ for ANE support
                compute_precision=compute_precision,
                compute_units=ct.ComputeUnit.ALL,  # Allow ANE, GPU, CPU
                convert_to="neuralnetwork",  # Use Neural Network format (more compatible)
            )
            print("    ✅ CoreML conversion successful (Neural Network format)")
        except Exception as e1:
            print(f"    ⚠️  Neural Network format failed: {e1}")
            print(f"    Attempting conversion with ML Program format...")
            # Try with ML Program format
            try:
                mlmodel = ct.convert(
                    traced_model,
                    inputs=input_spec,
                    outputs=None,  # Auto-detect outputs
                    minimum_deployment_target=ct.target.macOS13,
                    compute_precision=compute_precision,
                    compute_units=ct.ComputeUnit.ALL,
                    convert_to="mlprogram",  # Use newer ML Program format
                )
                print("    ✅ CoreML conversion successful (ML Program format)")
                output_extension = ".mlpackage"
            except Exception as e2:
                print(f"    ❌ ML Program format also failed: {e2}")
                raise
        
    except Exception as e:
        print(f"❌ CoreML conversion failed: {e}")
        print("\n💡 Troubleshooting:")
        print("    - Ensure coremltools is up to date: pip install --upgrade coremltools")
        print("    - Check model architecture compatibility")
        print("    - Try using ONNX as intermediate format: pip install onnx onnxruntime")
        sys.exit(1)
    
    # Step 6: Apply quantization if requested
    if quantize:
        print("\n[*] Applying INT8 quantization...")
        try:
            mlmodel = quantization_utils.quantize_weights(mlmodel, nbits=8)
            print("    ✅ Quantization applied")
        except Exception as e:
            print(f"    ⚠️  Quantization failed: {e}")
            print("    Continuing without quantization...")
    
    # Step 7: Save the model
    print("\n[*] Saving CoreML model...")
    
    # Determine output extension based on format
    if 'output_extension' not in locals():
        # Default to mlpackage if ML Program, mlmodel if Neural Network
        output_extension = ".mlpackage" if hasattr(mlmodel, '_spec') and mlmodel._spec.WhichOneof('Type') == 'mlProgram' else ".mlmodel"
    
    output_path = output_dir / f"embeddinggemma{output_extension}"
    
    try:
        mlmodel.save(str(output_path))
        # Calculate size (mlpackage is a directory)
        import os
        total_size = sum(
            os.path.getsize(os.path.join(dirpath, filename))
            for dirpath, dirnames, filenames in os.walk(output_path)
            for filename in filenames
        )
        model_size = total_size / (1024 * 1024)  # MB
        print(f"    ✅ Model saved: {output_path}")
        print(f"    📦 Size: {model_size:.2f} MB")
        
    except Exception as e:
        print(f"❌ Failed to save model: {e}")
        sys.exit(1)
    
    # Step 8: Save tokenizer separately (needed for inference)
    tokenizer_path = output_dir / "embeddinggemma_tokenizer"
    try:
        tokenizer.save_pretrained(str(tokenizer_path))
        print(f"    ✅ Tokenizer saved: {tokenizer_path}")
    except Exception as e:
        print(f"    ⚠️  Failed to save tokenizer: {e}")
    
    print("\n" + "=" * 70)
    print("✅ Conversion Complete!")
    print("=" * 70)
    print(f"\n📦 Output files:")
    print(f"   Model: {output_path}")
    print(f"   Tokenizer: {tokenizer_path}")
    print(f"\n💡 Next steps:")
    print(f"   1. Test the model with coremltools:")
    print(f"      python3 -c 'import coremltools as ct; model = ct.models.MLModel(\"{output_path}\"); print(model)'")
    print(f"   2. Update COREML_EMBEDDING_MODEL_PATH environment variable")
    print(f"   3. The CoreMLEmbeddingProvider will auto-detect this model")
    print()
    
    return output_path


def main():
    parser = argparse.ArgumentParser(
        description="Convert EmbeddingGemma model to CoreML format"
    )
    parser.add_argument(
        "--model-id",
        default="headwAI/embeddinggemma-300m",
        help="HuggingFace model identifier (default: headwAI/embeddinggemma-300m)",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Output directory (default: models/coreml)",
    )
    parser.add_argument(
        "--no-quantize",
        action="store_true",
        help="Skip INT8 quantization (larger file, better accuracy)",
    )
    parser.add_argument(
        "--fp32",
        action="store_true",
        help="Use FP32 precision instead of FP16",
    )
    
    args = parser.parse_args()
    
    convert_embeddinggemma_to_coreml(
        model_id=args.model_id,
        output_dir=args.output_dir,
        quantize=not args.no_quantize,
        fp16=not args.fp32,
    )


if __name__ == "__main__":
    main()

