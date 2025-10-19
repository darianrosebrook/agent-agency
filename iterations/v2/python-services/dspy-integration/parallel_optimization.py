#!/usr/bin/env python3
"""
Parallel DSPy Optimization Runner

This script demonstrates multiprocessing and joblib usage for parallel execution
of CPU-bound DSPy optimization tasks.

@ author @darianrosebrook
"""

import multiprocessing
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from typing import List, Dict, Any
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# Mock DSPy optimization task (replace with actual DSPy logic)
def optimize_dspy_signature(signature_data: Dict[str, Any], optimization_config: Dict[str, Any]) -> Dict[str, Any]:
    """
    Mock DSPy signature optimization task.

    In real implementation, this would use DSPy to optimize signatures
    based on the provided data and configuration.
    """
    import random
    import time

    # Simulate CPU-bound work (optimization computation)
    time.sleep(random.uniform(0.5, 2.0))

    # Mock optimization result
    result = {
        "signature_id": signature_data.get("id", "unknown"),
        "optimized_signature": f"optimized_{signature_data.get('name', 'unknown')}",
        "performance_score": random.uniform(0.7, 0.95),
        "iterations": random.randint(10, 50),
        "processing_time": random.uniform(0.5, 2.0)
    }

    logger.info(f"Optimized signature {result['signature_id']} - Score: {result['performance_score']:.3f}")
    return result

def run_parallel_optimization_multiprocessing(
    signatures: List[Dict[str, Any]],
    optimization_config: Dict[str, Any],
    max_workers: int = None
) -> List[Dict[str, Any]]:
    """
    Run DSPy optimization using multiprocessing.Pool for CPU-bound tasks.
    """
    if max_workers is None:
        max_workers = min(multiprocessing.cpu_count(), len(signatures))

    logger.info(f"Starting multiprocessing optimization with {max_workers} workers")

    start_time = time.time()

    with multiprocessing.Pool(processes=max_workers) as pool:
        # Prepare arguments for each task
        tasks = [(sig, optimization_config) for sig in signatures]

        # Execute tasks in parallel
        results = pool.starmap(optimize_dspy_signature, tasks)

    total_time = time.time() - start_time
    logger.info(".2f")

    return results

def run_parallel_optimization_joblib(
    signatures: List[Dict[str, Any]],
    optimization_config: Dict[str, Any],
    n_jobs: int = -1
) -> List[Dict[str, Any]]:
    """
    Run DSPy optimization using joblib for parallel execution.
    """
    try:
        from joblib import Parallel, delayed
    except ImportError:
        logger.error("joblib not installed. Install with: uv add joblib")
        return []

    logger.info(f"Starting joblib optimization with {n_jobs} jobs")

    start_time = time.time()

    # Execute tasks in parallel using joblib
    results = Parallel(n_jobs=n_jobs, backend='multiprocessing')(
        delayed(optimize_dspy_signature)(sig, optimization_config)
        for sig in signatures
    )

    total_time = time.time() - start_time
    logger.info(".2f")

    return results

def run_parallel_optimization_concurrent(
    signatures: List[Dict[str, Any]],
    optimization_config: Dict[str, Any],
    max_workers: int = None
) -> List[Dict[str, Any]]:
    """
    Run DSPy optimization using concurrent.futures for fine-grained control.
    """
    if max_workers is None:
        max_workers = min(multiprocessing.cpu_count(), len(signatures))

    logger.info(f"Starting concurrent optimization with {max_workers} workers")

    start_time = time.time()
    results = []

    with ProcessPoolExecutor(max_workers=max_workers) as executor:
        # Submit all tasks
        future_to_signature = {
            executor.submit(optimize_dspy_signature, sig, optimization_config): sig
            for sig in signatures
        }

        # Collect results as they complete
        for future in as_completed(future_to_signature):
            try:
                result = future.result()
                results.append(result)
            except Exception as exc:
                signature = future_to_signature[future]
                logger.error(f"Signature {signature.get('id')} generated an exception: {exc}")

    total_time = time.time() - start_time
    logger.info(".2f")

    return results

def main():
    """Main execution function."""
    # Sample data for demonstration
    sample_signatures = [
        {"id": f"sig_{i}", "name": f"signature_{i}", "data": f"sample_data_{i}"}
        for i in range(10)
    ]

    optimization_config = {
        "max_iterations": 100,
        "learning_rate": 0.01,
        "optimizer": "adam"
    }

    logger.info("Starting DSPy parallel optimization demonstration")

    # Test different parallel execution methods
    methods = [
        ("Multiprocessing", run_parallel_optimization_multiprocessing),
        ("Joblib", run_parallel_optimization_joblib),
        ("Concurrent Futures", run_parallel_optimization_concurrent),
    ]

    for method_name, method_func in methods:
        logger.info(f"\n--- Testing {method_name} ---")
        try:
            results = method_func(sample_signatures, optimization_config)

            # Calculate statistics
            scores = [r["performance_score"] for r in results]
            avg_score = sum(scores) / len(scores)
            max_score = max(scores)

            logger.info(f"{method_name} Results:")
            logger.info(f"  Average Score: {avg_score:.3f}")
            logger.info(f"  Best Score: {max_score:.3f}")
            logger.info(f"  Total Signatures: {len(results)}")

        except Exception as e:
            logger.error(f"{method_name} failed: {e}")

if __name__ == "__main__":
    main()
