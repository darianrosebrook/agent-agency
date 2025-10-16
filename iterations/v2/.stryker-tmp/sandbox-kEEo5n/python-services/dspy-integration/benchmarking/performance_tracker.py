"""
Performance Tracker

Tracks optimization performance over time.

@author @darianrosebrook
"""

import sqlite3
from typing import Dict, Any, List, Optional
from datetime import datetime
import structlog

logger = structlog.get_logger()


class PerformanceTracker:
    """
    Tracks model performance over time.

    Monitors metrics to detect degradation or improvement.
    """

    def __init__(self, db_path: str = "./performance_tracking.db"):
        """
        Initialize performance tracker.

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

        logger.info("performance_tracker_initialized", db_path=db_path)

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
        conn.execute("""
            CREATE TABLE IF NOT EXISTS performance_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                module_type TEXT NOT NULL,
                model_id TEXT,
                metrics TEXT NOT NULL,
                sample_size INTEGER,
                notes TEXT
            )
        """)

        # Create indexes
        conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_module_type 
            ON performance_snapshots(module_type)
        """)

        conn.execute("""
            CREATE INDEX IF NOT EXISTS idx_timestamp 
            ON performance_snapshots(timestamp)
        """)

        conn.commit()
        self._close_conn(conn)

    def record_snapshot(
        self,
        module_type: str,
        metrics: Dict[str, float],
        model_id: Optional[str] = None,
        sample_size: Optional[int] = None,
        notes: Optional[str] = None
    ):
        """
        Record performance snapshot.

        Args:
            module_type: Type of module
            metrics: Performance metrics dict
            model_id: Model ID (if applicable)
            sample_size: Number of evaluations in snapshot
            notes: Optional notes
        """
        import json

        timestamp = datetime.utcnow().isoformat()
        metrics_json = json.dumps(metrics)

        conn = self._get_conn()
        conn.execute("""
            INSERT INTO performance_snapshots (
                timestamp, module_type, model_id, metrics, sample_size, notes
            ) VALUES (?, ?, ?, ?, ?, ?)
        """, (timestamp, module_type, model_id, metrics_json, sample_size, notes))
        conn.commit()
        self._close_conn(conn)

        logger.info(
            "performance_snapshot_recorded",
            module_type=module_type,
            model_id=model_id,
            primary_metric=metrics.get("mean_score", 0.0)
        )

    def get_history(
        self,
        module_type: str,
        limit: Optional[int] = None
    ) -> List[Dict[str, Any]]:
        """
        Get performance history for module type.

        Args:
            module_type: Type of module
            limit: Maximum number of snapshots to return

        Returns:
            List of snapshot dicts
        """
        import json

        query = """
            SELECT * FROM performance_snapshots
            WHERE module_type = ?
            ORDER BY timestamp DESC
        """

        if limit:
            query += f" LIMIT {limit}"

        conn = self._get_conn()
        conn.row_factory = sqlite3.Row
        rows = conn.execute(query, (module_type,)).fetchall()
        self._close_conn(conn)

        snapshots = []
        for row in rows:
            snapshot = dict(row)
            snapshot["metrics"] = json.loads(snapshot["metrics"])
            snapshots.append(snapshot)

        logger.info(
            "performance_history_retrieved",
            module_type=module_type,
            count=len(snapshots)
        )

        return snapshots

    def detect_degradation(
        self,
        module_type: str,
        metric_key: str = "mean_score",
        threshold: float = 0.1
    ) -> Dict[str, Any]:
        """
        Detect performance degradation.

        Args:
            module_type: Type of module
            metric_key: Metric to check
            threshold: Degradation threshold (e.g., 0.1 = 10% drop)

        Returns:
            Dict with degradation analysis
        """
        history = self.get_history(module_type, limit=10)

        if len(history) < 2:
            return {
                "degradation_detected": False,
                "reason": "insufficient_data",
                "current_value": None,
                "baseline_value": None
            }

        # Compare latest to average of previous
        latest = history[0]["metrics"].get(metric_key, 0.0)
        previous = [
            h["metrics"].get(metric_key, 0.0)
            for h in history[1:]
        ]
        baseline = sum(previous) / len(previous) if previous else 0.0

        if baseline == 0:
            return {
                "degradation_detected": False,
                "reason": "zero_baseline",
                "current_value": latest,
                "baseline_value": baseline
            }

        drop_percent = (baseline - latest) / baseline

        degradation_detected = drop_percent > threshold

        if degradation_detected:
            logger.warning(
                "performance_degradation_detected",
                module_type=module_type,
                current=latest,
                baseline=baseline,
                drop_percent=drop_percent
            )

        return {
            "degradation_detected": degradation_detected,
            "reason": "threshold_exceeded" if degradation_detected else "within_threshold",
            "current_value": latest,
            "baseline_value": baseline,
            "drop_percent": drop_percent,
            "threshold": threshold
        }

    def get_summary(self, module_type: str) -> Dict[str, Any]:
        """
        Get performance summary for module type.

        Args:
            module_type: Type of module

        Returns:
            Summary dict
        """
        history = self.get_history(module_type, limit=100)

        if not history:
            return {
                "module_type": module_type,
                "snapshots_count": 0,
                "latest": None,
                "trend": None
            }

        latest = history[0]
        oldest = history[-1]

        # Calculate trend
        latest_score = latest["metrics"].get("mean_score", 0.0)
        oldest_score = oldest["metrics"].get("mean_score", 0.0)

        trend = "improving" if latest_score > oldest_score else "declining"
        trend_percent = ((latest_score - oldest_score) /
                         oldest_score * 100) if oldest_score > 0 else 0.0

        return {
            "module_type": module_type,
            "snapshots_count": len(history),
            "latest": latest,
            "trend": trend,
            "trend_percent": trend_percent,
            "first_snapshot": oldest["timestamp"],
            "last_snapshot": latest["timestamp"]
        }
