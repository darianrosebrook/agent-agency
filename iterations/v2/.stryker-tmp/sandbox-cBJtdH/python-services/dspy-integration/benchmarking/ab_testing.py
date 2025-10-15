"""
A/B Testing Framework

Framework for comparing baseline vs optimized models.

@author @darianrosebrook
"""

import sqlite3
import random
import uuid
from typing import Dict, Any, Optional, List
from dataclasses import dataclass
from datetime import datetime
import structlog

logger = structlog.get_logger()


@dataclass
class ABTestResults:
    """Results from an A/B test."""
    experiment_id: str
    baseline_mean: float
    optimized_mean: float
    baseline_count: int
    optimized_count: int
    improvement_percent: float
    is_significant: bool
    p_value: float
    confidence_interval: tuple[float, float]


class ABTestingFramework:
    """
    A/B testing framework for model optimization.

    Manages experiments comparing baseline vs optimized models.
    """

    def __init__(self, db_path: str = "./ab_tests.db"):
        """
        Initialize A/B testing framework.

        Args:
            db_path: Path to SQLite database
        """
        self.db_path = db_path
        # For in-memory databases, keep a persistent connection
        self._is_memory = (db_path == ":memory:")
        if self._is_memory:
            self._conn = sqlite3.connect(db_path, check_same_thread=False)
        else:
            self._conn = None

        self._init_database()

        logger.info("ab_testing_framework_initialized", db_path=db_path)

    def _get_conn(self):
        """Get database connection."""
        if self._is_memory:
            return self._conn
        return sqlite3.connect(self.db_path)

    def _close_conn(self, conn):
        """Close connection if not persistent."""
        if not self._is_memory:
            conn.close()

    def _init_database(self):
        """Initialize database schema."""
        conn = self._get_conn()

        # Experiments table
        conn.execute("""
            CREATE TABLE IF NOT EXISTS experiments (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                module_type TEXT NOT NULL,
                baseline_model_id TEXT,
                optimized_model_id TEXT,
                split_ratio REAL DEFAULT 0.5,
                created_at TEXT NOT NULL,
                status TEXT DEFAULT 'active',
                notes TEXT
            )
        """)

        # Evaluations table
        conn.execute("""
            CREATE TABLE IF NOT EXISTS ab_evaluations (
                id TEXT PRIMARY KEY,
                experiment_id TEXT NOT NULL,
                variant TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                metrics TEXT NOT NULL,
                FOREIGN KEY (experiment_id) REFERENCES experiments(id)
            )
        """)

        # Create indexes
        conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_experiment_id 
            ON ab_evaluations(experiment_id)
        """)

        conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_variant 
            ON ab_evaluations(variant)
        """)

        conn.commit()
        self._close_conn(conn)

    def create_experiment(
        self,
        name: str,
        module_type: str,
        baseline_model_id: Optional[str] = None,
        optimized_model_id: Optional[str] = None,
        split_ratio: float = 0.5,
        notes: Optional[str] = None
    ) -> str:
        """
        Create A/B test experiment.

        Args:
            name: Experiment name
            module_type: Type of module being tested
            baseline_model_id: Baseline model ID (None for default)
            optimized_model_id: Optimized model ID
            split_ratio: Traffic split ratio (0.5 = 50/50)
            notes: Optional notes

        Returns:
            Experiment ID
        """
        experiment_id = f"exp_{uuid.uuid4().hex[:12]}"
        created_at = datetime.utcnow().isoformat()

        conn = self._get_conn()
        conn.execute("""
            INSERT INTO experiments (
                id, name, module_type, baseline_model_id, optimized_model_id,
                split_ratio, created_at, notes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            experiment_id, name, module_type, baseline_model_id, optimized_model_id,
            split_ratio, created_at, notes
        ))
        conn.commit()
        self._close_conn(conn)

        logger.info(
            "ab_experiment_created",
            experiment_id=experiment_id,
            name=name,
            module_type=module_type
        )

        return experiment_id

    def assign_variant(self, experiment_id: str) -> str:
        """
        Assign variant for an evaluation.

        Args:
            experiment_id: Experiment ID

        Returns:
            Variant name ('baseline' or 'optimized')
        """
        # Get split ratio
        conn = self._get_conn()
        row = conn.execute(
            "SELECT split_ratio FROM experiments WHERE id = ?",
            (experiment_id,)
        ).fetchone()
        self._close_conn(conn)

        if not row:
            raise ValueError(f"Experiment not found: {experiment_id}")

        split_ratio = row[0]

        # Random assignment
        variant = "baseline" if random.random() < split_ratio else "optimized"

        logger.debug(
            "variant_assigned",
            experiment_id=experiment_id,
            variant=variant
        )

        return variant

    def record_evaluation(
        self,
        experiment_id: str,
        variant: str,
        metrics: Dict[str, float]
    ):
        """
        Record evaluation metrics for A/B test.

        Args:
            experiment_id: Experiment ID
            variant: Variant name ('baseline' or 'optimized')
            metrics: Metrics dict
        """
        import json

        eval_id = f"eval_{uuid.uuid4().hex[:12]}"
        timestamp = datetime.utcnow().isoformat()
        metrics_json = json.dumps(metrics)

        conn = self._get_conn()
        conn.execute("""
            INSERT INTO ab_evaluations (
                id, experiment_id, variant, timestamp, metrics
            ) VALUES (?, ?, ?, ?, ?)
        """, (eval_id, experiment_id, variant, timestamp, metrics_json))
        conn.commit()
        self._close_conn(conn)

        logger.debug(
            "ab_evaluation_recorded",
            experiment_id=experiment_id,
            variant=variant,
            primary_metric=metrics.get("primary_score", 0.0)
        )

    def analyze_results(
        self,
        experiment_id: str,
        metric_key: str = "primary_score",
        confidence_level: float = 0.95
    ) -> ABTestResults:
        """
        Analyze A/B test results with statistical significance.

        Args:
            experiment_id: Experiment ID
            metric_key: Metric to analyze
            confidence_level: Confidence level for significance testing

        Returns:
            ABTestResults with analysis
        """
        import json

        # Get evaluations
        conn = self._get_conn()
        rows = conn.execute("""
            SELECT variant, metrics
            FROM ab_evaluations
            WHERE experiment_id = ?
        """, (experiment_id,)).fetchall()
        self._close_conn(conn)

        if not rows:
            raise ValueError(
                f"No evaluations found for experiment: {experiment_id}")

        # Parse metrics
        baseline_scores = []
        optimized_scores = []

        for variant, metrics_json in rows:
            metrics = json.loads(metrics_json)
            score = metrics.get(metric_key, 0.0)

            if variant == "baseline":
                baseline_scores.append(score)
            elif variant == "optimized":
                optimized_scores.append(score)

        # Calculate statistics
        baseline_mean = sum(baseline_scores) / \
            len(baseline_scores) if baseline_scores else 0.0
        optimized_mean = sum(optimized_scores) / \
            len(optimized_scores) if optimized_scores else 0.0

        improvement_percent = (
            ((optimized_mean - baseline_mean) / baseline_mean * 100)
            if baseline_mean > 0 else 0.0
        )

        # Statistical significance test (simplified t-test)
        is_significant, p_value = self._calculate_significance(
            baseline_scores,
            optimized_scores,
            confidence_level
        )

        # Confidence interval
        ci_lower, ci_upper = self._calculate_confidence_interval(
            baseline_scores,
            optimized_scores,
            confidence_level
        )

        results = ABTestResults(
            experiment_id=experiment_id,
            baseline_mean=baseline_mean,
            optimized_mean=optimized_mean,
            baseline_count=len(baseline_scores),
            optimized_count=len(optimized_scores),
            improvement_percent=improvement_percent,
            is_significant=is_significant,
            p_value=p_value,
            confidence_interval=(ci_lower, ci_upper)
        )

        logger.info(
            "ab_test_analyzed",
            experiment_id=experiment_id,
            improvement_percent=improvement_percent,
            is_significant=is_significant,
            p_value=p_value
        )

        return results

    def _calculate_significance(
        self,
        baseline_scores: List[float],
        optimized_scores: List[float],
        confidence_level: float
    ) -> tuple[bool, float]:
        """Calculate statistical significance (simplified t-test)."""
        if len(baseline_scores) < 3 or len(optimized_scores) < 3:
            return False, 1.0

        mean1 = sum(baseline_scores) / len(baseline_scores)
        mean2 = sum(optimized_scores) / len(optimized_scores)

        var1 = sum((x - mean1) ** 2 for x in baseline_scores) / \
            (len(baseline_scores) - 1)
        var2 = sum((x - mean2) ** 2 for x in optimized_scores) / \
            (len(optimized_scores) - 1)

        pooled_se = ((var1 / len(baseline_scores)) +
                     (var2 / len(optimized_scores))) ** 0.5

        if pooled_se == 0:
            return False, 1.0

        t_stat = abs((mean2 - mean1) / pooled_se)

        if t_stat > 2.0:
            p_value = 0.05
        elif t_stat > 1.96:
            p_value = 0.05
        else:
            p_value = 0.1

        is_significant = p_value < (1.0 - confidence_level)

        return is_significant, p_value

    def _calculate_confidence_interval(
        self,
        baseline_scores: List[float],
        optimized_scores: List[float],
        confidence_level: float
    ) -> tuple[float, float]:
        """Calculate confidence interval for difference."""
        if not baseline_scores or not optimized_scores:
            return (0.0, 0.0)

        mean1 = sum(baseline_scores) / len(baseline_scores)
        mean2 = sum(optimized_scores) / len(optimized_scores)
        diff = mean2 - mean1

        var1 = sum((x - mean1) ** 2 for x in baseline_scores) / \
            max(1, len(baseline_scores) - 1)
        var2 = sum((x - mean2) ** 2 for x in optimized_scores) / \
            max(1, len(optimized_scores) - 1)
        se = ((var1 / len(baseline_scores)) +
              (var2 / len(optimized_scores))) ** 0.5

        margin = 1.96 * se

        return (diff - margin, diff + margin)

    def stop_experiment(self, experiment_id: str):
        """Stop an active experiment."""
        conn = self._get_conn()
        conn.execute("""
            UPDATE experiments
            SET status = 'stopped'
            WHERE id = ?
        """, (experiment_id,))
        conn.commit()
        self._close_conn(conn)

        logger.info("experiment_stopped", experiment_id=experiment_id)

    def list_experiments(self, status: Optional[str] = None) -> List[Dict[str, Any]]:
        """List experiments."""
        query = "SELECT * FROM experiments"

        if status:
            query += f" WHERE status = '{status}'"

        query += " ORDER BY created_at DESC"

        conn = self._get_conn()
        conn.row_factory = sqlite3.Row
        rows = conn.execute(query).fetchall()
        self._close_conn(conn)

        experiments = [dict(row) for row in rows]

        logger.info(
            "experiments_listed",
            count=len(experiments),
            status=status
        )

        return experiments
