"""
Evaluation Layer for DSPy Optimization

Manages evaluation data collection, feedback tracking, and quality scoring.

@author @darianrosebrook
"""

from .data_collector import EvaluationDataCollector
from .feedback_tracker import FeedbackTracker
from .quality_scorer import QualityScorer

__all__ = ["EvaluationDataCollector", "FeedbackTracker", "QualityScorer"]
