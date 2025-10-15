"""
Optimization Metrics

Evaluation metrics for DSPy optimization.

@author @darianrosebrook
"""

import dspy
from typing import Optional, Any
import structlog

logger = structlog.get_logger()


def rubric_metric(example: dspy.Example, pred: dspy.Prediction, trace: Optional[Any] = None) -> float:
    """
    Evaluate rubric prediction quality.

    This metric measures how well the optimized rubric module performs
    compared to ground truth evaluations.

    Args:
        example: Ground truth example
        pred: Model prediction
        trace: Optional execution trace

    Returns:
        Score from 0.0 to 1.0 (higher is better)
    """
    score = 0.0

    # Component 1: Score Accuracy (50% weight)
    # Measure how close the predicted score is to the ground truth
    try:
        score_diff = abs(pred.reward_score - example.reward_score)
        score_accuracy = max(0.0, 1.0 - score_diff)
        score += score_accuracy * 0.5

        logger.debug(
            "rubric_metric_score_accuracy",
            predicted=pred.reward_score,
            expected=example.reward_score,
            accuracy=score_accuracy
        )
    except (AttributeError, TypeError) as error:
        logger.warning("rubric_metric_score_missing", error=str(error))
        score_accuracy = 0.0

    # Component 2: Reasoning Quality (30% weight)
    # Measure reasoning completeness and relevance
    try:
        reasoning = pred.reasoning
        expected_reasoning = example.reasoning

        # Check length (min 20 words for quality reasoning)
        reasoning_words = len(reasoning.split())
        length_score = min(1.0, reasoning_words / 30)

        # Check if reasoning mentions the criteria
        criteria = example.evaluation_criteria.lower()
        reasoning_lower = reasoning.lower()
        criteria_words = set(criteria.split()[:5])
        reasoning_words_set = set(reasoning_lower.split())

        criteria_overlap = len(criteria_words & reasoning_words_set)
        criteria_score = min(1.0, criteria_overlap / 3)

        reasoning_quality = (length_score + criteria_score) / 2
        score += reasoning_quality * 0.3

        logger.debug(
            "rubric_metric_reasoning_quality",
            length_score=length_score,
            criteria_score=criteria_score,
            quality=reasoning_quality
        )
    except (AttributeError, TypeError) as error:
        logger.warning("rubric_metric_reasoning_missing", error=str(error))
        reasoning_quality = 0.0

    # Component 3: Suggestion Quality (20% weight)
    # Measure actionability and specificity of suggestions
    try:
        suggestions = pred.improvement_suggestions

        # Check for actionable language
        actionable_keywords = ["should", "could", "use",
                               "try", "consider", "add", "remove", "include"]
        suggestions_lower = suggestions.lower()
        actionable_count = sum(
            1 for keyword in actionable_keywords if keyword in suggestions_lower)
        actionability = min(1.0, actionable_count / 3)

        # Check length (min 15 words for quality suggestions)
        suggestions_words = len(suggestions.split())
        suggestion_length = min(1.0, suggestions_words / 20)

        suggestion_quality = (actionability + suggestion_length) / 2
        score += suggestion_quality * 0.2

        logger.debug(
            "rubric_metric_suggestion_quality",
            actionability=actionability,
            length=suggestion_length,
            quality=suggestion_quality
        )
    except (AttributeError, TypeError) as error:
        logger.warning("rubric_metric_suggestions_missing", error=str(error))
        suggestion_quality = 0.0

    logger.info(
        "rubric_metric_evaluated",
        total_score=score,
        score_accuracy=score_accuracy if 'score_accuracy' in locals() else 0.0,
        reasoning_quality=reasoning_quality if 'reasoning_quality' in locals() else 0.0,
        suggestion_quality=suggestion_quality if 'suggestion_quality' in locals() else 0.0
    )

    return score


def judge_metric(example: dspy.Example, pred: dspy.Prediction, trace: Optional[Any] = None) -> float:
    """
    Evaluate judge prediction quality.

    This metric measures how well the optimized judge module performs
    compared to ground truth judgments.

    Args:
        example: Ground truth example
        pred: Model prediction
        trace: Optional execution trace

    Returns:
        Score from 0.0 to 1.0 (higher is better)
    """
    score = 0.0

    # Component 1: Judgment Correctness (60% weight)
    # Binary: Does the judgment match the ground truth?
    try:
        judgment_correct = pred.judgment.lower().strip() == example.judgment.lower().strip()
        judgment_score = 1.0 if judgment_correct else 0.0
        score += judgment_score * 0.6

        logger.debug(
            "judge_metric_judgment",
            predicted=pred.judgment,
            expected=example.judgment,
            correct=judgment_correct
        )
    except (AttributeError, TypeError) as error:
        logger.warning("judge_metric_judgment_missing", error=str(error))
        judgment_score = 0.0

    # Component 2: Confidence Calibration (20% weight)
    # Is confidence appropriate for correctness?
    try:
        confidence = pred.confidence
        expected_confidence = example.confidence

        # Penalize large confidence errors
        confidence_diff = abs(confidence - expected_confidence)
        calibration_score = max(0.0, 1.0 - confidence_diff)

        # Extra penalty for overconfidence on incorrect judgments
        if 'judgment_correct' in locals() and not judgment_correct and confidence > 0.8:
            calibration_score *= 0.5

        score += calibration_score * 0.2

        logger.debug(
            "judge_metric_confidence",
            predicted=confidence,
            expected=expected_confidence,
            calibration=calibration_score
        )
    except (AttributeError, TypeError) as error:
        logger.warning("judge_metric_confidence_missing", error=str(error))
        calibration_score = 0.0

    # Component 3: Reasoning Clarity (20% weight)
    # Quality and relevance of reasoning
    try:
        reasoning = pred.reasoning

        # Check length (min 20 words for quality reasoning)
        reasoning_words = len(reasoning.split())
        length_score = min(1.0, reasoning_words / 25)

        # Check if reasoning references the artifact
        artifact = example.artifact.lower()
        reasoning_lower = reasoning.lower()
        artifact_words = set(artifact.split()[:8])
        reasoning_words_set = set(reasoning_lower.split())

        artifact_overlap = len(artifact_words & reasoning_words_set)
        reference_score = min(1.0, artifact_overlap / 3)

        clarity_score = (length_score + reference_score) / 2
        score += clarity_score * 0.2

        logger.debug(
            "judge_metric_reasoning",
            length_score=length_score,
            reference_score=reference_score,
            clarity=clarity_score
        )
    except (AttributeError, TypeError) as error:
        logger.warning("judge_metric_reasoning_missing", error=str(error))
        clarity_score = 0.0

    logger.info(
        "judge_metric_evaluated",
        total_score=score,
        judgment_correct=judgment_score if 'judgment_score' in locals() else 0.0,
        calibration=calibration_score if 'calibration_score' in locals() else 0.0,
        clarity=clarity_score if 'clarity_score' in locals() else 0.0
    )

    return score


def aggregate_metrics(scores: list[float]) -> dict[str, float]:
    """
    Aggregate multiple metric scores.

    Args:
        scores: List of individual scores

    Returns:
        Dict with aggregate statistics
    """
    if not scores:
        return {
            "mean": 0.0,
            "min": 0.0,
            "max": 0.0,
            "count": 0
        }

    return {
        "mean": sum(scores) / len(scores),
        "min": min(scores),
        "max": max(scores),
        "count": len(scores)
    }
