"""
Evaluation Storage

Persistent storage for evaluation data used in training.

@author @darianrosebrook
"""

import sqlite3
import json
from pathlib import Path
from typing import List, Dict, Any, Optional
from datetime import datetime
import structlog

logger = structlog.get_logger()


class EvaluationStore:
    """
    Persistent storage for evaluation data.

    Stores rubric evaluations and judge evaluations for training purposes.
    Uses SQLite for simplicity and local-first approach.
    """

    def __init__(self, db_path: str = "./dspy_evaluations.db"):
        """
        Initialize evaluation store.

        Args:
            db_path: Path to SQLite database file
        """
        self.db_path = db_path
        self._init_database()

        logger.info("evaluation_store_initialized", db_path=db_path)

    def _init_database(self):
        """Initialize database schema."""
        with sqlite3.connect(self.db_path) as conn:
            # Rubric evaluations table
            conn.execute("""
                CREATE TABLE IF NOT EXISTS rubric_evaluations (
                    id TEXT PRIMARY KEY,
                    timestamp TEXT NOT NULL,
                    task_context TEXT NOT NULL,
                    agent_output TEXT NOT NULL,
                    evaluation_criteria TEXT NOT NULL,
                    reward_score REAL NOT NULL,
                    reasoning TEXT NOT NULL,
                    improvement_suggestions TEXT NOT NULL,
                    model_used TEXT NOT NULL,
                    feedback_score REAL,
                    feedback_notes TEXT,
                    metadata TEXT
                )
            """)

            # Judge evaluations table
            conn.execute("""
                CREATE TABLE IF NOT EXISTS judge_evaluations (
                    id TEXT PRIMARY KEY,
                    timestamp TEXT NOT NULL,
                    judge_type TEXT NOT NULL,
                    artifact TEXT NOT NULL,
                    ground_truth TEXT NOT NULL,
                    context TEXT NOT NULL,
                    judgment TEXT NOT NULL,
                    confidence REAL NOT NULL,
                    reasoning TEXT NOT NULL,
                    model_used TEXT NOT NULL,
                    feedback_correct BOOLEAN,
                    feedback_notes TEXT,
                    metadata TEXT
                )
            """)

            # Create indexes
            conn.execute("""
                CREATE INDEX IF NOT EXISTS idx_rubric_timestamp 
                ON rubric_evaluations(timestamp)
            """)

            conn.execute("""
                CREATE INDEX IF NOT EXISTS idx_judge_timestamp 
                ON judge_evaluations(timestamp)
            """)

            conn.execute("""
                CREATE INDEX IF NOT EXISTS idx_judge_type 
                ON judge_evaluations(judge_type)
            """)

            conn.commit()

    def store_rubric_evaluation(
        self,
        evaluation_id: str,
        task_context: str,
        agent_output: str,
        evaluation_criteria: str,
        reward_score: float,
        reasoning: str,
        improvement_suggestions: str,
        model_used: str,
        feedback_score: Optional[float] = None,
        feedback_notes: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        Store rubric evaluation.

        Args:
            evaluation_id: Unique evaluation ID
            task_context: Task description
            agent_output: Agent's output
            evaluation_criteria: Evaluation criteria
            reward_score: Computed reward score
            reasoning: Evaluation reasoning
            improvement_suggestions: Suggestions for improvement
            model_used: Model that generated evaluation
            feedback_score: Optional human feedback score
            feedback_notes: Optional human feedback notes
            metadata: Optional metadata dict

        Returns:
            Evaluation ID
        """
        timestamp = datetime.utcnow().isoformat()
        metadata_json = json.dumps(metadata) if metadata else None

        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                INSERT INTO rubric_evaluations (
                    id, timestamp, task_context, agent_output, evaluation_criteria,
                    reward_score, reasoning, improvement_suggestions, model_used,
                    feedback_score, feedback_notes, metadata
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                evaluation_id, timestamp, task_context, agent_output, evaluation_criteria,
                reward_score, reasoning, improvement_suggestions, model_used,
                feedback_score, feedback_notes, metadata_json
            ))
            conn.commit()

        logger.info(
            "rubric_evaluation_stored",
            evaluation_id=evaluation_id,
            reward_score=reward_score,
            has_feedback=feedback_score is not None
        )

        return evaluation_id

    def store_judge_evaluation(
        self,
        evaluation_id: str,
        judge_type: str,
        artifact: str,
        ground_truth: str,
        context: str,
        judgment: str,
        confidence: float,
        reasoning: str,
        model_used: str,
        feedback_correct: Optional[bool] = None,
        feedback_notes: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        Store judge evaluation.

        Args:
            evaluation_id: Unique evaluation ID
            judge_type: Type of judgment
            artifact: Output being judged
            ground_truth: Reference output
            context: Task context
            judgment: Judge's judgment
            confidence: Confidence score
            reasoning: Judge's reasoning
            model_used: Model that generated judgment
            feedback_correct: Optional feedback on correctness
            feedback_notes: Optional feedback notes
            metadata: Optional metadata dict

        Returns:
            Evaluation ID
        """
        timestamp = datetime.utcnow().isoformat()
        metadata_json = json.dumps(metadata) if metadata else None

        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                INSERT INTO judge_evaluations (
                    id, timestamp, judge_type, artifact, ground_truth, context,
                    judgment, confidence, reasoning, model_used,
                    feedback_correct, feedback_notes, metadata
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                evaluation_id, timestamp, judge_type, artifact, ground_truth, context,
                judgment, confidence, reasoning, model_used,
                feedback_correct, feedback_notes, metadata_json
            ))
            conn.commit()

        logger.info(
            "judge_evaluation_stored",
            evaluation_id=evaluation_id,
            judge_type=judge_type,
            judgment=judgment,
            has_feedback=feedback_correct is not None
        )

        return evaluation_id

    def get_rubric_evaluations(
        self,
        limit: Optional[int] = None,
        with_feedback_only: bool = False
    ) -> List[Dict[str, Any]]:
        """
        Get rubric evaluations.

        Args:
            limit: Maximum number of evaluations to return
            with_feedback_only: Only return evaluations with human feedback

        Returns:
            List of evaluation dicts
        """
        query = "SELECT * FROM rubric_evaluations"

        if with_feedback_only:
            query += " WHERE feedback_score IS NOT NULL"

        query += " ORDER BY timestamp DESC"

        if limit:
            query += f" LIMIT {limit}"

        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(query)
            rows = cursor.fetchall()

        evaluations = []
        for row in rows:
            eval_dict = dict(row)
            if eval_dict["metadata"]:
                eval_dict["metadata"] = json.loads(eval_dict["metadata"])
            evaluations.append(eval_dict)

        logger.info(
            "rubric_evaluations_retrieved",
            count=len(evaluations),
            with_feedback_only=with_feedback_only
        )

        return evaluations

    def get_judge_evaluations(
        self,
        judge_type: Optional[str] = None,
        limit: Optional[int] = None,
        with_feedback_only: bool = False
    ) -> List[Dict[str, Any]]:
        """
        Get judge evaluations.

        Args:
            judge_type: Filter by judge type
            limit: Maximum number of evaluations to return
            with_feedback_only: Only return evaluations with human feedback

        Returns:
            List of evaluation dicts
        """
        query = "SELECT * FROM judge_evaluations"
        conditions = []

        if judge_type:
            conditions.append(f"judge_type = '{judge_type}'")

        if with_feedback_only:
            conditions.append("feedback_correct IS NOT NULL")

        if conditions:
            query += " WHERE " + " AND ".join(conditions)

        query += " ORDER BY timestamp DESC"

        if limit:
            query += f" LIMIT {limit}"

        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(query)
            rows = cursor.fetchall()

        evaluations = []
        for row in rows:
            eval_dict = dict(row)
            if eval_dict["metadata"]:
                eval_dict["metadata"] = json.loads(eval_dict["metadata"])
            evaluations.append(eval_dict)

        logger.info(
            "judge_evaluations_retrieved",
            count=len(evaluations),
            judge_type=judge_type,
            with_feedback_only=with_feedback_only
        )

        return evaluations

    def count_evaluations(self) -> Dict[str, int]:
        """
        Get count of stored evaluations.

        Returns:
            Dict with counts by type
        """
        with sqlite3.connect(self.db_path) as conn:
            rubric_count = conn.execute(
                "SELECT COUNT(*) FROM rubric_evaluations"
            ).fetchone()[0]

            judge_count = conn.execute(
                "SELECT COUNT(*) FROM judge_evaluations"
            ).fetchone()[0]

            rubric_with_feedback = conn.execute(
                "SELECT COUNT(*) FROM rubric_evaluations WHERE feedback_score IS NOT NULL"
            ).fetchone()[0]

            judge_with_feedback = conn.execute(
                "SELECT COUNT(*) FROM judge_evaluations WHERE feedback_correct IS NOT NULL"
            ).fetchone()[0]

        return {
            "rubric_total": rubric_count,
            "judge_total": judge_count,
            "rubric_with_feedback": rubric_with_feedback,
            "judge_with_feedback": judge_with_feedback,
            "total": rubric_count + judge_count
        }
