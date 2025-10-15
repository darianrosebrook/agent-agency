"""
Model Registry

Manages versioning and storage of optimized DSPy models.

@author @darianrosebrook
"""

import sqlite3
import pickle
import json
from pathlib import Path
from typing import Optional, List, Dict, Any
from datetime import datetime
import structlog

logger = structlog.get_logger()


class ModelRegistry:
    """
    Registry for optimized DSPy models.

    Handles versioning, storage, and retrieval of optimized modules.
    """

    def __init__(self, db_path: str = "./dspy_models.db", models_dir: str = "./dspy_models"):
        """
        Initialize model registry.

        Args:
            db_path: Path to SQLite database
            models_dir: Directory to store serialized models
        """
        self.db_path = db_path
        self.models_dir = Path(models_dir)
        self.models_dir.mkdir(exist_ok=True)

        self._init_database()

        logger.info("model_registry_initialized",
                    db_path=db_path, models_dir=models_dir)

    def _init_database(self):
        """Initialize database schema."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS models (
                    id TEXT PRIMARY KEY,
                    module_type TEXT NOT NULL,
                    version INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    file_path TEXT NOT NULL,
                    metrics TEXT,
                    training_examples_count INTEGER,
                    optimization_params TEXT,
                    is_active BOOLEAN DEFAULT FALSE,
                    notes TEXT,
                    UNIQUE(module_type, version)
                )
            """)

            # Create indexes
            conn.execute("""
                CREATE INDEX IF NOT EXISTS idx_module_type 
                ON models(module_type)
            """)

            conn.execute("""
                CREATE INDEX IF NOT EXISTS idx_is_active 
                ON models(is_active)
            """)

            conn.commit()

    def register_model(
        self,
        model_id: str,
        module_type: str,
        module: Any,
        version: Optional[int] = None,
        metrics: Optional[Dict[str, float]] = None,
        training_examples_count: Optional[int] = None,
        optimization_params: Optional[Dict[str, Any]] = None,
        notes: Optional[str] = None
    ) -> str:
        """
        Register an optimized model.

        Args:
            model_id: Unique model identifier
            module_type: Type of module (rubric_optimizer, judge_relevance, etc.)
            module: The DSPy module to store
            version: Version number (auto-increments if not provided)
            metrics: Performance metrics dict
            training_examples_count: Number of training examples used
            optimization_params: Optimization hyperparameters
            notes: Optional notes about this version

        Returns:
            Model ID
        """
        # Auto-increment version if not provided
        if version is None:
            version = self._get_next_version(module_type)

        # Serialize module to file
        timestamp = datetime.utcnow().isoformat().replace(":", "-")
        filename = f"{module_type}_v{version}_{timestamp}.pkl"
        file_path = self.models_dir / filename

        with open(file_path, "wb") as f:
            pickle.dump(module, f)

        # Store metadata in database
        created_at = datetime.utcnow().isoformat()
        metrics_json = json.dumps(metrics) if metrics else None
        params_json = json.dumps(
            optimization_params) if optimization_params else None

        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                INSERT INTO models (
                    id, module_type, version, created_at, file_path,
                    metrics, training_examples_count, optimization_params, notes
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                model_id, module_type, version, created_at, str(file_path),
                metrics_json, training_examples_count, params_json, notes
            ))
            conn.commit()

        logger.info(
            "model_registered",
            model_id=model_id,
            module_type=module_type,
            version=version,
            metrics=metrics
        )

        return model_id

    def load_model(self, model_id: Optional[str] = None, module_type: Optional[str] = None, version: Optional[int] = None):
        """
        Load a model from registry.

        Args:
            model_id: Specific model ID to load
            module_type: Load active model for this type
            version: Load specific version (requires module_type)

        Returns:
            Loaded DSPy module
        """
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row

            if model_id:
                row = conn.execute(
                    "SELECT * FROM models WHERE id = ?",
                    (model_id,)
                ).fetchone()
            elif module_type and version:
                row = conn.execute(
                    "SELECT * FROM models WHERE module_type = ? AND version = ?",
                    (module_type, version)
                ).fetchone()
            elif module_type:
                # Load active model for this type
                row = conn.execute(
                    "SELECT * FROM models WHERE module_type = ? AND is_active = TRUE",
                    (module_type,)
                ).fetchone()
            else:
                raise ValueError(
                    "Must provide model_id, or module_type (with optional version)")

        if not row:
            logger.warning("model_not_found", model_id=model_id,
                           module_type=module_type, version=version)
            return None

        file_path = Path(row["file_path"])

        if not file_path.exists():
            logger.error("model_file_missing", file_path=str(file_path))
            raise FileNotFoundError(f"Model file not found: {file_path}")

        with open(file_path, "rb") as f:
            module = pickle.load(f)

        logger.info(
            "model_loaded",
            model_id=row["id"],
            module_type=row["module_type"],
            version=row["version"]
        )

        return module

    def set_active_model(self, model_id: str):
        """
        Set a model as the active version for its type.

        Args:
            model_id: Model ID to activate
        """
        with sqlite3.connect(self.db_path) as conn:
            # Get module type
            row = conn.execute(
                "SELECT module_type FROM models WHERE id = ?",
                (model_id,)
            ).fetchone()

            if not row:
                raise ValueError(f"Model not found: {model_id}")

            module_type = row[0]

            # Deactivate all models of this type
            conn.execute(
                "UPDATE models SET is_active = FALSE WHERE module_type = ?",
                (module_type,)
            )

            # Activate specified model
            conn.execute(
                "UPDATE models SET is_active = TRUE WHERE id = ?",
                (model_id,)
            )

            conn.commit()

        logger.info("active_model_set", model_id=model_id,
                    module_type=module_type)

    def get_model_info(self, model_id: str) -> Optional[Dict[str, Any]]:
        """
        Get metadata for a model.

        Args:
            model_id: Model ID

        Returns:
            Model metadata dict
        """
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            row = conn.execute(
                "SELECT * FROM models WHERE id = ?",
                (model_id,)
            ).fetchone()

        if not row:
            return None

        info = dict(row)
        if info["metrics"]:
            info["metrics"] = json.loads(info["metrics"])
        if info["optimization_params"]:
            info["optimization_params"] = json.loads(
                info["optimization_params"])

        return info

    def list_models(self, module_type: Optional[str] = None, active_only: bool = False) -> List[Dict[str, Any]]:
        """
        List registered models.

        Args:
            module_type: Filter by module type
            active_only: Only return active models

        Returns:
            List of model metadata dicts
        """
        query = "SELECT * FROM models"
        conditions = []

        if module_type:
            conditions.append(f"module_type = '{module_type}'")

        if active_only:
            conditions.append("is_active = TRUE")

        if conditions:
            query += " WHERE " + " AND ".join(conditions)

        query += " ORDER BY module_type, version DESC"

        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            rows = conn.execute(query).fetchall()

        models = []
        for row in rows:
            model_dict = dict(row)
            if model_dict["metrics"]:
                model_dict["metrics"] = json.loads(model_dict["metrics"])
            if model_dict["optimization_params"]:
                model_dict["optimization_params"] = json.loads(
                    model_dict["optimization_params"])
            models.append(model_dict)

        logger.info(
            "models_listed",
            count=len(models),
            module_type=module_type,
            active_only=active_only
        )

        return models

    def _get_next_version(self, module_type: str) -> int:
        """Get next version number for module type."""
        with sqlite3.connect(self.db_path) as conn:
            row = conn.execute(
                "SELECT MAX(version) FROM models WHERE module_type = ?",
                (module_type,)
            ).fetchone()

        max_version = row[0] if row[0] is not None else 0
        return max_version + 1

    def delete_model(self, model_id: str):
        """
        Delete a model from registry.

        Args:
            model_id: Model ID to delete
        """
        with sqlite3.connect(self.db_path) as conn:
            # Get file path
            row = conn.execute(
                "SELECT file_path FROM models WHERE id = ?",
                (model_id,)
            ).fetchone()

            if row:
                file_path = Path(row[0])
                if file_path.exists():
                    file_path.unlink()

                # Delete from database
                conn.execute("DELETE FROM models WHERE id = ?", (model_id,))
                conn.commit()

                logger.info("model_deleted", model_id=model_id)
