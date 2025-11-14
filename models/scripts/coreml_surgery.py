"""
Model surgery utilities for CoreML compatibility

Replaces unsupported operations (like bitwise_or) with CoreML-compatible alternatives.
"""

import torch
import torch.nn as nn
from typing import Any, Callable, Optional


class BitwiseOperationReplacer:
    """Context manager that replaces bitwise operations with CoreML-compatible alternatives"""
    
    def __init__(self):
        self.original_ops = {}
        self._patched = False
    
    def _logical_or_replacement(self, a: torch.Tensor, b: torch.Tensor) -> torch.Tensor:
        """
        Replace bitwise_or with logical_or for CoreML compatibility.
        
        For boolean tensors, logical_or is equivalent to bitwise_or.
        For integer tensors, we convert to bool, apply logical_or, then convert back.
        """
        # Handle tensor arguments
        if isinstance(a, torch.Tensor) and isinstance(b, torch.Tensor):
            # If both are boolean, use logical_or directly
            if a.dtype == torch.bool and b.dtype == torch.bool:
                return torch.logical_or(a, b)
            
            # Convert to boolean, apply logical_or, then convert back
            original_dtype = a.dtype
            a_bool = a.bool() if a.dtype != torch.bool else a
            b_bool = b.bool() if b.dtype != torch.bool else b
            
            result_bool = torch.logical_or(a_bool, b_bool)
            
            # Convert back to original dtype
            if original_dtype != torch.bool:
                return result_bool.to(original_dtype)
            return result_bool
        
        # Fallback to original for non-tensor cases
        return torch.bitwise_or(a, b)
    
    def _logical_and_replacement(self, a: torch.Tensor, b: torch.Tensor) -> torch.Tensor:
        """Replace bitwise_and with logical_and for CoreML compatibility"""
        if isinstance(a, torch.Tensor) and isinstance(b, torch.Tensor):
            if a.dtype == torch.bool and b.dtype == torch.bool:
                return torch.logical_and(a, b)
            
            original_dtype = a.dtype
            a_bool = a.bool() if a.dtype != torch.bool else a
            b_bool = b.bool() if b.dtype != torch.bool else b
            
            result_bool = torch.logical_and(a_bool, b_bool)
            
            if original_dtype != torch.bool:
                return result_bool.to(original_dtype)
            return result_bool
        
        return torch.bitwise_and(a, b)
    
    def __enter__(self):
        """Patch bitwise operations"""
        if self._patched:
            return self
        
        self.original_ops['bitwise_or'] = torch.bitwise_or
        self.original_ops['bitwise_and'] = torch.bitwise_and
        
        # Replace with logical operations
        torch.bitwise_or = self._logical_or_replacement
        torch.bitwise_and = self._logical_and_replacement
        
        self._patched = True
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Restore original operations"""
        if self._patched:
            torch.bitwise_or = self.original_ops['bitwise_or']
            torch.bitwise_and = self.original_ops['bitwise_and']
            self._patched = False


class CoreMLCompatibleWrapper(nn.Module):
    """
    Wraps a PyTorch module to replace unsupported operations during forward pass.
    
    This is useful for models that use operations CoreML doesn't support,
    like bitwise_or on boolean tensors.
    """
    
    def __init__(self, base_module: nn.Module):
        super().__init__()
        self.base_module = base_module
    
    def forward(self, *args, **kwargs):
        """Forward pass with operation replacements"""
        with BitwiseOperationReplacer():
            return self.base_module(*args, **kwargs)


def patch_model_for_coreml(model: nn.Module) -> nn.Module:
    """
    Patch a model to replace CoreML-incompatible operations.
    
    Args:
        model: The PyTorch model to patch
        
    Returns:
        Wrapped model with operation replacements
    """
    return CoreMLCompatibleWrapper(model)


def trace_with_patches(model: nn.Module, example_inputs: Any) -> torch.jit.ScriptModule:
    """
    Trace a model with operation patches applied.
    
    Args:
        model: The PyTorch model to trace
        example_inputs: Example inputs for tracing
        
    Returns:
        Traced TorchScript model
    """
    with BitwiseOperationReplacer():
        if isinstance(example_inputs, tuple):
            return torch.jit.trace(model, example_inputs)
        else:
            return torch.jit.trace(model, (example_inputs,))



