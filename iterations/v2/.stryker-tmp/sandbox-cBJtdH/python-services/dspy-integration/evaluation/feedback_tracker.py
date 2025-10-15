"""
Feedback Tracker

Tracks and manages human feedback on evaluations.

@author @darianrosebrook
"""

from typing import Dict, List, Optional
from datetime import datetime
import structlog

logger = structlog.get_logger()


class FeedbackTracker:
    """
    Tracks human feedback on evaluations.

    Manages feedback collection and aggregation for optimization.
    """

    def __init__(self):
        """Initialize feedback tracker."""
        self.pending_feedback: Dict[str, Dict] = {}

        logger.info("feedback_tracker_initialized")

    def request_feedback(
        self,
        evaluation_id: str,
        evaluation_type: str,
        prompt_text: str
    ):
        """
        Request feedback for an evaluation.

        Args:
            evaluation_id: Evaluation ID
            evaluation_type: Type (rubric or judge)
            prompt_text: Text to show to reviewer
        """
        self.pending_feedback[evaluation_id] = {
            "type": evaluation_type,
            "prompt": prompt_text,
            "requested_at": datetime.utcnow().isoformat()
        }

        logger.info(
            "feedback_requested",
            evaluation_id=evaluation_id,
            type=evaluation_type
        )

    def has_pending_feedback(self, evaluation_id: str) -> bool:
        """Check if feedback is pending."""
        return evaluation_id in self.pending_feedback

    def get_pending_feedback_items(self, limit: int = 10) -> List[Dict]:
        """
        Get pending feedback items.

        Args:
            limit: Maximum number of items to return

        Returns:
            List of pending feedback dicts
        """
        items = []
        for eval_id, data in list(self.pending_feedback.items())[:limit]:
            items.append({
                "evaluation_id": eval_id,
                **data
            })

        return items

    def mark_feedback_received(self, evaluation_id: str):
        """Mark feedback as received."""
        if evaluation_id in self.pending_feedback:
            del self.pending_feedback[evaluation_id]

            logger.info("feedback_received", evaluation_id=evaluation_id)

    def get_feedback_stats(self) -> Dict:
        """Get feedback statistics."""
        return {
            "pending_count": len(self.pending_feedback),
            "pending_by_type": self._count_by_type()
        }

    def _count_by_type(self) -> Dict[str, int]:
        """Count pending feedback by type."""
        counts = {}
        for data in self.pending_feedback.values():
            eval_type = data["type"]
            counts[eval_type] = counts.get(eval_type, 0) + 1
        return counts
