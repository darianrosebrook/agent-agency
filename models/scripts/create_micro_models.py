#!/usr/bin/env python3
"""
Create micro-models for ANE performance baseline testing.

These small models help separate "CoreML+ANE as platform" performance
from "Mistral 7B converted to CoreML" performance.

Models created:
1. Single dense linear layer (matmul) - similar width to Mistral hidden size
2. Single self-attention block - tests attention ops on ANE
"""

import os
import sys
import torch
import torch.nn as nn
from pathlib import Path

try:
    import coremltools as ct
    from coremltools.models.neural_network import quantization_utils
except ImportError:
    print("ERROR: coremltools not installed. Install with: pip install coremltools")
    sys.exit(1)


def create_dense_layer_model(output_dir: Path, hidden_size: int = 4096):
    """
    Create a single dense linear layer model.

    This tests pure matrix multiplication on ANE, which should show
    strong speedup if ANE is working correctly.

    Args:
        output_dir: Directory to save the model
        hidden_size: Size of hidden dimension (default 4096 matches Mistral 7B)

    Note:
        - Creates FP32 ML Program (CoreML default for traced models)
        - FP32 pipeline may understate ANE ceiling; FP16/INT8 variants would show better speedup
        - Consider adding FP16/INT8 variants for "best case" ANE performance
    """
    print(f"Creating dense layer model (hidden_size={hidden_size})...")

    class DenseLayer(nn.Module):
        def __init__(self, hidden_size):
            super().__init__()
            self.linear = nn.Linear(hidden_size, hidden_size)
            self.activation = nn.GELU()

        def forward(self, x):
            return self.activation(self.linear(x))

    model = DenseLayer(hidden_size)
    model.eval()

    # Create example input
    # [batch, seq_len, hidden]
    example_input = torch.randn(1, 128, hidden_size)

    # Trace the model (use no_grad for tracing)
    with torch.no_grad():
        traced_model = torch.jit.trace(model, example_input)

    # Convert to CoreML
    # Use convert_to="mlprogram" explicitly for modern CoreML (macOS 13+)
    # compute_units is a hint at conversion time; actual device selection
    # happens at runtime via AgentBridge compute unit configuration
    # Note: CoreML infers outputs from traced graph, so outputs=[...] is optional
    mlmodel = ct.convert(
        traced_model,
        inputs=[ct.TensorType(name="input", shape=example_input.shape)],
        convert_to="mlprogram",  # Explicit ML Program format for ANE support
        compute_units=ct.ComputeUnit.ALL,  # Allow ANE (hint for converter)
        minimum_deployment_target=ct.target.macOS13,  # ANE support
    )

    # Add metadata
    mlmodel.author = "Agent Agency V3"
    mlmodel.short_description = f"Single dense layer (hidden_size={hidden_size}) for ANE baseline testing"
    mlmodel.version = "1.0"

    # Save model
    output_path = output_dir / "micro_dense_layer.mlpackage"
    mlmodel.save(str(output_path))

    print(f"✅ Saved dense layer model to: {output_path}")
    return output_path


def create_attention_block_model(output_dir: Path, hidden_size: int = 4096, num_heads: int = 32):
    """
    Create a single self-attention block model.

    This tests attention operations (QK^T, softmax, V projection) on ANE.
    Attention is a key component of transformers and should benefit from ANE.

    Args:
        output_dir: Directory to save the model
        hidden_size: Size of hidden dimension
        num_heads: Number of attention heads

    Note:
        - Creates FP32 ML Program (CoreML default for traced models)
        - FP32 pipeline may understate ANE ceiling; FP16/INT8 variants would show better speedup
        - Includes causal masking to test masked attention ANE support
        - If parts of masked attention fall back to CPU, that's signal (ANE limitation), not noise
    """
    print(
        f"Creating attention block model (hidden_size={hidden_size}, heads={num_heads})...")

    # Validate head dimension is divisible
    assert hidden_size % num_heads == 0, f"hidden_size ({hidden_size}) must be divisible by num_heads ({num_heads})"

    head_dim = hidden_size // num_heads

    class AttentionBlock(nn.Module):
        def __init__(self, hidden_size, num_heads):
            super().__init__()
            self.hidden_size = hidden_size
            self.num_heads = num_heads
            self.head_dim = hidden_size // num_heads

            # Q, K, V projections
            self.q_proj = nn.Linear(hidden_size, hidden_size)
            self.k_proj = nn.Linear(hidden_size, hidden_size)
            self.v_proj = nn.Linear(hidden_size, hidden_size)
            self.o_proj = nn.Linear(hidden_size, hidden_size)

            # Layer norm
            self.layer_norm = nn.LayerNorm(hidden_size)

        def forward(self, x):
            # Layer norm
            x_norm = self.layer_norm(x)

            # Q, K, V projections
            q = self.q_proj(x_norm)
            k = self.k_proj(x_norm)
            v = self.v_proj(x_norm)

            # Reshape for multi-head attention
            batch_size, seq_len, _ = x.shape
            q = q.view(batch_size, seq_len, self.num_heads,
                       self.head_dim).transpose(1, 2)
            k = k.view(batch_size, seq_len, self.num_heads,
                       self.head_dim).transpose(1, 2)
            v = v.view(batch_size, seq_len, self.num_heads,
                       self.head_dim).transpose(1, 2)

            # Attention scores: Q @ K^T
            scores = torch.matmul(q, k.transpose(-2, -1)) / \
                (self.head_dim ** 0.5)

            # Causal mask (lower triangular)
            # Using -inf is standard; if conversion issues occur, try -1e4 instead
            mask = torch.triu(torch.ones(seq_len, seq_len), diagonal=1).bool()
            scores = scores.masked_fill(
                mask.unsqueeze(0).unsqueeze(0), float('-inf'))

            # Softmax
            attn_weights = torch.softmax(scores, dim=-1)

            # Apply attention to values
            attn_output = torch.matmul(attn_weights, v)

            # Reshape and output projection
            attn_output = attn_output.transpose(1, 2).contiguous().view(
                batch_size, seq_len, self.hidden_size
            )
            output = self.o_proj(attn_output)

            # Residual connection
            return x + output

    model = AttentionBlock(hidden_size, num_heads)
    model.eval()

    # Create example input
    # [batch, seq_len, hidden]
    example_input = torch.randn(1, 128, hidden_size)

    # Trace the model (use no_grad for tracing)
    with torch.no_grad():
        traced_model = torch.jit.trace(model, example_input)

    # Convert to CoreML
    # Use convert_to="mlprogram" explicitly for modern CoreML (macOS 13+)
    # compute_units is a hint at conversion time; actual device selection
    # happens at runtime via AgentBridge compute unit configuration
    # Note: CoreML infers outputs from traced graph, so outputs=[...] is optional
    mlmodel = ct.convert(
        traced_model,
        inputs=[ct.TensorType(name="input", shape=example_input.shape)],
        convert_to="mlprogram",  # Explicit ML Program format for ANE support
        compute_units=ct.ComputeUnit.ALL,  # Allow ANE (hint for converter)
        minimum_deployment_target=ct.target.macOS13,  # ANE support
    )

    # Add metadata
    mlmodel.author = "Agent Agency V3"
    mlmodel.short_description = (
        f"Single attention block (hidden_size={hidden_size}, heads={num_heads}) "
        "for ANE baseline testing"
    )
    mlmodel.version = "1.0"

    # Save model
    output_path = output_dir / "micro_attention_block.mlpackage"
    mlmodel.save(str(output_path))

    print(f"✅ Saved attention block model to: {output_path}")
    return output_path


def main():
    """Create micro-models for ANE baseline testing."""
    # Determine output directory
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent
    models_dir = project_root / "models" / "coreml" / "micro"

    # Create output directory
    models_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("Creating micro-models for ANE baseline testing")
    print("=" * 60)
    print(f"Output directory: {models_dir}\n")

    # Create models
    try:
        dense_path = create_dense_layer_model(models_dir, hidden_size=4096)
        print()

        attention_path = create_attention_block_model(
            models_dir, hidden_size=4096, num_heads=32
        )
        print()

        print("=" * 60)
        print("✅ Successfully created micro-models:")
        print(f"   - Dense layer: {dense_path}")
        print(f"   - Attention block: {attention_path}")
        print("=" * 60)
        print("\nNext steps:")
        print("1. Run benchmarks: cargo test --test ane_performance_benchmarks")
        print("2. Compare micro-model ANE speedup vs Mistral 7B")
        print("3. If micro-models show 2-3x speedup → runtime path is fine")
        print("4. If micro-models show ~1.1x → platform limit for FP16 workloads")

    except Exception as e:
        print(f"\n❌ Error creating models: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
