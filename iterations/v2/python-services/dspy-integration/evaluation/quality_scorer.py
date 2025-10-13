"""
Quality Scorer

Automated quality scoring for evaluations.

@author @darianrosebrook
"""

from typing import Dict
import structlog

logger = structlog.get_logger()


class QualityScorer:
    """
    Automated quality scoring for evaluations.

    Provides heuristic-based quality scores for evaluations
    to supplement human feedback.
    """

    def __init__(self):
        """Initialize quality scorer."""
        logger.info("quality_scorer_initialized")

    def score_rubric_evaluation(
        self,
        task_context: str,
        agent_output: str,
        evaluation_criteria: str,
        reward_score: float,
        reasoning: str,
        improvement_suggestions: str
    ) -> Dict[str, float]:
        """
        Score quality of rubric evaluation.

        Args:
            task_context: Task description
            agent_output: Agent's output
            evaluation_criteria: Evaluation criteria
            reward_score: Computed reward score
            reasoning: Evaluation reasoning
            improvement_suggestions: Suggestions

        Returns:
            Dict of quality scores
        """
        scores = {}

        # Score 1: Reasoning completeness (based on length and structure)
        reasoning_words = len(reasoning.split())
        scores["reasoning_completeness"] = min(1.0, reasoning_words / 50)

        # Score 2: Suggestions actionability (presence of specific guidance)
        actionable_keywords = ["should", "could",
                               "use", "try", "consider", "add", "remove"]
        suggestions_words = improvement_suggestions.lower().split()
        actionable_count = sum(
            1 for word in suggestions_words if word in actionable_keywords)
        scores["suggestions_actionability"] = min(1.0, actionable_count / 3)

        # Score 3: Score consistency (reasonable score range)
        if 0.0 <= reward_score <= 1.0:
            scores["score_validity"] = 1.0
        else:
            scores["score_validity"] = 0.0

        # Score 4: Reasoning references criteria
        criteria_in_reasoning = any(
            word in reasoning.lower()
            for word in evaluation_criteria.lower().split()[:5]
        )
        scores["criteria_reference"] = 1.0 if criteria_in_reasoning else 0.5

        # Overall quality (weighted average)
        scores["overall"] = (
            scores["reasoning_completeness"] * 0.3 +
            scores["suggestions_actionability"] * 0.3 +
            scores["score_validity"] * 0.2 +
            scores["criteria_reference"] * 0.2
        )

        logger.debug(
            "rubric_quality_scored",
            overall=scores["overall"],
            scores=scores
        )

        return scores

    def score_judge_evaluation(
        self,
        judge_type: str,
        artifact: str,
        ground_truth: str,
        judgment: str,
        confidence: float,
        reasoning: str
    ) -> Dict[str, float]:
        """
        Score quality of judge evaluation.

        Args:
            judge_type: Type of judgment
            artifact: Output being judged
            ground_truth: Reference output
            judgment: Judge's judgment
            confidence: Confidence score
            reasoning: Judge's reasoning

        Returns:
            Dict of quality scores
        """
        scores = {}

        # Score 1: Judgment clarity (clear pass/fail/partial)
        clear_judgments = ["pass", "fail", "partial", "yes", "no"]
        judgment_clear = judgment.lower().strip() in clear_judgments
        scores["judgment_clarity"] = 1.0 if judgment_clear else 0.7

        # Score 2: Confidence calibration (reasonable confidence)
        if 0.0 <= confidence <= 1.0:
            # Penalize extreme confidence without sufficient reasoning
            reasoning_length = len(reasoning.split())
            if confidence > 0.9 and reasoning_length < 20:
                scores["confidence_calibration"] = 0.6
            elif confidence < 0.5 and reasoning_length < 30:
                scores["confidence_calibration"] = 0.7
            else:
                scores["confidence_calibration"] = 1.0
        else:
            scores["confidence_calibration"] = 0.0

        # Score 3: Reasoning depth (based on length and structure)
        reasoning_words = len(reasoning.split())
        scores["reasoning_depth"] = min(1.0, reasoning_words / 40)

        # Score 4: Artifact reference (reasoning mentions artifact)
        artifact_words = set(artifact.lower().split()[:10])
        reasoning_words_set = set(reasoning.lower().split())
        overlap = len(artifact_words & reasoning_words_set)
        scores["artifact_reference"] = min(1.0, overlap / 3)

        # Overall quality (weighted average)
        scores["overall"] = (
            scores["judgment_clarity"] * 0.3 +
            scores["confidence_calibration"] * 0.3 +
            scores["reasoning_depth"] * 0.2 +
            scores["artifact_reference"] * 0.2
        )

        logger.debug(
            "judge_quality_scored",
            judge_type=judge_type,
            overall=scores["overall"],
            scores=scores
        )

        return scores
