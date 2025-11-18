#!/usr/bin/env python3
"""
Compile micro-models from .mlpackage to .mlmodelc format.

This script uses CoreML's compilation API via a simple Python wrapper.
"""

import sys
import subprocess
from pathlib import Path


def compile_using_swift(model_path: Path) -> bool:
    """Compile a model using Swift's MLModel.compileModel."""
    try:
        # Create a temporary Swift script to compile the model
        swift_code = f'''
import CoreML

let modelURL = URL(fileURLWithPath: "{model_path.absolute()}")
let compiledURL = try MLModel.compileModel(at: modelURL)
print(compiledURL.path)
'''

        result = subprocess.run(
            ["swift", "-"],
            input=swift_code,
            text=True,
            capture_output=True,
            timeout=60
        )

        if result.returncode == 0:
            compiled_path = result.stdout.strip()
            expected_path = model_path.with_suffix(".mlpackage.mlmodelc")

            # Move to expected location if different
            if compiled_path != str(expected_path):
                import shutil
                if expected_path.exists():
                    shutil.rmtree(expected_path)
                shutil.move(compiled_path, str(expected_path))

            return True
        else:
            print(f"Swift compilation error: {result.stderr}", file=sys.stderr)
            return False

    except subprocess.TimeoutExpired:
        print(f"Compilation timed out for {model_path}", file=sys.stderr)
        return False
    except Exception as e:
        print(f"Error compiling {model_path}: {e}", file=sys.stderr)
        return False


def main():
    """Compile all micro-models."""
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent
    models_dir = project_root / "models" / "coreml" / "micro"

    print("=" * 60)
    print("Compiling micro-models")
    print("=" * 60)
    print(f"Models directory: {models_dir}\n")

    # Find all .mlpackage files
    mlpackage_files = list(models_dir.glob("*.mlpackage"))

    if not mlpackage_files:
        print("No .mlpackage files found to compile.")
        return

    success_count = 0
    for model_path in mlpackage_files:
        compiled_path = model_path.with_suffix(".mlpackage.mlmodelc")

        if compiled_path.exists():
            print(f"✅ {model_path.name} - Already compiled")
            success_count += 1
            continue

        print(f"🔨 Compiling: {model_path.name}...")
        if compile_using_swift(model_path):
            print(f"✅ {model_path.name} - Compiled successfully")
            success_count += 1
        else:
            print(f"❌ {model_path.name} - Compilation failed")

    print("\n" + "=" * 60)
    print(f"Compiled {success_count}/{len(mlpackage_files)} models")
    print("=" * 60)


if __name__ == "__main__":
    main()





