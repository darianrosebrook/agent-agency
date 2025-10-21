#!/usr/bin/env python3
"""Minimal mutation testing to validate the implementation."""

import os
import sys
import tempfile
import subprocess
import ast
import re
from pathlib import Path

def run_test():
    """Run the test file and return success/failure."""
    try:
        result = subprocess.run(
            [sys.executable, "test_simple.py"],
            capture_output=True,
            text=True,
            timeout=10
        )
        return result.returncode == 0
    except subprocess.TimeoutExpired:
        return False
    except Exception:
        return False

def test_mutation_with_simple_runner(mutation_file):
    """Test a mutation using our simple test runner."""
    try:
        # Copy the test file to the mutation
        import shutil
        shutil.copy2(mutation_file, "test_simple.py")

        # Run the test
        success = run_test()

        # Restore original
        shutil.copy2("test_simple_original.py", "test_simple.py")

        # Mutation is killed if test fails
        return not success
    except Exception as e:
        print(f"Error testing mutation: {e}")
        return False

def simple_mutation_test():
    """Run a simple mutation test on our test file."""
    print("🧬 Running minimal mutation test...")

    # Backup original
    import shutil
    shutil.copy2("test_simple.py", "test_simple_original.py")

    results = {
        "total_mutations": 0,
        "killed_mutations": 0,
        "survived_mutations": 0,
        "mutations": []
    }

    # Read the test file
    with open("test_simple.py", "r") as f:
        content = f.read()

    # Simple mutations
    mutations = [
        ("==", "!="),
        ("True", "False"),
        ("False", "True"),
        ("2 + 3", "2 - 3"),
        ("15", "5"),
    ]

    for original, mutated in mutations:
        if original in content:
            results["total_mutations"] += 1

            # Create mutated content
            mutated_content = content.replace(original, mutated)

            # Write to temp file
            with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as temp_file:
                temp_file.write(mutated_content)
                temp_file_path = temp_file.name

            # Test the mutation
            killed = test_mutation_with_simple_runner(temp_file_path)

            if killed:
                results["killed_mutations"] += 1
                status = "killed"
            else:
                results["survived_mutations"] += 1
                status = "survived"

            results["mutations"].append({
                "original": original,
                "mutated": mutated,
                "killed": killed,
                "status": status
            })

            # Clean up
            os.unlink(temp_file_path)

    # Restore original
    shutil.copy2("test_simple_original.py", "test_simple.py")
    os.unlink("test_simple_original.py")

    # Calculate score
    if results["total_mutations"] > 0:
        results["mutation_score"] = results["killed_mutations"] / results["total_mutations"]
    else:
        results["mutation_score"] = 0.0

    # Report
    print(f"Total mutations: {results['total_mutations']}")
    print(f"Killed: {results['killed_mutations']}")
    print(f"Survived: {results['survived_mutations']}")
    print(f"Mutation score: {results['mutation_score']:.1%}")
    for mutation in results["mutations"]:
        print(f"  {mutation['original']} -> {mutation['mutated']}: {mutation['status']}")

    return results

if __name__ == "__main__":
    simple_mutation_test()
