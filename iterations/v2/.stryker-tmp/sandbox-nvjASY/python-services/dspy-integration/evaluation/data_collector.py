"""
Evaluation Data Collector

Collects evaluation data from DSPy modules for training purposes.

@author @darianrosebrook
"""

import uuid
from typing import Optional, Dict, Any
from datetime import datetime
import structlog

from storage.evaluation_store import EvaluationStore

logger = structlog.get_logger()


class EvaluationDataCollector:
    """
    Collects and stores evaluation data for training.

    Captures all rubric and judge evaluations with optional human feedback
    for use in optimization.
    """

    def __init__(self, store: Optional[EvaluationStore] = None):
        """
        Initialize data collector.

        Args:
            store: EvaluationStore instance (creates new if not provided)
        """
        self.store = store or EvaluationStore()

        logger.info("evaluation_data_collector_initialized")

    def collect_rubric_evaluation(
        self,
        task_context: str,
        agent_output: str,
        evaluation_criteria: str,
        reward_score: float,
        reasoning: str,
        improvement_suggestions: str,
        model_used: str,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        Collect rubric evaluation for training.

        Args:
            task_context: Task description
            agent_output: Agent's output
            evaluation_criteria: Evaluation criteria
            reward_score: Computed reward score
            reasoning: Evaluation reasoning
            improvement_suggestions: Suggestions for improvement
            model_used: Model that generated evaluation
            metadata: Optional metadata

        Returns:
            Evaluation ID
        """
        evaluation_id = f"rubric_{uuid.uuid4().hex[:12]}"

        self.store.store_rubric_evaluation(
            evaluation_id=evaluation_id,
            task_context=task_context,
            agent_output=agent_output,
            evaluation_criteria=evaluation_criteria,
            reward_score=reward_score,
            reasoning=reasoning,
            improvement_suggestions=improvement_suggestions,
            model_used=model_used,
            metadata=metadata
        )

        logger.info(
            "rubric_evaluation_collected",
            evaluation_id=evaluation_id,
            reward_score=reward_score
        )

        return evaluation_id

    def collect_judge_evaluation(
        self,
        judge_type: str,
        artifact: str,
        ground_truth: str,
        context: str,
        judgment: str,
        confidence: float,
        reasoning: str,
        model_used: str,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        Collect judge evaluation for training.

        Args:
            judge_type: Type of judgment
            artifact: Output being judged
            ground_truth: Reference output
            context: Task context
            judgment: Judge's judgment
            confidence: Confidence score
            reasoning: Judge's reasoning
            model_used: Model that generated judgment
            metadata: Optional metadata

        Returns:
            Evaluation ID
        """
        evaluation_id = f"judge_{uuid.uuid4().hex[:12]}"

        self.store.store_judge_evaluation(
            evaluation_id=evaluation_id,
            judge_type=judge_type,
            artifact=artifact,
            ground_truth=ground_truth,
            context=context,
            judgment=judgment,
            confidence=confidence,
            reasoning=reasoning,
            model_used=model_used,
            metadata=metadata
        )

        logger.info(
            "judge_evaluation_collected",
            evaluation_id=evaluation_id,
            judge_type=judge_type,
            judgment=judgment
        )

        return evaluation_id

    def add_rubric_feedback(
        self,
        evaluation_id: str,
        feedback_score: float,
        feedback_notes: Optional[str] = None
    ):
        """
        Add human feedback to rubric evaluation.

        Args:
            evaluation_id: Evaluation ID
            feedback_score: Human feedback score (0.0-1.0)
            feedback_notes: Optional feedback notes
        """
        # Update existing record with feedback
        import sqlite3

        with sqlite3.connect(self.store.db_path) as conn:
            conn.execute("""
                UPDATE rubric_evaluations 
                SET feedback_score = ?, feedback_notes = ?
                WHERE id = ?
            """, (feedback_score, feedback_notes, evaluation_id))
            conn.commit()

        logger.info(
            "rubric_feedback_added",
            evaluation_id=evaluation_id,
            feedback_score=feedback_score
        )

    def add_judge_feedback(
        self,
        evaluation_id: str,
        feedback_correct: bool,
        feedback_notes: Optional[str] = None
    ):
        """
        Add human feedback to judge evaluation.

        Args:
            evaluation_id: Evaluation ID
            feedback_correct: Whether judgment was correct
            feedback_notes: Optional feedback notes
        """
        # Update existing record with feedback
        import sqlite3

        with sqlite3.connect(self.store.db_path) as conn:
            conn.execute("""
                UPDATE judge_evaluations 
                SET feedback_correct = ?, feedback_notes = ?
                WHERE id = ?
            """, (feedback_correct, feedback_notes, evaluation_id))
            conn.commit()

        logger.info(
            "judge_feedback_added",
            evaluation_id=evaluation_id,
            feedback_correct=feedback_correct
        )

    def get_training_data_stats(self) -> Dict[str, Any]:
        """
        Get statistics about collected training data.

        Returns:
            Statistics dict
        """
        counts = self.store.count_evaluations()

        # Get judge type breakdown
        import sqlite3
        with sqlite3.connect(self.store.db_path) as conn:
            judge_types = conn.execute("""
                SELECT judge_type, COUNT(*) as count
                FROM judge_evaluations
                GROUP BY judge_type
            """).fetchall()

        judge_breakdown = {
            judge_type: count for judge_type, count in judge_types}

        return {
            "total_evaluations": counts["total"],
            "rubric_evaluations": counts["rubric_total"],
            "judge_evaluations": counts["judge_total"],
            "rubric_with_feedback": counts["rubric_with_feedback"],
            "judge_with_feedback": counts["judge_with_feedback"],
            "judge_type_breakdown": judge_breakdown,
            "ready_for_optimization": {
                "rubric": counts["rubric_total"] >= 50,
                "judge_relevance": judge_breakdown.get("relevance", 0) >= 50,
                "judge_faithfulness": judge_breakdown.get("faithfulness", 0) >= 50,
                "judge_minimality": judge_breakdown.get("minimality", 0) >= 50,
                "judge_safety": judge_breakdown.get("safety", 0) >= 50,
            }
        }
