"""
Optimization Layer for DSPy

Implements MIPROv2 optimization pipeline for self-improving prompts.

@author @darianrosebrook
"""

from .training_data import RubricTrainingFactory, JudgeTrainingFactory
from .metrics import rubric_metric, judge_metric
from .pipeline import OptimizationPipeline

__all__ = [
    "RubricTrainingFactory",
    "JudgeTrainingFactory",
    "rubric_metric",
    "judge_metric",
    "OptimizationPipeline",
]
